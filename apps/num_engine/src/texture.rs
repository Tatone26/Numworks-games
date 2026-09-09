//! Textures, animation sequences, and frame coordinate definitions.

use numworks_utils::{eadk::Point, graphical::tiling::Tileset};

/// Coordinates of a single tile within a [`Tileset`].
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct FrameCoord {
    /// Column index of the tile on the tileset grid.
    pub tx: u8,
    /// Row index of the tile on the tileset grid.
    pub ty: u8,
}

/// Shared layout descriptor holding dimension, scaling, and color-keying rules for an [`Animation`].
///
/// Supports repeating tile patterns: if `width > sheet_w` or `height > sheet_h`, the source
/// tile block defined by `(sheet_w, sheet_h)` repeats across the full `(width, height)` footprint.
#[derive(Clone, Copy)]
pub struct TextureDescriptor<'a> {
    /// Reference to the underlying tileset containing raw pixel buffers.
    pub tileset: &'a Tileset,
    /// Total rendered footprint width expressed in tiles.
    pub width: u8,
    /// Total rendered footprint height expressed in tiles.
    pub height: u8,
    /// Source block width in tiles within the tileset sheet (repeats across `width`).
    pub sheet_w: u8,
    /// Source block height in tiles within the tileset sheet (repeats across `height`).
    pub sheet_h: u8,
    /// Multiplier applied to each tile dimension.
    pub scaling: u16,
    /// Whether pixels matching [`TRANSPARENCY_COLOR`] are keyed out during rasterization.
    pub transparency: bool,
}

impl<'a> TextureDescriptor<'a> {
    /// Constructs a standard (non-repeating) texture layout descriptor.
    ///
    /// Source tile size in the sheet matches the rendered width and height (`sheet_w = width`, `sheet_h = height`).
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
            sheet_w: width,
            sheet_h: height,
            scaling,
            transparency,
        }
    }

    /// Constructs a texture layout descriptor where a smaller `(sheet_w, sheet_h)` tile pattern
    /// repeats to fill a larger `(width, height)` footprint.
    pub const fn new_repeating(
        tileset: &'a Tileset,
        width: u8,
        height: u8,
        sheet_w: u8,
        sheet_h: u8,
        scaling: u16,
        transparency: bool,
    ) -> Self {
        Self {
            tileset,
            width,
            height,
            sheet_w: if sheet_w == 0 { 1 } else { sheet_w },
            sheet_h: if sheet_h == 0 { 1 } else { sheet_h },
            scaling,
            transparency,
        }
    }

    /// Returns the resolved pixel width: `width * tile_size * scaling`.
    #[inline(always)]
    pub fn pixel_width(&self) -> u16 {
        (self.width as u16) * self.tileset.tile_size * self.scaling
    }

    /// Returns the resolved pixel height: `height * tile_size * scaling`.
    #[inline(always)]
    pub fn pixel_height(&self) -> u16 {
        (self.height as u16) * self.tileset.tile_size * self.scaling
    }
}

/// Immutable animation sequence composed of sequential [`FrameCoord`] steps.
///
/// Safe to share among multiple [`Sprite`](crate::sprite::Sprite) instances via shared reference.
pub struct Animation<'a> {
    /// Dimensional and tileset metadata.
    pub desc: TextureDescriptor<'a>,
    /// Frame coordinates list stored in Flash (ROM).
    pub frames: &'static [FrameCoord],
    /// Engine ticks between frame increments (minimum is 1).
    pub speed: u8,
}

impl<'a> Animation<'a> {
    /// Creates a new animation sequence, clamping `speed` to a minimum of 1.
    pub const fn new(
        desc: TextureDescriptor<'a>,
        frames: &'static [FrameCoord],
        speed: u8,
    ) -> Self {
        Self {
            desc,
            frames,
            speed: if speed == 0 { 1 } else { speed },
        }
    }

    /// Retrieves the tile coordinate for a specific frame index, or `None` if out of bounds.
    #[inline(always)]
    pub fn get_frame(&self, frame_index: usize) -> Option<FrameCoord> {
        if self.frames.is_empty() {
            None
        } else {
            Some(self.frames[frame_index % self.frames.len()])
        }
    }

    /// Computes the frame index for a given monotonic frame counter.
    #[inline(always)]
    pub(crate) fn calculate_frame_index(&self, frames_counter: u32) -> usize {
        if self.frames.is_empty() {
            return 0;
        }
        let step = frames_counter / (self.speed as u32);
        (step as usize) % self.frames.len()
    }

    /// Draws the animation frame directly to the display glass via EADK syscalls.
    ///
    /// Bypasses the Compositor dirty grid; ideal for static HUD elements, title screens, or menus.
    pub fn draw_at(&self, pos: Point, frame_index: usize) {
        if let Some(coord) = self.get_frame(frame_index) {
            let step = self.desc.tileset.tile_size * self.desc.scaling;
            for ty in 0..(self.desc.height as u16) {
                let sheet_ty = ty % (self.desc.sheet_h as u16);
                for tx in 0..(self.desc.width as u16) {
                    let sheet_tx = tx % (self.desc.sheet_w as u16);
                    self.desc.tileset.draw_tile(
                        Point {
                            x: (pos.x as i16 + (tx * step) as i16).max(0) as u16,
                            y: (pos.y as i16 + (ty * step) as i16).max(0) as u16,
                        },
                        Point {
                            x: ((coord.tx as u16 + sheet_tx) as i16).max(0) as u16,
                            y: ((coord.ty as u16 + sheet_ty) as i16).max(0) as u16,
                        },
                        self.desc.scaling,
                        self.desc.transparency,
                    );
                }
            }
        }
    }
}
