//! Moving and animated multi-tile entity representation.

use numworks_utils::{
    eadk::{Color, Point},
    graphical::TRANSPARENCY_COLOR,
};

use crate::texture::{Animation, FrameCoord};

/// Describes an active, animated game entity in world or UI space.
///
/// Supports sub-pixel fractional velocity accumulation and multi-tile dimensional footprints.
///
/// # Generics
/// * `TILE_SIZE` - Width and height of one standard tile.
/// * `CELL_AREA` - Total pixels per tile (`TILE_SIZE * TILE_SIZE`).
pub struct Sprite<'a, const TILE_SIZE: usize, const CELL_AREA: usize> {
    /// Active animation metadata and frame table.
    animation: &'a Animation<'a>,
    /// Currently displayed frame index in `animation.frames`.
    pub current_frame_index: usize,
    /// Current integer pixel position `[x, y]`.
    pub position: [i16; 2],
    /// Position during the previous frame, used to compute dirty invalidation rects.
    pub(crate) prev_position: [i16; 2],
    /// Flag indicating whether the sprite moved or changed frame this tick.
    pub(crate) moved: bool,
    /// Velocity in pixels per frame `[vx, vy]`.
    speed: [f32; 2],
    /// Accumulated fractional sub-pixel offsets `[fx, fy]`.
    in_pixel_offset: [f32; 2],
    /// Depth layer for render sorting.
    pub(crate) z: u8,
}

impl<'a, const TILE_SIZE: usize, const CELL_AREA: usize> Sprite<'a, TILE_SIZE, CELL_AREA> {
    /// Constructs a sprite instance bound to an animation at an initial position.
    pub const fn new(animation: &'a Animation<'a>, position: [i16; 2], z: u8) -> Self {
        Self {
            animation,
            current_frame_index: 0,
            position,
            prev_position: position,
            moved: true,
            speed: [0.0, 0.0],
            in_pixel_offset: [0.0, 0.0],
            z,
        }
    }

    /// Returns the active animation frame coordinates `(tx, ty)` on the tileset grid.
    #[inline(always)]
    pub fn current_frame(&self) -> Option<FrameCoord> {
        self.animation.get_frame(self.current_frame_index)
    }

    /// Changes the animation of the sprite. Resets the current frame to 0.
    #[inline(always)]
    pub fn set_animation(&mut self, animation: &'a Animation<'a>) {
        self.animation = animation;
        self.current_frame_index = 0;
        self.moved = true;
    }

    /// Returns the full physical pixel width (`desc.width * tile_size`).
    #[inline(always)]
    pub fn pixel_width(&self) -> u16 {
        self.animation.desc.pixel_width()
    }

    /// Returns the full physical pixel height (`desc.height * tile_size`).
    #[inline(always)]
    pub fn pixel_height(&self) -> u16 {
        self.animation.desc.pixel_height()
    }

    /// Locks current position into `prev_position` and clears `moved`.
    ///
    /// Must be invoked at the end of every game frame after `render_frame`.
    #[inline(always)]
    pub fn commit_frame(&mut self) {
        self.prev_position = self.position;
        self.moved = false;
    }

    /// Explicitly overrides the sprite's position and optional sub-pixel accumulator.
    ///
    /// Flags `moved = true` if the new position differs from the current position.
    pub fn set_position(&mut self, pos: [i16; 2], offset: Option<[f32; 2]>) {
        if self.position[0] != pos[0] || self.position[1] != pos[1] {
            self.position = pos;
            self.moved = true;
        }
        if let Some(off) = offset {
            self.in_pixel_offset = off;
        }
    }

    /// Returns the current speed `[vx, vy]`.
    #[inline(always)]
    pub fn get_speed(&self) -> [f32; 2] {
        self.speed
    }

    /// Sets the directional velocity in pixels per frame.
    #[inline(always)]
    pub fn set_speed(&mut self, speed: [f32; 2]) {
        self.speed = speed;
    }

    /// Advances the animation frame and integrates velocity into integer coordinates.
    ///
    /// Automatically flags `moved = true` when frames change or integer pixels advance.
    ///
    /// # Arguments
    /// * `frames_counter` - Monotonic engine frame number (`engine.get_frame()`).
    pub fn update(&mut self, frames_counter: u32) {
        self.moved = false;

        let next_frame_idx = self.animation.calculate_frame_index(frames_counter);
        if next_frame_idx != self.current_frame_index {
            self.current_frame_index = next_frame_idx;
            self.moved = true;
        }

        self.in_pixel_offset[0] += self.speed[0];
        self.in_pixel_offset[1] += self.speed[1];

        let delta_x = self.in_pixel_offset[0] as i32;
        let delta_y = self.in_pixel_offset[1] as i32;

        if delta_x != 0 {
            let new_x = ((self.position[0] as i32) + delta_x)
                .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            self.position[0] = new_x;
            self.moved = true;
            self.in_pixel_offset[0] -= delta_x as f32;
        }

        if delta_y != 0 {
            let new_y = ((self.position[1] as i32) + delta_y)
                .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            self.position[1] = new_y;
            self.moved = true;
            self.in_pixel_offset[1] -= delta_y as f32;
        }
    }

    /// Rasterizes the intersecting bounds of this sprite directly into a screen cell buffer.
    ///
    /// Handles transparency color-keying, multi-tile lookups, and repeating pattern tiling.
    ///
    /// # Arguments
    /// * `cell_buf` - Target scratch cell pixel buffer.
    /// * `cell_x` - Top-left screen X of the destination cell.
    /// * `cell_y` - Top-left screen Y of the destination cell.
    /// * `screen_x` - Resolved top-left screen X of the sprite.
    /// * `screen_y` - Resolved top-left screen Y of the sprite.
    pub(crate) fn blit_to_cell_at(
        &self,
        cell_buf: &mut [Color; CELL_AREA],
        cell_x: i16,
        cell_y: i16,
        screen_x: i16,
        screen_y: i16,
    ) {
        let spr_w = self.pixel_width() as i16;
        let spr_h = self.pixel_height() as i16;

        let x_start = screen_x.max(cell_x);
        let x_end = (screen_x + spr_w).min(cell_x + TILE_SIZE as i16);
        let y_start = screen_y.max(cell_y);
        let y_end = (screen_y + spr_h).min(cell_y + TILE_SIZE as i16);

        if x_start >= x_end || y_start >= y_end {
            return;
        }

        let frame = match self.current_frame() {
            Some(f) => f,
            None => return,
        };

        let desc = &self.animation.desc;
        let tileset = desc.tileset;
        let tile_size = tileset.tile_size as usize;
        let sheet_w = desc.sheet_w.max(1) as usize;
        let sheet_h = desc.sheet_h.max(1) as usize;

        for cur_y in y_start..y_end {
            let cell_local_y = (cur_y - cell_y) as usize;
            let spr_local_y = (cur_y - screen_y) as usize;

            let tile_y_offset = (spr_local_y / tile_size) % sheet_h;
            let row_in_tile = spr_local_y % tile_size;

            for cur_x in x_start..x_end {
                let cell_local_x = (cur_x - cell_x) as usize;
                let spr_local_x = (cur_x - screen_x) as usize;

                let tile_x_offset = (spr_local_x / tile_size) % sheet_w;
                let col_in_tile = spr_local_x % tile_size;

                let tile = tileset.get_tile(Point {
                    x: frame.tx as u16 + tile_x_offset as u16,
                    y: frame.ty as u16 + tile_y_offset as u16,
                });

                let pixel = tile[row_in_tile * tile_size + col_in_tile];
                let dst_idx = cell_local_y * TILE_SIZE + cell_local_x;

                if dst_idx < CELL_AREA
                    && (!desc.transparency || pixel.rgb565 != TRANSPARENCY_COLOR.rgb565)
                {
                    cell_buf[dst_idx] = pixel;
                }
            }
        }
    }
}
