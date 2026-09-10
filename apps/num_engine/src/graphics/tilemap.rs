//! Multi-layered tile grids, parallax multipliers, per-axis wrapping, and sparse cell-level invalidation.

use numworks_utils::{
    eadk::{Color, Point},
    graphical::{tiling::Tileset, TRANSPARENCY_COLOR},
};

use crate::graphics::texture::Animation;

/// Per-axis infinite tiling configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WrapMode {
    #[default]
    None,
    Horizontal,
    Vertical,
    Both,
}

impl WrapMode {
    #[inline(always)]
    pub const fn wraps_x(self) -> bool {
        matches!(self, Self::Horizontal | Self::Both)
    }

    #[inline(always)]
    pub const fn wraps_y(self) -> bool {
        matches!(self, Self::Vertical | Self::Both)
    }
}

/// Parallax camera scroll rate expressed as a rational fraction `factor_num / factor_den`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parallax {
    pub factor_num: i16,
    pub factor_den: i16,
}

impl Parallax {
    pub const FIXED: Self = Self {
        factor_num: 0,
        factor_den: 1,
    };

    pub const FOREGROUND: Self = Self {
        factor_num: 1,
        factor_den: 1,
    };

    pub const fn from_distance(dist: u16) -> Self {
        Self {
            factor_num: 1,
            factor_den: if dist == 0 { 1 } else { dist as i16 },
        }
    }

    pub const fn ratio(num: i16, den: i16) -> Self {
        Self {
            factor_num: num,
            factor_den: if den == 0 { 1 } else { den },
        }
    }

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollVelocity {
    Stationary,
    Horizontal(f32),
    Vertical(f32),
}

pub struct Tilemap<'a, const TILE_SIZE: usize, const CELL_AREA: usize> {
    pub tileset: &'a Tileset,
    pub data: &'a mut [Option<[u8; 2]>],
    pub cols: usize,
    pub rows: usize,
    pub anim_slots: &'a [&'a Animation<'a>],
    pub z: u8,
    pub transparent: bool,
    pub parallax: Parallax,
    pub wrap: WrapMode,
    /// Internal scrolling origin (camera offset inside the layer)
    pub origin_x: i32,
    pub origin_y: i32,
    pub(crate) prev_origin_x: i32,
    pub(crate) prev_origin_y: i32,
    /// Physical placement anchor in world coordinates [x, y]
    pub offset: [i16; 2],
    pub scroll: ScrollVelocity,
    pub sub_pixel_acc: f32,
    pub(crate) moved: bool,
}

impl<'a, const TILE_SIZE: usize, const CELL_AREA: usize> Tilemap<'a, TILE_SIZE, CELL_AREA> {
    // -------------------------------------------------------------------------
    // 1. Constructors & Builders
    // -------------------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tileset: &'a Tileset,
        data: &'a mut [Option<[u8; 2]>],
        cols: usize,
        rows: usize,
        anim_slots: &'a [&'a Animation<'a>],
        z: u8,
        transparent: bool,
        parallax: Parallax,
        wrap: WrapMode,
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
            prev_origin_x: 0,
            prev_origin_y: 0,
            offset: [0, 0],
            scroll: ScrollVelocity::Stationary,
            sub_pixel_acc: 0.0,
            moved: false,
        }
    }

    #[inline(always)]
    pub const fn with_offset(mut self, offset: [i16; 2]) -> Self {
        self.offset = offset;
        self
    }

    // -------------------------------------------------------------------------
    // 2. Configuration & Position Setters
    // -------------------------------------------------------------------------

    #[inline(always)]
    pub fn set_offset(&mut self, offset: [i16; 2]) {
        self.offset = offset;
        self.moved = true;
    }

    #[inline(always)]
    pub fn set_origin(&mut self, x: i32, y: i32) {
        if self.origin_x != x || self.origin_y != y {
            self.origin_x = x;
            self.origin_y = y;
            self.moved = true;
        }
    }

    #[inline(always)]
    pub fn set_horizontal_speed(&mut self, speed: f32) {
        self.scroll = if speed == 0.0 {
            ScrollVelocity::Stationary
        } else {
            ScrollVelocity::Horizontal(speed)
        };
    }

    #[inline(always)]
    pub fn set_vertical_speed(&mut self, speed: f32) {
        self.scroll = if speed == 0.0 {
            ScrollVelocity::Stationary
        } else {
            ScrollVelocity::Vertical(speed)
        };
    }

    // -------------------------------------------------------------------------
    // 3. Tile Access & Mutation
    // -------------------------------------------------------------------------

    #[inline(always)]
    pub fn get_tile(&self, x: usize, y: usize) -> Option<[u8; 2]> {
        if x < self.cols && y < self.rows {
            self.data[y * self.cols + x]
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn set_tile(&mut self, x: usize, y: usize, tile: impl Into<Option<[u8; 2]>>) {
        if x < self.cols && y < self.rows {
            self.data[y * self.cols + x] = tile.into();
            self.moved = true;
        }
    }

    // -------------------------------------------------------------------------
    // 4. Lifecycle & Frame Synchronization
    // -------------------------------------------------------------------------

    pub fn update(&mut self) {
        match self.scroll {
            ScrollVelocity::Stationary => {}
            ScrollVelocity::Horizontal(speed) => {
                self.sub_pixel_acc += speed;
                let delta = self.sub_pixel_acc as i32;
                if delta != 0 {
                    self.origin_x -= delta;
                    self.sub_pixel_acc -= delta as f32;
                    self.moved = true;
                }
            }
            ScrollVelocity::Vertical(speed) => {
                self.sub_pixel_acc += speed;
                let delta = self.sub_pixel_acc as i32;
                if delta != 0 {
                    self.origin_y -= delta;
                    self.sub_pixel_acc -= delta as f32;
                    self.moved = true;
                }
            }
        }
    }

    #[inline(always)]
    pub(crate) fn commit_frame(&mut self) {
        self.prev_origin_x = self.origin_x;
        self.prev_origin_y = self.origin_y;
        self.moved = false;
    }

    // -------------------------------------------------------------------------
    // 5. Spatial & Collision Queries
    // -------------------------------------------------------------------------

    /// Tests intersection with an axis-aligned bounding box taking `offset` and `wrap` into account.
    pub fn collides_with_rect(&self, x: i16, y: i16, width: u16, height: u16) -> bool {
        if width == 0 || height == 0 {
            return false;
        }

        let local_x = (x - self.offset[0]) as i32;
        let local_y = (y - self.offset[1]) as i32;
        let x1 = local_x + width as i32;
        let y1 = local_y + height as i32;

        let ts = TILE_SIZE as i32;
        let c_start = local_x.div_euclid(ts);
        let r_start = local_y.div_euclid(ts);
        let c_end = (x1 - 1).div_euclid(ts);
        let r_end = (y1 - 1).div_euclid(ts);

        for r in r_start..=r_end {
            for c in c_start..=c_end {
                if self.has_tile(c, r) {
                    return true;
                }
            }
        }
        false
    }

    // -------------------------------------------------------------------------
    // 6. Cell Rasterization (Reverse-Z)
    // -------------------------------------------------------------------------

    /// Front-to-back blit with fast-path alignment and multi-quadrant straddling.
    pub(crate) fn blit_to_cell_reverse_z(
        &self,
        cell_coords: [usize; 2],
        cam: [i32; 2],
        cell_buf: &mut [Color; CELL_AREA],
        covered: &mut [bool; CELL_AREA],
        pixels_covered: &mut usize,
        frame_counter: u32,
    ) {
        if *pixels_covered >= CELL_AREA {
            return;
        }

        let [cx, cy] = cell_coords;
        let [cam_x, cam_y] = cam;

        // Apply parallax, internal origin scrolling, and layer offset
        let eff_x = self.parallax.apply(cam_x) - self.origin_x - self.offset[0] as i32;
        let eff_y = self.parallax.apply(cam_y) - self.origin_y - self.offset[1] as i32;

        let world_x = (cx * TILE_SIZE) as i32 + eff_x;
        let world_y = (cy * TILE_SIZE) as i32 + eff_y;

        let ts = TILE_SIZE as i32;
        let base_tx = world_x.div_euclid(ts);
        let base_ty = world_y.div_euclid(ts);
        let sub_x = world_x.rem_euclid(ts) as usize;
        let sub_y = world_y.rem_euclid(ts) as usize;

        let fast_copy = !self.transparent && *pixels_covered == 0;

        // Path A: Aligned Fast Path (Zero sub-tile boundary seams)
        if sub_x == 0 && sub_y == 0 {
            if let Some(tile) = self.get_tile_slice(base_tx, base_ty, frame_counter) {
                Self::blit_span(
                    cell_buf,
                    covered,
                    pixels_covered,
                    0,
                    &tile[..CELL_AREA],
                    self.transparent,
                    fast_copy,
                );
                if fast_copy {
                    *pixels_covered = CELL_AREA;
                }
            }
            return;
        }

        // Path B: Straddled Multi-Quadrant Path
        let t_tl = self.get_tile_slice(base_tx, base_ty, frame_counter);
        let t_tr = if sub_x != 0 {
            self.get_tile_slice(base_tx + 1, base_ty, frame_counter)
        } else {
            None
        };
        let t_bl = if sub_y != 0 {
            self.get_tile_slice(base_tx, base_ty + 1, frame_counter)
        } else {
            None
        };
        let t_br = if sub_x != 0 && sub_y != 0 {
            self.get_tile_slice(base_tx + 1, base_ty + 1, frame_counter)
        } else {
            None
        };

        // If all overlapping quadrants are empty, skip cell entirely
        if t_tl.is_none() && t_tr.is_none() && t_bl.is_none() && t_br.is_none() {
            return;
        }

        let w_left = TILE_SIZE - sub_x;
        let w_right = sub_x;
        let h_top = TILE_SIZE - sub_y;
        let h_bot = sub_y;

        for row in 0..h_top {
            let dst_row = row * TILE_SIZE;
            let src_row = (sub_y + row) * TILE_SIZE;

            if let Some(tl) = t_tl {
                Self::blit_span(
                    cell_buf,
                    covered,
                    pixels_covered,
                    dst_row,
                    &tl[src_row + sub_x..src_row + sub_x + w_left],
                    self.transparent,
                    fast_copy,
                );
            }
            if let Some(tr) = t_tr {
                Self::blit_span(
                    cell_buf,
                    covered,
                    pixels_covered,
                    dst_row + w_left,
                    &tr[src_row..src_row + w_right],
                    self.transparent,
                    fast_copy,
                );
            }
        }

        for row in 0..h_bot {
            let dst_row = (h_top + row) * TILE_SIZE;
            let src_row = row * TILE_SIZE;

            if let Some(bl) = t_bl {
                Self::blit_span(
                    cell_buf,
                    covered,
                    pixels_covered,
                    dst_row,
                    &bl[src_row + sub_x..src_row + sub_x + w_left],
                    self.transparent,
                    fast_copy,
                );
            }
            if let Some(br) = t_br {
                Self::blit_span(
                    cell_buf,
                    covered,
                    pixels_covered,
                    dst_row + w_left,
                    &br[src_row..src_row + w_right],
                    self.transparent,
                    fast_copy,
                );
            }
        }

        if fast_copy {
            *pixels_covered = CELL_AREA;
        }
    }

    // -------------------------------------------------------------------------
    // 7. Dirty Grid Invalidation
    // -------------------------------------------------------------------------

    pub(crate) fn mark_sparse_dirty<const SCREEN_COLS: usize, const SCREEN_ROWS: usize>(
        &self,
        frame_counter: u32,
        grid: &mut crate::graphics::dirty_grid::DirtyGrid<TILE_SIZE, SCREEN_COLS, SCREEN_ROWS>,
        cam_x: i32,
        cam_y: i32,
    ) {
        let mut active_anim_mask = 0u16;
        for (i, anim) in self.anim_slots.iter().enumerate() {
            if anim.speed > 0 && frame_counter.is_multiple_of(anim.speed as u32) {
                active_anim_mask |= 1 << i;
            }
        }

        if !self.moved && active_anim_mask == 0 {
            return;
        }

        let eff_x = self.parallax.apply(cam_x) - self.origin_x - self.offset[0] as i32;
        let eff_y = self.parallax.apply(cam_y) - self.origin_y - self.offset[1] as i32;
        let prev_eff_x = self.parallax.apply(cam_x) - self.prev_origin_x - self.offset[0] as i32;
        let prev_eff_y = self.parallax.apply(cam_y) - self.prev_origin_y - self.offset[1] as i32;

        for cy in 0..SCREEN_ROWS {
            let world_y = (cy * TILE_SIZE) as i32 + eff_y;
            let prev_world_y = (cy * TILE_SIZE) as i32 + prev_eff_y;

            for cx in 0..SCREEN_COLS {
                let world_x = (cx * TILE_SIZE) as i32 + eff_x;
                let prev_world_x = (cx * TILE_SIZE) as i32 + prev_eff_x;

                let mut mark = false;

                if self.moved {
                    mark = self.cell_has_tiles(world_x, world_y)
                        || self.cell_has_tiles(prev_world_x, prev_world_y);
                }

                if !mark && active_anim_mask != 0 {
                    mark = self.cell_has_anim(world_x, world_y, active_anim_mask);
                }

                if mark {
                    grid.mark_cell(cx, cy);
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // 8. Private Helpers
    // -------------------------------------------------------------------------

    #[inline(always)]
    fn blit_span(
        cell_buf: &mut [Color],
        covered: &mut [bool],
        pixels_covered: &mut usize,
        dst_offset: usize,
        src: &[Color],
        is_transparent: bool,
        fast_copy: bool,
    ) {
        let len = src.len();
        if len == 0 {
            return;
        }

        if fast_copy {
            cell_buf[dst_offset..dst_offset + len].copy_from_slice(src);
            return;
        }

        for (i, &p) in src.iter().enumerate() {
            let dst_idx = dst_offset + i;
            if !covered[dst_idx] && (!is_transparent || p.rgb565 != TRANSPARENCY_COLOR.rgb565) {
                cell_buf[dst_idx] = p;
                covered[dst_idx] = true;
                *pixels_covered += 1;
            }
        }
    }

    #[inline(always)]
    fn has_tile(&self, tx: i32, ty: i32) -> bool {
        self.get_tile_coord(tx, ty).is_some()
    }

    #[inline(always)]
    fn get_tile_coord(&self, tx: i32, ty: i32) -> Option<[u8; 2]> {
        let wrap_x = self.wrap.wraps_x();
        let wrap_y = self.wrap.wraps_y();

        let rtx = if wrap_x {
            tx.rem_euclid(self.cols as i32) as usize
        } else if tx >= 0 && tx < self.cols as i32 {
            tx as usize
        } else {
            return None;
        };

        let rty = if wrap_y {
            ty.rem_euclid(self.rows as i32) as usize
        } else if ty >= 0 && ty < self.rows as i32 {
            ty as usize
        } else {
            return None;
        };

        self.data[rty * self.cols + rtx]
    }

    #[inline(always)]
    fn get_tile_slice(&self, tx: i32, ty: i32, frame_counter: u32) -> Option<&'a [Color]> {
        let coord = self.get_tile_coord(tx, ty)?;
        let [resolved_tx, resolved_ty] = self.resolve_tile_coord(coord, frame_counter);
        Some(self.tileset.get_tile(Point {
            x: resolved_tx as u16,
            y: resolved_ty as u16,
        }))
    }

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

    #[inline(always)]
    fn cell_has_tiles(&self, world_x: i32, world_y: i32) -> bool {
        let ts = TILE_SIZE as i32;
        let base_tx = world_x.div_euclid(ts);
        let base_ty = world_y.div_euclid(ts);
        let max_dtx = if world_x.rem_euclid(ts) != 0 { 1 } else { 0 };
        let max_dty = if world_y.rem_euclid(ts) != 0 { 1 } else { 0 };

        for dty in 0..=max_dty {
            for dtx in 0..=max_dtx {
                if self.has_tile(base_tx + dtx, base_ty + dty) {
                    return true;
                }
            }
        }
        false
    }

    #[inline(always)]
    fn cell_has_anim(&self, world_x: i32, world_y: i32, mask: u16) -> bool {
        let ts = TILE_SIZE as i32;
        let base_tx = world_x.div_euclid(ts);
        let base_ty = world_y.div_euclid(ts);
        let max_dtx = if world_x.rem_euclid(ts) != 0 { 1 } else { 0 };
        let max_dty = if world_y.rem_euclid(ts) != 0 { 1 } else { 0 };

        for dty in 0..=max_dty {
            for dtx in 0..=max_dtx {
                if let Some(coord) = self.get_tile_coord(base_tx + dtx, base_ty + dty) {
                    if coord[0] >= 128 {
                        let slot = (coord[0] - 128) as usize;
                        if slot < 16 && (mask & (1 << slot)) != 0 {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}
