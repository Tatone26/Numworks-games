//! Multi-layered tile grids, parallax multipliers, and infinite boundary wrapping.

use numworks_utils::{
    eadk::{Color, Point},
    graphical::{tiling::Tileset, TRANSPARENCY_COLOR},
};

use crate::texture::Animation;

/// Parallax camera scroll rate expressed as a rational fraction `factor_num / factor_den`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parallax {
    pub factor_num: i16,
    pub factor_den: i16,
}

impl Parallax {
    /// Completely static layer ($0\times$ camera speed). Unaffected by viewport panning.
    pub const FIXED: Self = Self {
        factor_num: 0,
        factor_den: 1,
    };

    /// Foreground layer moving 1:1 with the camera viewport ($1.0\times$).
    pub const FOREGROUND: Self = Self {
        factor_num: 1,
        factor_den: 1,
    };

    /// Creates a distance-based attenuation ratio: `1 / dist`.
    ///
    /// * `dist = 1` -> 1.0x (Foreground)
    /// * `dist = 2` -> 0.5x (Midground)
    /// * `dist = 3` -> 0.33x (Distant mountains/clouds)
    pub const fn from_distance(dist: u16) -> Self {
        Self {
            factor_num: 1,
            factor_den: if dist == 0 { 1 } else { dist as i16 },
        }
    }

    /// Constructs an arbitrary fractional multiplier `num / den`.
    pub const fn ratio(num: i16, den: i16) -> Self {
        Self {
            factor_num: num,
            factor_den: if den == 0 { 1 } else { den },
        }
    }

    /// Computes the effective camera coordinate after applying the parallax scalar.
    #[inline(always)]
    pub fn apply(&self, cam: i32) -> i32 {
        if self.factor_num == 0 {
            0
        } else if self.factor_num == self.factor_den {
            cam
        } else {
            (cam * self.factor_num as i32) / (self.factor_den as i32)
        }
    }
}

/// A renderable grid layer of tile indices with independent parallax, origin, and wrapping properties.
///
/// Backed by a mutable slice of tile coordinate indices `[tx, ty]`, allowing tilemaps to be sized
/// independently of the physical screen or full world dimensions.
///
/// # Generics
/// * `TILE_SIZE` - Width and height of one standard tile in pixels (typically 20).
/// * `CELL_AREA` - Total pixels per tile (`TILE_SIZE * TILE_SIZE`, typically 400).
pub struct Tilemap<'a, const TILE_SIZE: usize, const CELL_AREA: usize> {
    tileset: &'a Tileset,
    data: &'a mut [Option<[u8; 2]>],
    cols: usize,
    rows: usize,
    anim_slots: &'a [&'a Animation<'a>],
    pub(crate) z: u8,
    pub(crate) transparent: bool,
    parallax: Parallax,
    wrap: bool,
    origin_x: i32,
    origin_y: i32,
}

impl<'a, const TILE_SIZE: usize, const CELL_AREA: usize> Tilemap<'a, TILE_SIZE, CELL_AREA> {
    /// Constructs a tilemap layer.
    ///
    /// If `transparent` is set to `false`, the Compositor marks this layer as an opaque base,
    /// bypassing clear-color fills on covered cells.
    pub fn new(
        tileset: &'a Tileset,
        data: &'a mut [Option<[u8; 2]>],
        cols: usize,
        rows: usize,
        anim_slots: &'a [&'a Animation<'a>],
        z: u8,
        transparent: bool,
        parallax: Parallax,
        wrap: bool,
    ) -> Self {
        Self {
            tileset,
            data,
            cols,
            rows,
            anim_slots,
            z,
            transparent,
            parallax,
            wrap,
            origin_x: 0,
            origin_y: 0,
        }
    }

    /// Sets an anchor offset in world-space pixels where `(0, 0)` of this tilemap begins.
    #[inline(always)]
    pub fn set_origin(&mut self, x: i32, y: i32) {
        self.origin_x = x;
        self.origin_y = y;
    }

    /// Retrieves the tile coordinate `[tx, ty]` at grid position `(x, y)`.
    #[inline(always)]
    pub fn get_tile(&self, x: usize, y: usize) -> Option<[u8; 2]> {
        if x < self.cols && y < self.rows {
            self.data[y * self.cols + x]
        } else {
            None
        }
    }

    /// Overrides the tile coordinate `[tx, ty]` at grid position `(x, y)`.
    #[inline(always)]
    pub fn set_tile(&mut self, x: usize, y: usize, tile: Option<[u8; 2]>) {
        if x < self.cols && y < self.rows {
            self.data[y * self.cols + x] = tile;
        }
    }

    /// Blits overlapping tiles for screen cell `(cx, cy)` into the provided scratch buffer.
    ///
    /// Translates camera coordinates according to the layer's [`Parallax`] and handles sub-tile seam quadrant copies.
    pub(crate) fn blit_to_cell(
        &self,
        cx: usize,
        cy: usize,
        cam_x: i32,
        cam_y: i32,
        cell_buf: &mut [Color; CELL_AREA],
        frame_counter: u32,
    ) {
        let eff_x = self.parallax.apply(cam_x) - self.origin_x;
        let eff_y = self.parallax.apply(cam_y) - self.origin_y;

        let world_x = (cx * TILE_SIZE) as i32 + eff_x;
        let world_y = (cy * TILE_SIZE) as i32 + eff_y;

        let base_tx = world_x.div_euclid(TILE_SIZE as i32);
        let base_ty = world_y.div_euclid(TILE_SIZE as i32);
        let sub_x = world_x.rem_euclid(TILE_SIZE as i32) as usize;
        let sub_y = world_y.rem_euclid(TILE_SIZE as i32) as usize;

        // Path 1: Perfect tile boundary alignment
        if sub_x == 0 && sub_y == 0 {
            if let Some(tile) = self.get_tile_slice(base_tx, base_ty, frame_counter) {
                if !self.transparent {
                    cell_buf.copy_from_slice(&tile[..CELL_AREA]);
                } else {
                    for i in 0..CELL_AREA {
                        let p = tile[i];
                        if p.rgb565 != TRANSPARENCY_COLOR.rgb565 {
                            cell_buf[i] = p;
                        }
                    }
                }
            }
            return;
        }

        // Path 2: Pure Vertical Scroll
        if sub_x == 0 {
            let h_top = TILE_SIZE - sub_y;
            let h_bottom = sub_y;

            if let Some(t_top) = self.get_tile_slice(base_tx, base_ty, frame_counter) {
                self.blit_span(cell_buf, 0, &t_top[sub_y * TILE_SIZE..], h_top * TILE_SIZE);
            }
            if let Some(t_bot) = self.get_tile_slice(base_tx, base_ty + 1, frame_counter) {
                self.blit_span(
                    cell_buf,
                    h_top * TILE_SIZE,
                    &t_bot[..h_bottom * TILE_SIZE],
                    h_bottom * TILE_SIZE,
                );
            }
            return;
        }

        // Path 3: Pure Horizontal Scroll
        let w_left = TILE_SIZE - sub_x;
        let w_right = sub_x;

        if sub_y == 0 {
            let t_left = self.get_tile_slice(base_tx, base_ty, frame_counter);
            let t_right = self.get_tile_slice(base_tx + 1, base_ty, frame_counter);

            for row in 0..TILE_SIZE {
                let row_offset = row * TILE_SIZE;
                if let Some(tl) = t_left {
                    self.blit_span(
                        &mut cell_buf[row_offset..],
                        0,
                        &tl[row_offset + sub_x..],
                        w_left,
                    );
                }
                if let Some(tr) = t_right {
                    self.blit_span(
                        &mut cell_buf[row_offset..],
                        w_left,
                        &tr[row_offset..],
                        w_right,
                    );
                }
            }
            return;
        }

        // Path 4: Diagonal Scroll
        let h_top = TILE_SIZE - sub_y;
        let h_bottom = sub_y;

        let t_tl = self.get_tile_slice(base_tx, base_ty, frame_counter);
        let t_tr = self.get_tile_slice(base_tx + 1, base_ty, frame_counter);
        let t_bl = self.get_tile_slice(base_tx, base_ty + 1, frame_counter);
        let t_br = self.get_tile_slice(base_tx + 1, base_ty + 1, frame_counter);

        for row in 0..h_top {
            let dst_row = row * TILE_SIZE;
            let src_row = (sub_y + row) * TILE_SIZE;
            if let Some(tl) = t_tl {
                self.blit_span(&mut cell_buf[dst_row..], 0, &tl[src_row + sub_x..], w_left);
            }
            if let Some(tr) = t_tr {
                self.blit_span(&mut cell_buf[dst_row..], w_left, &tr[src_row..], w_right);
            }
        }

        for row in 0..h_bottom {
            let dst_row = (h_top + row) * TILE_SIZE;
            let src_row = row * TILE_SIZE;
            if let Some(bl) = t_bl {
                self.blit_span(&mut cell_buf[dst_row..], 0, &bl[src_row + sub_x..], w_left);
            }
            if let Some(br) = t_br {
                self.blit_span(&mut cell_buf[dst_row..], w_left, &br[src_row..], w_right);
            }
        }
    }

    /// Scans animated tiles and marks intersecting screen cells dirty if an animation stepped.
    pub(crate) fn mark_animated_dirty<const SCREEN_COLS: usize, const SCREEN_ROWS: usize>(
        &self,
        frame_counter: u32,
        grid: &mut crate::dirty_grid::DirtyGrid<TILE_SIZE, SCREEN_COLS, SCREEN_ROWS>,
        cam_x: i32,
        cam_y: i32,
    ) {
        if self.anim_slots.is_empty() {
            return;
        }

        let mut active_mask = 0u16;
        for (i, anim) in self.anim_slots.iter().enumerate() {
            if anim.speed > 0 && frame_counter.is_multiple_of(anim.speed as u32) {
                active_mask |= 1 << i;
            }
        }

        if active_mask == 0 {
            return;
        }

        let eff_x = self.parallax.apply(cam_x) - self.origin_x;
        let eff_y = self.parallax.apply(cam_y) - self.origin_y;

        for cy in 0..SCREEN_ROWS {
            let world_y = (cy * TILE_SIZE) as i32 + eff_y;
            let base_ty = world_y.div_euclid(TILE_SIZE as i32);
            let sub_y = world_y.rem_euclid(TILE_SIZE as i32);
            let max_ty_offset = if sub_y != 0 { 1 } else { 0 };

            for cx in 0..SCREEN_COLS {
                let world_x = (cx * TILE_SIZE) as i32 + eff_x;
                let base_tx = world_x.div_euclid(TILE_SIZE as i32);
                let sub_x = world_x.rem_euclid(TILE_SIZE as i32);
                let max_tx_offset = if sub_x != 0 { 1 } else { 0 };

                let mut is_dirty = false;
                'seam_check: for dty in 0..=max_ty_offset {
                    let ty = base_ty + dty;
                    if !self.wrap && (ty < 0 || ty >= self.rows as i32) {
                        continue;
                    }
                    let rty = ty.rem_euclid(self.rows as i32) as usize;

                    for dtx in 0..=max_tx_offset {
                        let tx = base_tx + dtx;
                        if !self.wrap && (tx < 0 || tx >= self.cols as i32) {
                            continue;
                        }
                        let rtx = tx.rem_euclid(self.cols as i32) as usize;

                        if let Some([coord_tx, _]) = self.data[rty * self.cols + rtx] {
                            if coord_tx >= 128 {
                                let slot = (coord_tx - 128) as usize;
                                if slot < 16 && (active_mask & (1 << slot)) != 0 {
                                    is_dirty = true;
                                    break 'seam_check;
                                }
                            }
                        }
                    }
                }

                if is_dirty {
                    grid.mark_cell(cx, cy);
                }
            }
        }
    }

    /// Fetches the raw pixel slice for a tile, resolving animated tile indices (`tx >= 128`) if present.
    #[inline(always)]
    fn get_tile_slice(&self, tx: i32, ty: i32, frame_counter: u32) -> Option<&'a [Color]> {
        if !self.wrap && (tx < 0 || tx >= self.cols as i32 || ty < 0 || ty >= self.rows as i32) {
            return None;
        }
        let rtx = tx.rem_euclid(self.cols as i32) as usize;
        let rty = ty.rem_euclid(self.rows as i32) as usize;

        let coord = self.data[rty * self.cols + rtx]?;
        let [resolved_tx, resolved_ty] = self.resolve_tile_coord(coord, frame_counter);
        Some(self.tileset.get_tile(Point {
            x: resolved_tx as u16,
            y: resolved_ty as u16,
        }))
    }

    /// Copies a contiguous horizontal span into the cell buffer, handling color-key transparency.
    #[inline(always)]
    fn blit_span(&self, cell_buf: &mut [Color], dst_offset: usize, src: &[Color], len: usize) {
        if len == 0 {
            return;
        }
        let d = &mut cell_buf[dst_offset..dst_offset + len];
        let s = &src[..len];

        if !self.transparent {
            d.copy_from_slice(s);
        } else {
            for i in 0..len {
                let p = s[i];
                if p.rgb565 != TRANSPARENCY_COLOR.rgb565 {
                    d[i] = p;
                }
            }
        }
    }

    /// Resolves dynamic animation slots when tile X coordinates use the animated flag range (`tx >= 128`).
    #[inline(always)]
    fn resolve_tile_coord(&self, coord: [u8; 2], frame_counter: u32) -> [u8; 2] {
        let [mut tx, mut ty] = coord;
        if tx >= 128 {
            let slot = (tx - 128) as usize;
            if let Some(anim) = self.anim_slots.get(slot) {
                let frame_idx = anim.calculate_frame_index(frame_counter);
                if let Some(f) = anim.get_frame(frame_idx) {
                    tx = f.tx;
                    ty = f.ty;
                }
            }
        }
        [tx, ty]
    }
}
