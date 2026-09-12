//! Pure visual animated sprite handles and reverse-Z cell rasterization.

use crate::graphics::texture::{Animation, FrameCoord};
use numworks_utils::{
    eadk::{Color, Point},
    graphical::TRANSPARENCY_COLOR,
};

pub struct Sprite<'a, const TILE_SIZE: usize, const CELL_AREA: usize> {
    pub animation: &'a Animation<'a>,
    pub current_frame_index: usize,
    /// Absolute position [x, y] in integer world coordinates.
    pub position: [i16; 2],
    /// Local displacement relative to an entity's parent Body anchor.
    pub offset: [i16; 2],
    pub(crate) prev_position: [i16; 2],
    pub(crate) moved: bool,
    pub z: u8,
    pub active: bool,
}

impl<'a, const TILE_SIZE: usize, const CELL_AREA: usize> Sprite<'a, TILE_SIZE, CELL_AREA> {
    pub const fn new(animation: &'a Animation<'a>, position: [i16; 2], z: u8) -> Self {
        Self {
            animation,
            current_frame_index: 0,
            position,
            offset: [0, 0],
            prev_position: position,
            moved: true,
            z,
            active: true,
        }
    }

    pub const fn with_offset(mut self, offset: [i16; 2]) -> Self {
        self.offset = offset;
        self
    }

    pub const fn empty(fallback_anim: &'a Animation<'a>) -> Self {
        Self {
            animation: fallback_anim,
            current_frame_index: 0,
            position: [0, 0],
            offset: [0, 0],
            prev_position: [0, 0],
            moved: true,
            z: 0,
            active: false,
        }
    }

    /// Sets the absolute anchor, applying internal relative offset.
    #[inline(always)]
    pub fn sync_to_anchor(&mut self, anchor: [i16; 2]) {
        let new_pos = [anchor[0] + self.offset[0], anchor[1] + self.offset[1]];
        if self.position != new_pos {
            self.position = new_pos;
            self.moved = true;
        }
    }

    #[inline(always)]
    pub fn set_position(&mut self, pos: [i16; 2]) {
        if self.position != pos {
            self.position = pos;
            self.moved = true;
        }
    }

    #[inline(always)]
    pub fn set_animation(&mut self, animation: &'a Animation<'a>) {
        self.animation = animation;
        self.current_frame_index = 0;
        self.moved = true;
    }

    #[inline(always)]
    pub fn update_animation(&mut self, frame_counter: u32) {
        let next_idx = self.animation.calculate_frame_index(frame_counter);
        if next_idx != self.current_frame_index {
            self.current_frame_index = next_idx;
            self.moved = true;
        }
    }

    #[inline(always)]
    pub fn pixel_width(&self) -> u16 {
        self.animation.desc.pixel_width()
    }

    #[inline(always)]
    pub fn pixel_height(&self) -> u16 {
        self.animation.desc.pixel_height()
    }

    #[inline(always)]
    pub(crate) fn commit_frame(&mut self) {
        self.prev_position = self.position;
        self.moved = false;
    }

    #[inline(always)]
    pub fn current_frame_coords(&self) -> Option<FrameCoord> {
        self.animation.get_frame(self.current_frame_index)
    }

    #[allow(clippy::too_many_arguments)]
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

        for (i, pixel) in src.iter().enumerate().take(len) {
            let dst_idx = dst_start + i;
            if !covered[dst_idx] && (is_opaque || pixel.rgb565 != TRANSPARENCY_COLOR.rgb565) {
                cell_buf[dst_idx] = *pixel;
                covered[dst_idx] = true;
                *pixels_covered += 1;
            }
        }
    }

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

        // Yes, this following code packet seems catastrophic. But believe me when I say it works,
        // and it is fast. The Numworks processor really is really strong.
        // Especially when all of this is optimized by the compiler anyway...

        let desc = &self.animation.desc;
        let is_opaque = !desc.transparency;
        let tileset = desc.tileset;
        let tile_size = tileset.tile_size as usize;
        let sheet_w = desc.sheet_w.max(1) as usize;
        let sheet_h = desc.sheet_h.max(1) as usize;

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
