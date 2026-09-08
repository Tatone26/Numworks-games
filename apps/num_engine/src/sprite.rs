use numworks_utils::{
    eadk::{
        display::{SCREEN_HEIGHT, SCREEN_WIDTH},
        Color, Point,
    },
    graphical::TRANSPARENCY_COLOR,
};

use crate::texture::Animation;

pub struct Sprite<'a, const TILE_SIZE: usize, const CELL_AREA: usize> {
    pub animation: &'a mut Animation<'a>,
    pub position: Point,
    pub prev_position: Point,
    pub moved: bool,
    pub speed: [f32; 2],
    pub in_pixel_offset: [f32; 2],
    pub z: u8,
}

impl<'a, const TILE_SIZE: usize, const CELL_AREA: usize> Sprite<'a, TILE_SIZE, CELL_AREA> {
    pub fn new(animation: &'a mut Animation<'a>, position: Point, z: u8) -> Self {
        Self {
            animation,
            position,
            prev_position: position,
            moved: true,
            speed: [0.0, 0.0],
            in_pixel_offset: [0.0, 0.0],
            z,
        }
    }

    /// Advances animation and movement with boundary clamping.
    pub fn update(&mut self, frames_counter: u32) {
        self.moved = false;

        self.animation.update(frames_counter);
        if self.animation.changed {
            self.moved = true;
        }

        self.in_pixel_offset[0] += self.speed[0];
        self.in_pixel_offset[1] += self.speed[1];

        let delta_x = self.in_pixel_offset[0] as i32;
        let delta_y = self.in_pixel_offset[1] as i32;

        if delta_x != 0 {
            let max_x = (SCREEN_WIDTH as i32).saturating_sub(self.pixel_width() as i32);
            let new_x = ((self.position.x as i32) + delta_x).clamp(0, max_x.max(0));

            if new_x as u16 != self.position.x {
                self.position.x = new_x as u16;
                self.moved = true;
            }
            self.in_pixel_offset[0] -= delta_x as f32;
        }

        if delta_y != 0 {
            let max_y = (SCREEN_HEIGHT as i32).saturating_sub(self.pixel_height() as i32);
            let new_y = ((self.position.y as i32) + delta_y).clamp(0, max_y.max(0));

            if new_y as u16 != self.position.y {
                self.position.y = new_y as u16;
                self.moved = true;
            }
            self.in_pixel_offset[1] -= delta_y as f32;
        }
    }

    #[inline(always)]
    pub fn commit_frame(&mut self) {
        self.prev_position = self.position;
        self.moved = false;
    }

    #[inline(always)]
    pub fn pixel_width(&self) -> u16 {
        self.animation.desc.pixel_width()
    }

    #[inline(always)]
    pub fn pixel_height(&self) -> u16 {
        self.animation.desc.pixel_height()
    }

    pub fn set_position(&mut self, pos: Point, offset: Option<[f32; 2]>) {
        if self.position.x != pos.x || self.position.y != pos.y {
            self.position = pos;
            self.moved = true;
        }
        if let Some(off) = offset {
            self.in_pixel_offset = off;
        }
    }

    pub fn set_speed(&mut self, speed: [f32; 2]) {
        self.speed = speed;
    }

    /// Blits the clipped visible region of this sprite into a 16x16 cell buffer.
    pub fn blit_to_cell(&self, cell_buf: &mut [Color; CELL_AREA], cell_x: i16, cell_y: i16) {
        let spr_w = self.pixel_width() as i16;
        let spr_h = self.pixel_height() as i16;

        let rel_x = self.position.x as i16 - cell_x;
        let rel_y = self.position.y as i16 - cell_y;

        // Bounding box overlap inside sprite-local pixel coordinates
        let sx0 = if rel_x < 0 { -rel_x } else { 0 } as usize;
        let sy0 = if rel_y < 0 { -rel_y } else { 0 } as usize;
        let sx1 = if rel_x + spr_w > TILE_SIZE as i16 {
            (TILE_SIZE as i16 - rel_x).max(0)
        } else {
            spr_w
        } as usize;
        let sy1 = if rel_y + spr_h > TILE_SIZE as i16 {
            (TILE_SIZE as i16 - rel_y).max(0)
        } else {
            spr_h
        } as usize;

        if sx1 <= sx0 || sy1 <= sy0 {
            return;
        }

        let frame = match self.animation.current_frame() {
            Some(f) => f,
            None => return,
        };

        let desc = &self.animation.desc;
        let tileset = desc.tileset;
        let tile_size = tileset.tile_size as usize;

        let min_tx = sx0 / tile_size;
        let max_tx = (sx1 - 1) / tile_size;
        let min_ty = sy0 / tile_size;
        let max_ty = (sy1 - 1) / tile_size;

        for ty in min_ty..=max_ty {
            let tile_top_y = ty * tile_size;
            let local_y0 = sy0.saturating_sub(tile_top_y).min(tile_size);
            let local_y1 = sy1.saturating_sub(tile_top_y).min(tile_size);

            if local_y1 <= local_y0 {
                continue;
            }

            for tx in min_tx..=max_tx {
                let tile_left_x = tx * tile_size;
                let local_x0 = sx0.saturating_sub(tile_left_x).min(tile_size);
                let local_x1 = sx1.saturating_sub(tile_left_x).min(tile_size);

                if local_x1 <= local_x0 {
                    continue;
                }
                let blit_w = local_x1 - local_x0;

                let tile_slice = tileset.get_tile(Point {
                    x: frame.tx as u16 + tx as u16,
                    y: frame.ty as u16 + ty as u16,
                });

                for row in local_y0..local_y1 {
                    let dst_y = (rel_y + (tile_top_y + row) as i16) as usize;
                    let dst_x = (rel_x + (tile_left_x + local_x0) as i16) as usize;

                    let dst_idx = dst_y * TILE_SIZE + dst_x;
                    let src_idx = row * tile_size + local_x0;

                    blit_row_span(
                        &mut cell_buf[dst_idx..],
                        &tile_slice[src_idx..],
                        blit_w,
                        desc.transparency,
                    );
                }
            }
        }
    }
}

/// Blits a horizontal contiguous pixel span with optional color-key transparency.
#[inline(always)]
fn blit_row_span(dst: &mut [Color], src: &[Color], width: usize, has_transparency: bool) {
    if !has_transparency {
        dst[..width].copy_from_slice(&src[..width]);
        return;
    }

    let d = &mut dst[..width];
    let s = &src[..width];
    for i in 0..width {
        let p = s[i];
        if p.rgb565 != TRANSPARENCY_COLOR.rgb565 {
            d[i] = p;
        }
    }
}
