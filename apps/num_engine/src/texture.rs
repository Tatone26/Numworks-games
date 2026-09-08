use heapless::Vec;
use numworks_utils::{eadk::Point, graphical::tiling::Tileset};

pub const MAX_LENGTH_ANIMATION: usize = 8;

/// Inside-tileset position
#[derive(Clone, Copy, Default)]
pub struct FrameCoord {
    pub tx: u8,
    pub ty: u8,
}

/// Shared layout descriptor: holds everything common across all frames
pub struct TextureDescriptor<'a> {
    pub tileset: &'a Tileset,
    pub width: u8,  // in tiles
    pub height: u8, // in tiles
    pub scaling: u16,
    pub transparency: bool,
}

impl<'a> TextureDescriptor<'a> {
    pub const fn new(
        tileset: &'a Tileset,
        width: u8,
        height: u8,
        scaling: u16,
        transparency: bool,
    ) -> Self {
        Self {
            tileset,
            width,
            height,
            scaling,
            transparency,
        }
    }

    #[inline(always)]
    pub fn pixel_width(&self) -> u16 {
        (self.width as u16) * self.tileset.tile_size * self.scaling
    }

    #[inline(always)]
    pub fn pixel_height(&self) -> u16 {
        (self.height as u16) * self.tileset.tile_size * self.scaling
    }
}

/// Represent the full sprite texture/animation object.
/// Can have only 1 frame for static texture.
/// use frames.len() to have that info
pub struct Animation<'a> {
    pub desc: TextureDescriptor<'a>,
    pub frames: Vec<FrameCoord, MAX_LENGTH_ANIMATION>,
    pub current_frame_index: usize,
    pub speed: u8, // how many screen frame between each animation frame. Min is 1.
    pub changed: bool,
}

impl<'a> Animation<'a> {
    pub fn new(
        desc: TextureDescriptor<'a>,
        frames: Vec<FrameCoord, MAX_LENGTH_ANIMATION>,
        speed: u8,
    ) -> Self {
        Self {
            desc,
            frames,
            current_frame_index: 0,
            speed: speed.max(1),
            changed: true,
        }
    }

    #[inline(always)]
    pub fn current_frame(&self) -> Option<FrameCoord> {
        self.frames.get(self.current_frame_index).copied()
    }

    /// Call this every frame.
    pub fn update(&mut self, frames_counter: u32) {
        if self.frames.is_empty() {
            return;
        }
        let step = frames_counter / (self.speed as u32);
        let next_idx = (step as usize) % self.frames.len();

        self.changed = next_idx != self.current_frame_index;
        self.current_frame_index = next_idx;
    }

    /// Immediate full draw (if used for menus, static HUD, etc.)
    pub fn draw_at(&self, pos: Point) {
        if let Some(coord) = self.current_frame() {
            let step = self.desc.tileset.tile_size * self.desc.scaling;
            for ty in 0..(self.desc.height as u16) {
                for tx in 0..(self.desc.width as u16) {
                    self.desc.tileset.draw_tile(
                        Point {
                            x: (pos.x as i16 + (tx * step) as i16).max(0) as u16,
                            y: (pos.y as i16 + (ty * step) as i16).max(0) as u16,
                        },
                        Point {
                            x: ((coord.tx as u16 + tx) as i16).max(0) as u16,
                            y: ((coord.ty as u16 + ty) as i16).max(0) as u16,
                        },
                        self.desc.scaling,
                        self.desc.transparency,
                    );
                }
            }
        }
    }
}
