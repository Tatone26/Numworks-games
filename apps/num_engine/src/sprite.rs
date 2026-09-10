//! Moving and animated multi-tile entity representation.

use numworks_utils::{
    eadk::{Color, Point},
    graphical::TRANSPARENCY_COLOR,
};

use crate::texture::{Animation, FrameCoord};

/// Describes an active, animated game entity in world or UI space.
///
/// Supports sub-pixel fractional velocity accumulation, multi-tile dimensional footprints,
/// and front-to-back (Reverse-Z) occlusion-culled blitting.
///
/// # Generics
/// * `TILE_SIZE` - Width and height of one standard tile in pixels.
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
    pub fn current_frame_coords(&self) -> Option<FrameCoord> {
        self.animation.get_frame(self.current_frame_index)
    }

    /// Returns the raw pixels of the root tile of the current frame (for 1x1 tile entities).
    #[inline(always)]
    pub fn current_frame_tile_pixels(&self) -> Option<&[Color]> {
        let frame = self.current_frame_coords()?;
        Some(self.animation.desc.tileset.get_tile(Point {
            x: frame.tx as u16,
            y: frame.ty as u16,
        }))
    }

    /// Access to internal animation metadata descriptor.
    #[inline(always)]
    pub fn desc(&self) -> &crate::texture::TextureDescriptor<'a> {
        &self.animation.desc
    }

    /// Changes the animation of the sprite. Resets the current frame to 0 and flags moved.
    #[inline(always)]
    pub fn set_animation(&mut self, animation: &'a Animation<'a>) {
        self.animation = animation;
        self.current_frame_index = 0;
        self.moved = true;
    }

    /// Returns whether this sprite's texture has transparency enabled.
    #[inline(always)]
    pub fn has_transparency(&self) -> bool {
        self.animation.desc.transparency
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
    pub(crate) fn commit_frame(&mut self) {
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
    /// Automatically flags `moved = true` when frames advance or integer pixels change.
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

    /// Blits a horizontal contiguous pixel span with optional Reverse-Z occlusion culling.
    #[inline(always)]
    fn blit_row_span(
        cell_buf: &mut [Color; CELL_AREA],
        covered: &mut [bool; CELL_AREA],
        pixels_covered: &mut usize,
        dst_start: usize,
        src: &[Color],
        len: usize,
        is_opaque: bool,
        fast_copy: bool,
    ) {
        if fast_copy {
            cell_buf[dst_start..dst_start + len].copy_from_slice(&src[..len]);
            return;
        }

        for i in 0..len {
            let dst_idx = dst_start + i;
            if !covered[dst_idx] {
                let pixel = src[i];
                if is_opaque || pixel.rgb565 != TRANSPARENCY_COLOR.rgb565 {
                    cell_buf[dst_idx] = pixel;
                    covered[dst_idx] = true;
                    *pixels_covered += 1;
                }
            }
        }
    }

    /// Rasterizes this sprite in Reverse-Z order (front-to-back):
    /// Reads multi-tile frames from flash, skips pixels already occluded in `covered`,
    /// writes newly visible pixels to `cell_buf`, and marks them in `covered`.
    pub(crate) fn blit_to_cell_reverse_z(
        &self,
        cell_buf: &mut [Color; CELL_AREA],
        covered: &mut [bool; CELL_AREA],
        pixels_covered: &mut usize,
        cell_pos: [i16; 2],
        screen_pos: [i16; 2],
    ) {
        let [cell_x, cell_y] = cell_pos;
        let [screen_x, screen_y] = screen_pos;

        let spr_w = self.pixel_width() as i16;
        let spr_h = self.pixel_height() as i16;

        let x_start = screen_x.max(cell_x);
        let x_end = (screen_x + spr_w).min(cell_x + TILE_SIZE as i16);
        let y_start = screen_y.max(cell_y);
        let y_end = (screen_y + spr_h).min(cell_y + TILE_SIZE as i16);

        if x_start >= x_end || y_start >= y_end {
            return;
        }

        let frame = match self.current_frame_coords() {
            Some(f) => f,
            None => return,
        };

        let desc = &self.animation.desc;
        let is_opaque = !desc.transparency;
        let tileset = desc.tileset;
        let tile_size = tileset.tile_size as usize;
        let sheet_w = desc.sheet_w.max(1) as usize;
        let sheet_h = desc.sheet_h.max(1) as usize;

        // Check if sprite fully covers this cell with opaque pixels and nothing drew above it
        let fast_full_cell = is_opaque
            && x_start == cell_x
            && y_start == cell_y
            && x_end == cell_x + TILE_SIZE as i16
            && y_end == cell_y + TILE_SIZE as i16
            && *pixels_covered == 0;

        let start_spr_y = (y_start - screen_y) as usize;
        let mut cur_ty = (start_spr_y / tile_size) % sheet_h;
        let mut row_in_t = start_spr_y % tile_size;

        let start_spr_x = (x_start - screen_x) as usize;
        let tx0 = (start_spr_x / tile_size) % sheet_w;
        let col0 = start_spr_x % tile_size;

        let total_x_len = (x_end - x_start) as usize;
        let span1_len = (tile_size - col0).min(total_x_len);
        let tx1 = (tx0 + 1) % sheet_w;
        let span2_len = total_x_len - span1_len;

        let cell_local_x0 = (x_start - cell_x) as usize;

        for cur_y in y_start..y_end {
            if *pixels_covered >= CELL_AREA {
                return;
            }

            let cell_local_y = (cur_y - cell_y) as usize;
            let row_dst = cell_local_y * TILE_SIZE + cell_local_x0;

            // 1. Fetch Left Tile and blit primary span
            let t_left = tileset.get_tile(Point {
                x: frame.tx as u16 + tx0 as u16,
                y: frame.ty as u16 + cur_ty as u16,
            });
            Self::blit_row_span(
                cell_buf,
                covered,
                pixels_covered,
                row_dst,
                &t_left[row_in_t * tile_size + col0..],
                span1_len,
                is_opaque,
                fast_full_cell,
            );

            // 2. Fetch Right Tile and blit secondary span (only if straddling across tile boundary)
            if span2_len > 0 {
                let t_right = tileset.get_tile(Point {
                    x: frame.tx as u16 + tx1 as u16,
                    y: frame.ty as u16 + cur_ty as u16,
                });
                Self::blit_row_span(
                    cell_buf,
                    covered,
                    pixels_covered,
                    row_dst + span1_len,
                    &t_right[row_in_t * tile_size..],
                    span2_len,
                    is_opaque,
                    fast_full_cell,
                );
            }

            // Advance vertical tile counters
            row_in_t += 1;
            if row_in_t == tile_size {
                row_in_t = 0;
                cur_ty += 1;
                if cur_ty == sheet_h {
                    cur_ty = 0;
                }
            }
        }

        if fast_full_cell {
            *pixels_covered = CELL_AREA;
        }
    }
}
