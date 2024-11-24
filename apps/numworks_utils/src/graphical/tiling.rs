use crate::{
    eadk::{Color, Point},
    graphical::{draw_image, TRANSPARENCY_COLOR},
};

/// A simple struct representing a Tileset.
/// Is just a constant (the size of a tile) and a static reference to the pixels
/// All tileset images should respect the following criterias :
///     Be on "column" form : all tiles should be on top of each other.
///     The column is as large as a single tile, not a pixel more, not a pixel less.
///     A ppm image can be processed by the build.rs file given in every apps I make.
///
///     NOT DONE FOR NOW
pub struct Tileset {
    pub tile_size: u16,
    pub width: u16,
    pub image: &'static [u8],
}

impl Tileset {
    /// Draws an image of width*height pixels (can be scaled) from a given tileset and its position on this tileset.
    pub fn draw_tile<const PIXELS: usize>(
        &self,
        pos: Point,
        tile: Point,
        scaling: u16,
        transparency: bool,
    ) {
        let image: [Color; PIXELS] = self.get_tile::<PIXELS>(tile);
        draw_image(
            &image,
            pos,
            (self.tile_size, self.tile_size),
            scaling,
            transparency,
        );
    }

    /// For really fast tiling : use scaling = 1 and transparency = false.
    /// In other case, every pixel will be drawn after the other, with a dedicated Rect (= far slower than a single Rect)
    pub fn tiling<const PIXELS: usize>(
        &self,
        pos: Point,
        dimensions: (u16, u16),
        // The position (in tiles) of the tile in the tilemap.
        tile: Point,
        transparency: bool,
        scaling: u16,
    ) {
        let image: [Color; PIXELS] = self.get_tile::<PIXELS>(tile);
        for x in 0..dimensions.0 {
            for y in 0..dimensions.1 {
                draw_image(
                    &image,
                    Point::new(x * self.tile_size + pos.x, y * self.tile_size + pos.y),
                    (self.tile_size, self.tile_size),
                    scaling,
                    transparency,
                );
            }
        }
    }

    /// Stores the pixels of a tile in RAM. Should be used rarely, and never for big images.
    /// [draw_image] is better in this case (does not load anything in RAM)
    /// To use transparency : use draw_image with the image returned by this function.
    pub fn get_tile<const PIXELS: usize>(
        &self,
        pos_in_tileset: Point, // as tiles
    ) -> [Color; PIXELS] {
        let mut image: [Color; PIXELS] = [TRANSPARENCY_COLOR; PIXELS];
        let mut x_pos = 0;
        let mut y_pos = 0;
        for pixel in image
            .iter_mut()
            .take((self.tile_size * self.tile_size) as usize)
        {
            let i = 2
                * (pos_in_tileset.x * self.tile_size
                    + x_pos
                    + (y_pos + pos_in_tileset.y * self.tile_size) * self.width)
                    as usize;
            *pixel = Color {
                rgb565: (self.image[i] as u16) << 8 | (self.image[i + 1] as u16),
            };
            x_pos += 1;
            if x_pos >= self.tile_size {
                x_pos = 0;
                y_pos += 1;
                if y_pos >= self.tile_size {
                    break;
                }
            }
        }
        image
    }
}
