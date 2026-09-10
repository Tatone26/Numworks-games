//! Multi-layered tile grids, parallax multipliers, per-axis wrapping, and sparse cell-level invalidation.

use numworks_utils::{
    eadk::{Color, Point},
    graphical::{tiling::Tileset, TRANSPARENCY_COLOR},
};

use crate::texture::Animation;

/// Maximum tile rows tracked in the fast occupancy bitmask.
pub const MAX_TILEMAP_ROWS: usize = 32;

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
    tileset: &'a Tileset,
    data: &'a mut [Option<[u8; 2]>],
    cols: usize,
    rows: usize,
    anim_slots: &'a [&'a Animation<'a>],
    pub(crate) z: u8,
    pub(crate) transparent: bool,
    parallax: Parallax,
    wrap: WrapMode,
    pub(crate) origin_x: i32,
    pub(crate) origin_y: i32,
    pub(crate) prev_origin_x: i32,
    pub(crate) prev_origin_y: i32,
    scroll: ScrollVelocity,
    sub_pixel_acc: f32,
    pub(crate) moved: bool,
    occupancy: [u32; MAX_TILEMAP_ROWS],
}

impl<'a, const TILE_SIZE: usize, const CELL_AREA: usize> Tilemap<'a, TILE_SIZE, CELL_AREA> {
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
        let mut map = Self {
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
            scroll: ScrollVelocity::Stationary,
            sub_pixel_acc: 0.0,
            moved: false,
            occupancy: [0; MAX_TILEMAP_ROWS],
        };
        map.recalculate_occupancy();
        map
    }

    pub fn recalculate_occupancy(&mut self) {
        for r in 0..self.rows.min(MAX_TILEMAP_ROWS) {
            let mut row_mask = 0u32;
            let offset = r * self.cols;
            for c in 0..self.cols.min(32) {
                if self.data[offset + c].is_some() {
                    row_mask |= 1u32 << c;
                }
            }
            self.occupancy[r] = row_mask;
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

    #[inline(always)]
    pub fn set_origin(&mut self, x: i32, y: i32) {
        if self.origin_x != x || self.origin_y != y {
            self.origin_x = x;
            self.origin_y = y;
            self.moved = true;
        }
    }

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

    #[inline(always)]
    pub fn get_tile(&self, x: usize, y: usize) -> Option<[u8; 2]> {
        if x < self.cols && y < self.rows {
            self.data[y * self.cols + x]
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn set_tile(&mut self, x: usize, y: usize, tile: Option<[u8; 2]>) {
        if x < self.cols && y < self.rows {
            self.data[y * self.cols + x] = tile;
            if y < MAX_TILEMAP_ROWS && x < 32 {
                if tile.is_some() {
                    self.occupancy[y] |= 1u32 << x;
                } else {
                    self.occupancy[y] &= !(1u32 << x);
                }
            }
        }
    }

    /// Front-to-back (Reverse-Z) blit with fused occupancy check and aligned fast-path.
    pub(crate) fn blit_to_cell_reverse_z(
        &self,
        cell_coords: [usize; 2],
        cam: [i32; 2],
        cell_buf: &mut [Color; CELL_AREA],
        covered: &mut [bool; CELL_AREA],
        pixels_covered: &mut usize,
        frame_counter: u32,
    ) {
        let [cx, cy] = cell_coords;
        let [cam_x, cam_y] = cam;

        let eff_x = self.parallax.apply(cam_x) - self.origin_x;
        let eff_y = self.parallax.apply(cam_y) - self.origin_y;

        let world_x = (cx * TILE_SIZE) as i32 + eff_x;
        let world_y = (cy * TILE_SIZE) as i32 + eff_y;

        let base_tx = world_x.div_euclid(TILE_SIZE as i32);
        let base_ty = world_y.div_euclid(TILE_SIZE as i32);
        let sub_x = world_x.rem_euclid(TILE_SIZE as i32) as usize;
        let sub_y = world_y.rem_euclid(TILE_SIZE as i32) as usize;

        let wrap_x = self.wrap.wraps_x();
        let wrap_y = self.wrap.wraps_y();

        // 1. Fast Occupancy check: abort immediately if cell touches only empty air
        let max_dtx = if sub_x != 0 { 1 } else { 0 };
        let max_dty = if sub_y != 0 { 1 } else { 0 };

        let mut has_content = false;
        'occ_check: for dty in 0..=max_dty {
            let ty = base_ty + dty;
            let rty = if wrap_y {
                ty.rem_euclid(self.rows as i32) as usize
            } else if ty >= 0 && ty < self.rows as i32 {
                ty as usize
            } else {
                continue;
            };

            if rty >= MAX_TILEMAP_ROWS {
                has_content = true;
                break;
            }

            let row_mask = self.occupancy[rty];
            if row_mask == 0 {
                continue;
            }

            for dtx in 0..=max_dtx {
                let tx = base_tx + dtx;
                let rtx = if wrap_x {
                    tx.rem_euclid(self.cols as i32) as usize
                } else if tx >= 0 && tx < self.cols as i32 {
                    tx as usize
                } else {
                    continue;
                };

                if rtx < 32 && (row_mask & (1u32 << rtx)) != 0 {
                    has_content = true;
                    break 'occ_check;
                }
            }
        }

        if !has_content {
            return;
        }

        let fast_copy = !self.transparent && *pixels_covered == 0;

        // 2. Aligned Fast Path: Camera is tile-aligned (zero sub-tile offsets)
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

        // 3. Straddled Path: Multi-quadrant sampling
        let w_left = TILE_SIZE - sub_x;
        let w_right = sub_x;
        let h_top = TILE_SIZE - sub_y;
        let h_bot = sub_y;

        let t_tl = self.get_tile_slice(base_tx, base_ty, frame_counter);
        let t_tr = self.get_tile_slice(base_tx + 1, base_ty, frame_counter);
        let t_bl = self.get_tile_slice(base_tx, base_ty + 1, frame_counter);
        let t_br = self.get_tile_slice(base_tx + 1, base_ty + 1, frame_counter);

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
    fn get_tile_slice(&self, tx: i32, ty: i32, frame_counter: u32) -> Option<&'a [Color]> {
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

        let coord = self.data[rty * self.cols + rtx]?;
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

    pub(crate) fn mark_sparse_dirty<const SCREEN_COLS: usize, const SCREEN_ROWS: usize>(
        &self,
        frame_counter: u32,
        grid: &mut crate::dirty_grid::DirtyGrid<TILE_SIZE, SCREEN_COLS, SCREEN_ROWS>,
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

        let eff_x = self.parallax.apply(cam_x) - self.origin_x;
        let eff_y = self.parallax.apply(cam_y) - self.origin_y;
        let prev_eff_x = self.parallax.apply(cam_x) - self.prev_origin_x;
        let prev_eff_y = self.parallax.apply(cam_y) - self.prev_origin_y;

        let wrap_x = self.wrap.wraps_x();
        let wrap_y = self.wrap.wraps_y();

        for cy in 0..SCREEN_ROWS {
            let world_y = (cy * TILE_SIZE) as i32 + eff_y;
            let base_ty = world_y.div_euclid(TILE_SIZE as i32);
            let sub_y = world_y.rem_euclid(TILE_SIZE as i32);
            let max_dty = if sub_y != 0 { 1 } else { 0 };

            let prev_world_y = (cy * TILE_SIZE) as i32 + prev_eff_y;
            let prev_base_ty = prev_world_y.div_euclid(TILE_SIZE as i32);
            let prev_sub_y = prev_world_y.rem_euclid(TILE_SIZE as i32);
            let prev_max_dty = if prev_sub_y != 0 { 1 } else { 0 };

            for cx in 0..SCREEN_COLS {
                let world_x = (cx * TILE_SIZE) as i32 + eff_x;
                let base_tx = world_x.div_euclid(TILE_SIZE as i32);
                let sub_x = world_x.rem_euclid(TILE_SIZE as i32);
                let max_dtx = if sub_x != 0 { 1 } else { 0 };

                let mut mark = false;

                if self.moved {
                    'curr_pos: for dty in 0..=max_dty {
                        let ty = base_ty + dty;
                        let rty = if wrap_y {
                            ty.rem_euclid(self.rows as i32) as usize
                        } else if ty >= 0 && ty < self.rows as i32 {
                            ty as usize
                        } else {
                            continue;
                        };

                        if rty >= MAX_TILEMAP_ROWS {
                            continue;
                        }

                        let row_mask = self.occupancy[rty];
                        if row_mask == 0 {
                            continue;
                        }

                        for dtx in 0..=max_dtx {
                            let tx = base_tx + dtx;
                            let rtx = if wrap_x {
                                tx.rem_euclid(self.cols as i32) as usize
                            } else if tx >= 0 && tx < self.cols as i32 {
                                tx as usize
                            } else {
                                continue;
                            };

                            if rtx < 32 && (row_mask & (1u32 << rtx)) != 0 {
                                mark = true;
                                break 'curr_pos;
                            }
                        }
                    }

                    if !mark {
                        let prev_world_x = (cx * TILE_SIZE) as i32 + prev_eff_x;
                        let prev_base_tx = prev_world_x.div_euclid(TILE_SIZE as i32);
                        let prev_sub_x = prev_world_x.rem_euclid(TILE_SIZE as i32);
                        let prev_max_dtx = if prev_sub_x != 0 { 1 } else { 0 };

                        'prev_pos: for dty in 0..=prev_max_dty {
                            let ty = prev_base_ty + dty;
                            let rty = if wrap_y {
                                ty.rem_euclid(self.rows as i32) as usize
                            } else if ty >= 0 && ty < self.rows as i32 {
                                ty as usize
                            } else {
                                continue;
                            };

                            if rty >= MAX_TILEMAP_ROWS {
                                continue;
                            }

                            let row_mask = self.occupancy[rty];
                            if row_mask == 0 {
                                continue;
                            }

                            for dtx in 0..=prev_max_dtx {
                                let tx = prev_base_tx + dtx;
                                let rtx = if wrap_x {
                                    tx.rem_euclid(self.cols as i32) as usize
                                } else if tx >= 0 && tx < self.cols as i32 {
                                    tx as usize
                                } else {
                                    continue;
                                };

                                if rtx < 32 && (row_mask & (1u32 << rtx)) != 0 {
                                    mark = true;
                                    break 'prev_pos;
                                }
                            }
                        }
                    }
                }

                if !mark && active_anim_mask != 0 {
                    'anim_check: for dty in 0..=max_dty {
                        let ty = base_ty + dty;
                        let rty = if wrap_y {
                            ty.rem_euclid(self.rows as i32) as usize
                        } else if ty >= 0 && ty < self.rows as i32 {
                            ty as usize
                        } else {
                            continue;
                        };

                        for dtx in 0..=max_dtx {
                            let tx = base_tx + dtx;
                            let rtx = if wrap_x {
                                tx.rem_euclid(self.cols as i32) as usize
                            } else if tx >= 0 && tx < self.cols as i32 {
                                tx as usize
                            } else {
                                continue;
                            };

                            if let Some([coord_tx, _]) = self.data[rty * self.cols + rtx] {
                                if coord_tx >= 128 {
                                    let slot = (coord_tx - 128) as usize;
                                    if slot < 16 && (active_anim_mask & (1 << slot)) != 0 {
                                        mark = true;
                                        break 'anim_check;
                                    }
                                }
                            }
                        }
                    }
                }

                if mark {
                    grid.mark_cell(cx, cy);
                }
            }
        }
    }
}
