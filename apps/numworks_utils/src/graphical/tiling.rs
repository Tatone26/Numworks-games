use crate::{
    eadk::{display::push_rect_ptr, Color, Point, Rect},
    graphical::draw_image,
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
    pub image: &'static [u8],
    pub tile_size: u16,
    pub width: u16,
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
        // if no scaling and no transparency -> mega speed by giving directly the pointer on the pixels to the screen.
        if scaling == 1 && !transparency {
            debug_assert_eq!(
                (self.image.as_ptr() as usize) % 2,
                0,
                "Image data pointer is not properly aligned! Please use "
            );
            unsafe {
                push_rect_ptr(
                    Rect {
                        x: pos.x,
                        y: pos.y,
                        width: self.tile_size,
                        height: self.tile_size,
                    },
                    self.image
                        .as_ptr()
                        .add(
                            (2 * self.tile_size * (tile.y * self.width + tile.x * self.tile_size))
                                as usize,
                        )
                        .cast::<Color>(),
                );
            }
        } else {
            // Else go by all the draw_image process, which is far slower and needs to load a tile in ram.
            draw_image(
                &self.get_tile::<PIXELS>(tile),
                pos,
                (self.tile_size, self.tile_size),
                scaling,
                transparency,
            );
        }
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
        unsafe {
            let tile_ptr = self
                .image
                .as_ptr()
                .add(
                    (2 * self.tile_size
                        * ((tile.y + if bottom { self.tile_size / 2 } else { 0 }) * self.width
                            + tile.x * self.tile_size)) as usize,
                )
                .cast::<Color>();
            for x in 0..dimensions.0 {
                let x_pos = x * self.tile_size + pos.x;
                for y in (pos.y..(dimensions.1 * self.tile_size + pos.y))
                    .step_by(self.tile_size as usize)
                {
                    push_rect_ptr(
                        Rect {
                            x: x_pos,
                            y,
                            width: self.tile_size,
                            height: self.tile_size / 2,
                        },
                        tile_ptr,
                    );
                }
            }
        }
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
        if !transparency && scaling == 1 {
            unsafe {
                let tile_ptr = self
                    .image
                    .as_ptr()
                    .add(
                        (2 * self.tile_size * (tile.y * self.width + tile.x * self.tile_size))
                            as usize,
                    )
                    .cast::<Color>();
                for x in 0..dimensions.0 {
                    let x_pos = x * self.tile_size + pos.x;
                    for y in (pos.y..(dimensions.1 * self.tile_size + pos.y))
                        .step_by(self.tile_size as usize)
                    {
                        push_rect_ptr(
                            Rect {
                                x: x_pos,
                                y,
                                width: self.tile_size,
                                height: self.tile_size,
                            },
                            tile_ptr,
                        );
                    }
                }
            }
        } else {
            let tile: [Color; PIXELS] = self.get_tile(tile);
            for x in 0..dimensions.0 {
                let x_pos = x * self.tile_size + pos.x;
                for y in 0..dimensions.1 {
                    draw_image(
                        &tile,
                        Point::new(x_pos, y * self.tile_size + pos.y),
                        (self.tile_size, self.tile_size),
                        scaling,
                        transparency,
                    );
                }
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
        let mut image: [Color; PIXELS] = [Color::BLACK; PIXELS];
        let offset = (2
            * self.tile_size
            * (pos_in_tileset.y * self.width + pos_in_tileset.x * self.tile_size))
            as usize; // Offset represents the first pixel of this tile in the data.
        for (d, pixel) in image.iter_mut().enumerate() {
            *pixel = Color {
                rgb565: (self.image[offset + 2 * d + 1] as u16) << 8
                    | (self.image[offset + 2 * d] as u16),
            };
        }
        image
    }
}
