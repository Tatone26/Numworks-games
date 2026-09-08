use crate::dirty_grid::DirtyGrid;
use crate::sprite::Sprite;
use crate::tilemap::Tilemap;
use numworks_utils::eadk::{display, Color, Rect};

pub enum Renderable<
    'r,
    'a,
    const TILE_SIZE: usize,
    const CELL_AREA: usize,
    const COLS: usize,
    const ROWS: usize,
> {
    Tilemap(&'r Tilemap<'a, TILE_SIZE, CELL_AREA, COLS, ROWS>),
    Sprite(&'r Sprite<'a, TILE_SIZE, CELL_AREA>),
}

impl<
        'r,
        'a,
        const TILE_SIZE: usize,
        const CELL_AREA: usize,
        const COLS: usize,
        const ROWS: usize,
    > Renderable<'r, 'a, TILE_SIZE, CELL_AREA, COLS, ROWS>
{
    #[inline(always)]
    pub fn z(&self) -> u8 {
        match self {
            Renderable::Tilemap(tm) => tm.z,
            Renderable::Sprite(s) => s.z,
        }
    }
}

pub struct Compositor<
    const TILE_SIZE: usize,
    const CELL_AREA: usize,
    const COLS: usize,
    const ROWS: usize,
> {
    pub grid: DirtyGrid<TILE_SIZE, COLS, ROWS>,
    pub clear_color: Color,
}

impl<const TILE_SIZE: usize, const CELL_AREA: usize, const COLS: usize, const ROWS: usize>
    Compositor<TILE_SIZE, CELL_AREA, COLS, ROWS>
{
    pub fn new(clear_color: Color) -> Self {
        let mut comp = Self {
            grid: DirtyGrid::new(),
            clear_color,
        };
        comp.grid.mark_all();
        comp
    }

    pub fn render<'a>(
        &mut self,
        items: &mut [Renderable<'_, 'a, TILE_SIZE, CELL_AREA, COLS, ROWS>],
    ) {
        let mut cell_buffer = [self.clear_color; CELL_AREA];

        // 1. Sort all layers and sprites together by Z depth (lowest to highest)
        items.sort_unstable_by_key(|item| item.z());

        // 2. Outer: Right to Left (Columns)
        for cx in (0..COLS).rev() {
            let col_mask = 1u32 << cx;
            let cell_x = (cx * TILE_SIZE) as i16;
            let cell_x1 = cell_x + TILE_SIZE as i16 - 1;

            // Inner: Bottom to Top (Rows)
            for cy in (0..ROWS).rev() {
                // Skip unchanged cells
                if ((self.grid.curr[cy] | self.grid.prev[cy]) & col_mask) == 0 {
                    continue;
                }

                let cell_y = (cy * TILE_SIZE) as i16;
                let cell_y1 = cell_y + TILE_SIZE as i16 - 1;

                // Reset the cell to the base clear color
                cell_buffer.fill(self.clear_color);

                // 3. Composite all elements overlapping this cell in ascending Z order
                for item in items.iter() {
                    match item {
                        Renderable::Tilemap(tm) => {
                            // Blit the tile if set, leaves previous contents if None
                            tm.blit_to_cell(cx, cy, &mut cell_buffer);
                        }
                        Renderable::Sprite(s) => {
                            let sx0 = s.position.x as i16;
                            let sy0 = s.position.y as i16;
                            let sx1 = sx0 + s.pixel_width() as i16 - 1;
                            let sy1 = sy0 + s.pixel_height() as i16 - 1;

                            // AABB intersection check
                            if !(sx0 > cell_x1 || sx1 < cell_x || sy0 > cell_y1 || sy1 < cell_y) {
                                s.blit_to_cell(&mut cell_buffer, cell_x, cell_y);
                            }
                        }
                    }
                }

                // 4. Push final composited cell to screen
                let dest_rect = Rect {
                    x: cell_x as u16,
                    y: cell_y as u16,
                    width: TILE_SIZE as u16,
                    height: TILE_SIZE as u16,
                };
                display::push_rect(dest_rect, &cell_buffer);
            }
        }

        self.grid.swap_and_clear();
    }
}
