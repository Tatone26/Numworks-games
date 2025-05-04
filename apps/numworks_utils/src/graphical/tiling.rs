use core::{mem, slice};

use crate::{
    eadk::{display::push_rect, Color, Point, Rect},
    graphical::draw_image,
};

/// Transforms a &[u8] to &[Color]
/// Good luck with it
const fn as_color_slice(bytes: &'static [u8]) -> &'static [Color] {
    unsafe {
        slice::from_raw_parts(
            bytes.as_ptr() as *const Color,
            bytes.len() / mem::size_of::<Color>(),
        )
    }
}

/// A simple struct representing a Tileset.
/// Is just a constant (the size of a tile) and a static reference to the pixels
/// All tileset images should respect the following criterias :
///     Be on "column" form : all tiles should be on top of each other.
///     The column is as large as a single tile, not a pixel more, not a pixel less.
///     A ppm image can be processed by the build.rs file given in every apps I make.
///
pub struct Tileset {
    image: &'static [Color],
    pub tile_size: u16,
    width: u16,
}

impl Tileset {
    /// Creates a new Tileset with the tile size, the width in tiles (necessary to calculate offsets and all) and the image bytes.
    pub const fn new(tile_size: u16, width_in_tiles: u16, image: &'static [u8]) -> Self {
        Self {
            image: as_color_slice(image),
            tile_size,
            width: tile_size * width_in_tiles,
        }
    }

    /// Returns the slice associated with the data of the image.
    /// Because I don't care, the size associated with it is totally wrong. Use Tileset.tile_size instead.
    pub fn get_tile(
        &self,
        pos_in_tileset: Point, // as tiles
    ) -> &[Color] {
        let offset = (self.tile_size
            * (pos_in_tileset.y * self.width + pos_in_tileset.x * self.tile_size))
            as usize;
        &self.image[offset..]
    }

    #[inline(always)]
    /// Draws the given tile.
    /// Just a helper function, the logic being in draw_image anyway.
    pub fn draw_tile(&self, pos: Point, tile: Point, scaling: u16, transparency: bool) {
        draw_image(
            self.get_tile(tile),
            pos,
            (self.tile_size, self.tile_size),
            scaling,
            transparency,
        );
    }

    /// I needed this function to optimise a game, but put it here if I ever need it again.
    ///
    /// It allows tiling with only half of a tile, up or down, without scaling or transparency.
    ///
    /// Can be used to draw lines like some ground, and I reversed it because the rectangles on the right need to be drawn first (because of how the screen is refreshed)
    ///
    /// Will probably end up refactored with a draw_half_tile function.
    pub fn reverse_half_tiling(
        &self,
        pos: Point,
        dimensions: (u16, u16),
        tile: Point,
        bottom: bool,
    ) {
        let mut tile = self.get_tile(tile);
        if bottom {
            // offset the data by tile_size / 2 lines.
            tile = &tile[((self.tile_size * (self.tile_size / 2)) as usize)..];
        }
        for x in 0..dimensions.0 {
            let x_pos = x * self.tile_size + pos.x;
            for y in
                (pos.y..(dimensions.1 * self.tile_size + pos.y)).step_by(self.tile_size as usize)
            {
                push_rect(
                    Rect {
                        x: x_pos,
                        y,
                        width: self.tile_size,
                        height: self.tile_size / 2,
                    },
                    tile,
                );
            }
        }
    }

    /// Helper function to draw a tiling of the given tile. Will simply call draw_image for all the necessary tiles.
    /// WARNING : can go out of the screen !
    pub fn tiling(
        &self,
        pos: Point,
        dimensions: (u16, u16),
        tile: Point,
        transparency: bool,
        scaling: u16,
    ) {
        // some optimisation by skipping the overhead necessary to calculate the offset...
        // but really, it so small that it is not necessary.
        let tile: &[Color] = self.get_tile(tile);
        for x in 0..dimensions.0 {
            let x_pos = x * self.tile_size + pos.x;
            for y in 0..dimensions.1 {
                draw_image(
                    tile,
                    Point::new(x_pos, y * self.tile_size + pos.y),
                    (self.tile_size, self.tile_size),
                    scaling,
                    transparency,
                );
            }
        }
    }
}
