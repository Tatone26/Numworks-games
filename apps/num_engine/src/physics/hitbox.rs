//! Axis-aligned bounding box (AABB) queries, collision resolution, and generic tilemap tests.

use crate::graphics::tilemap::Tilemap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Hitbox {
    pub offset_x: i16,
    pub offset_y: i16,
    pub width: u16,
    pub height: u16,
    pub active: bool,
}

impl Hitbox {
    pub const fn new(offset_x: i16, offset_y: i16, width: u16, height: u16) -> Self {
        Self {
            offset_x,
            offset_y,
            width,
            height,
            active: true,
        }
    }

    pub const fn new_centered(width: u16, height: u16) -> Self {
        Self {
            offset_x: -((width / 2) as i16),
            offset_y: -((height / 2) as i16),
            width,
            height,
            active: true,
        }
    }

    pub const fn new_inset(width: u16, height: u16, pad_x: u16, pad_y: u16) -> Self {
        Self {
            offset_x: pad_x as i16,
            offset_y: pad_y as i16,
            width: width.saturating_sub(pad_x * 2),
            height: height.saturating_sub(pad_y * 2),
            active: true,
        }
    }

    // --- Spatial Bounds ---

    /// Absolute world-space bounds `[x0, y0, x1, y1]` relative to an anchor origin.
    #[inline(always)]
    pub fn world_bounds(&self, anchor: [i16; 2]) -> [i16; 4] {
        let x0 = anchor[0] + self.offset_x;
        let y0 = anchor[1] + self.offset_y;
        [x0, y0, x0 + self.width as i16, y0 + self.height as i16]
    }

    /// Center coordinate `[cx, cy]` in world space.
    #[inline(always)]
    pub fn center(&self, anchor: [i16; 2]) -> [i16; 2] {
        [
            anchor[0] + self.offset_x + (self.width / 2) as i16,
            anchor[1] + self.offset_y + (self.height / 2) as i16,
        ]
    }

    /// Resolves overlapping tile coordinate range `(col_start, row_start, col_end, row_end)`.
    #[inline(always)]
    pub fn tile_bounds<const TILE_SIZE: usize>(
        &self,
        anchor: [i16; 2],
    ) -> (usize, usize, usize, usize) {
        let [x0, y0, x1, y1] = self.world_bounds(anchor);
        let ts = TILE_SIZE as i16;

        let col_start = (x0.max(0) / ts) as usize;
        let row_start = (y0.max(0) / ts) as usize;
        let col_end = ((x1.max(0) + ts - 1) / ts) as usize;
        let row_end = ((y1.max(0) + ts - 1) / ts) as usize;

        (col_start, row_start, col_end, row_end)
    }

    // --- Collision & Intersection Tests ---

    #[inline(always)]
    pub fn intersects(&self, pos: [i16; 2], other: &Self, other_pos: [i16; 2]) -> bool {
        if !self.active || !other.active {
            return false;
        }

        let [ax0, ay0, ax1, ay1] = self.world_bounds(pos);
        let [bx0, by0, bx1, by1] = other.world_bounds(other_pos);

        ax0 < bx1 && ax1 > bx0 && ay0 < by1 && ay1 > by0
    }

    /// Returns the overlapping rectangle `[x0, y0, x1, y1]` if intersecting.
    #[inline(always)]
    pub fn overlap_rect(
        &self,
        pos: [i16; 2],
        other: &Self,
        other_pos: [i16; 2],
    ) -> Option<[i16; 4]> {
        if !self.intersects(pos, other, other_pos) {
            return None;
        }

        let [ax0, ay0, ax1, ay1] = self.world_bounds(pos);
        let [bx0, by0, bx1, by1] = other.world_bounds(other_pos);

        Some([ax0.max(bx0), ay0.max(by0), ax1.min(bx1), ay1.min(by1)])
    }

    /// Computes Minimum Translation Vector `[dx, dy]` to push `self` out of `other`.
    #[inline(always)]
    pub fn penetration_depth(
        &self,
        pos: [i16; 2],
        other: &Self,
        other_pos: [i16; 2],
    ) -> Option<[i16; 2]> {
        let [ox0, oy0, ox1, oy1] = self.overlap_rect(pos, other, other_pos)?;
        let overlap_w = ox1 - ox0;
        let overlap_h = oy1 - oy0;

        let [ax0, ay0, _, _] = self.world_bounds(pos);
        let [bx0, by0, _, _] = other.world_bounds(other_pos);

        if overlap_w < overlap_h {
            let push_x = if ax0 < bx0 { -overlap_w } else { overlap_w };
            Some([push_x, 0])
        } else {
            let push_y = if ay0 < by0 { -overlap_h } else { overlap_h };
            Some([0, push_y])
        }
    }

    #[inline(always)]
    pub fn contains_point(&self, pos: [i16; 2], px: i16, py: i16) -> bool {
        if !self.active {
            return false;
        }
        let [x0, y0, x1, y1] = self.world_bounds(pos);
        px >= x0 && px < x1 && py >= y0 && py < y1
    }

    #[inline(always)]
    pub fn collides_tilemap<const TILE_SIZE: usize, const CELL_AREA: usize>(
        &self,
        pos: [i16; 2],
        tilemap: &Tilemap<'_, TILE_SIZE, CELL_AREA>,
    ) -> bool {
        if !self.active {
            return false;
        }
        tilemap.collides_with_rect(
            pos[0] + self.offset_x,
            pos[1] + self.offset_y,
            self.width,
            self.height,
        )
    }

    // --- Modification ---

    #[inline(always)]
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    #[inline(always)]
    pub fn set_offset(&mut self, ox: i16, oy: i16) {
        self.offset_x = ox;
        self.offset_y = oy;
    }
}
