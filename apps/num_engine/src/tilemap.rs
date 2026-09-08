use numworks_utils::{
    eadk::{Color, Point},
    graphical::{tiling::Tileset, TRANSPARENCY_COLOR},
};

pub struct Tilemap<
    'a,
    const TILE_SIZE: usize,
    const CELL_AREA: usize,
    const COLS: usize,
    const ROWS: usize,
> {
    pub tileset: &'a Tileset,
    pub tilemap: [[Option<[u8; 2]>; ROWS]; COLS],
    pub z: u8,
    pub transparent_key: bool, // If true, skip TRANSPARENCY_COLOR pixels
}

impl<'a, const TILE_SIZE: usize, const CELL_AREA: usize, const COLS: usize, const ROWS: usize>
    Tilemap<'a, TILE_SIZE, CELL_AREA, COLS, ROWS>
{
    pub fn new(
        tileset: &'a Tileset,
        tilemap: [[Option<[u8; 2]>; ROWS]; COLS],
        z: u8,
        transparent_key: bool,
    ) -> Self {
        Self {
            tileset,
            tilemap,
            z,
            transparent_key,
        }
    }

    #[inline(always)]
    pub fn get_tile(&self, cx: usize, cy: usize) -> Option<[u8; 2]> {
        if cx < COLS && cy < ROWS {
            self.tilemap[cx][cy]
        } else {
            None
        }
    }

    /// Blits the tile at (cx, cy) into `cell_buffer`.
    /// - If None: does nothing at all (leaves whatever is below untouched).
    /// - If transparent_key is true: skips pixels equal to TRANSPARENCY_COLOR.
    /// - If transparent_key is false: directly copies all pixels (opaque).
    pub fn blit_to_cell(&self, cx: usize, cy: usize, buffer: &mut [Color; CELL_AREA]) {
        let Some([tx, ty]) = self.get_tile(cx, cy) else {
            return; // Fully transparent cell: do not touch the buffer!
        };

        let tile_slice = self.tileset.get_tile(Point {
            x: tx as u16,
            y: ty as u16,
        });

        let len = CELL_AREA.min(tile_slice.len());

        if self.transparent_key {
            for i in 0..len {
                let pixel = tile_slice[i];
                if pixel.rgb565 != TRANSPARENCY_COLOR.rgb565 {
                    buffer[i] = pixel;
                }
            }
        } else {
            buffer[..len].copy_from_slice(&tile_slice[..len]);
        }
    }
}
