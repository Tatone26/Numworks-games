//! Low-level rasterizer, layer sorter, and hardware push batcher.

use crate::dirty_grid::DirtyGrid;
use crate::sprite::Sprite;
use crate::tilemap::Tilemap;
use crate::viewport::Viewport;
use numworks_utils::eadk::{display, Color, Rect};

/// Interlacing patterns used to drop 50% of cell work during heavy camera scroll.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum InterlaceMode {
    /// Render every cell every frame. Full quality.
    #[default]
    None,
    /// Alternate odd and even columns each frame (retains fast continuous SPI bursts).
    Columns,
    /// Alternate odd and even rows each frame.
    Rows,
    /// Alternating 2D checkerboard pattern (`(cx + cy + phase) & 1 == 0`).
    Checkerboard,
}

/// A renderable object variant submitted to the engine render list.
#[derive(Clone, Copy)]
pub enum Renderable<'r, 'a, const TILE_SIZE: usize, const CELL_AREA: usize> {
    Tilemap(&'r Tilemap<'a, TILE_SIZE, CELL_AREA>),
    Sprite(&'r Sprite<'a, TILE_SIZE, CELL_AREA>),
    UiSprite(&'r Sprite<'a, TILE_SIZE, CELL_AREA>),
}

impl<'r, 'a, const TILE_SIZE: usize, const CELL_AREA: usize>
    Renderable<'r, 'a, TILE_SIZE, CELL_AREA>
{
    #[inline(always)]
    pub fn z(&self) -> u8 {
        match self {
            Renderable::Tilemap(tm) => tm.z,
            Renderable::Sprite(s) => s.z,
            Renderable::UiSprite(s) => s.z,
        }
    }
}

pub(crate) struct Compositor<
    const TILE_SIZE: usize,
    const CELL_AREA: usize,
    const SCREEN_COLS: usize,
    const SCREEN_ROWS: usize,
> {
    pub(crate) grid: DirtyGrid<TILE_SIZE, SCREEN_COLS, SCREEN_ROWS>,
    clear_color: Color,
    frame: u32,
}

impl<
        const TILE_SIZE: usize,
        const CELL_AREA: usize,
        const SCREEN_COLS: usize,
        const SCREEN_ROWS: usize,
    > Compositor<TILE_SIZE, CELL_AREA, SCREEN_COLS, SCREEN_ROWS>
{
    pub fn new(clear_color: Color) -> Self {
        let mut comp = Self {
            grid: DirtyGrid::new(),
            clear_color,
            frame: 0,
        };
        comp.grid.mark_all();
        comp
    }

    #[inline(always)]
    pub fn set_frame(&mut self, frame: u32) {
        self.frame = frame;
    }

    /// Sorts renderables by Z-depth and executes the optimal hardware flush path
    /// based on camera motion and the active interlacing pattern.
    pub fn render<'a>(
        &mut self,
        items: &mut [Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
        viewport: &Viewport,
        scratch: &mut [Color],
        viewport_moved: bool,
        interlace: InterlaceMode,
    ) {
        items.sort_unstable_by_key(|item| item.z());

        let phase = (self.frame & 1) as usize;

        match interlace {
            // Uninterlaced: stationary uses discrete cells; moving uses tall column bursts
            InterlaceMode::None => {
                if !viewport_moved {
                    self.render_discrete(items, viewport, scratch, None);
                } else {
                    self.render_columns(items, viewport, scratch, None);
                }
            }
            // Columns interlacing: keep tall vertical bursts, dropping alternate columns
            InterlaceMode::Columns => {
                self.render_columns(items, viewport, scratch, Some(phase));
            }
            // Rows interlacing: alternate odd/even horizontal bands via discrete cells
            InterlaceMode::Rows => {
                self.render_discrete(items, viewport, scratch, Some(FilterKind::Rows(phase)));
            }
            // Checkerboard interlacing: alternate (cx + cy) checkerboard parity
            InterlaceMode::Checkerboard => {
                self.render_discrete(
                    items,
                    viewport,
                    scratch,
                    Some(FilterKind::Checkerboard(phase)),
                );
            }
        }

        self.grid.swap_and_clear();
    }

    /// Pushes single cells independently with optional row/checkerboard filtering.
    fn render_discrete<'a>(
        &mut self,
        items: &[Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
        viewport: &Viewport,
        scratch: &mut [Color],
        filter: Option<FilterKind>,
    ) {
        let cell_buf: &mut [Color; CELL_AREA] =
            unsafe { &mut *(scratch.as_mut_ptr() as *mut [Color; CELL_AREA]) };

        for cx in (0..SCREEN_COLS).rev() {
            let col_mask = 1u32 << cx;
            let cell_x = viewport.screen_x as i16 + (cx * TILE_SIZE) as i16;
            let cell_x1 = cell_x + TILE_SIZE as i16 - 1;

            for cy in (0..SCREEN_ROWS).rev() {
                // Apply interlacing filter before checking dirty mask
                if let Some(f) = filter {
                    if !f.should_draw(cx, cy) {
                        continue;
                    }
                }

                if ((self.grid.curr[cy] | self.grid.prev[cy]) & col_mask) == 0 {
                    continue;
                }

                let cell_y = viewport.screen_y as i16 + (cy * TILE_SIZE) as i16;
                let cell_y1 = cell_y + TILE_SIZE as i16 - 1;

                self.composite_cell(
                    cx, cy, cell_x, cell_y, cell_x1, cell_y1, items, viewport, cell_buf,
                );

                display::push_rect(
                    Rect {
                        x: cell_x as u16,
                        y: cell_y as u16,
                        width: TILE_SIZE as u16,
                        height: TILE_SIZE as u16,
                    },
                    cell_buf,
                );
            }
        }
    }

    /// Fuses vertical runs of dirty cells in each column into single continuous bursts.
    fn render_columns<'a>(
        &mut self,
        items: &[Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
        viewport: &Viewport,
        scratch: &mut [Color],
        column_filter: Option<usize>,
    ) {
        let (cell_part, col_part) = scratch.split_at_mut(CELL_AREA);
        let cell_buf: &mut [Color; CELL_AREA] =
            unsafe { &mut *(cell_part.as_mut_ptr() as *mut [Color; CELL_AREA]) };

        for cx in (0..SCREEN_COLS).rev() {
            if let Some(phase) = column_filter {
                if (cx & 1) != phase {
                    continue;
                }
            }

            let col_mask = 1u32 << cx;
            let cell_x = viewport.screen_x as i16 + (cx * TILE_SIZE) as i16;
            let cell_x1 = cell_x + TILE_SIZE as i16 - 1;

            let mut y = 0;
            while y < SCREEN_ROWS {
                while y < SCREEN_ROWS && ((self.grid.curr[y] | self.grid.prev[y]) & col_mask) == 0 {
                    y += 1;
                }
                if y >= SCREEN_ROWS {
                    break;
                }

                let span_start = y;
                while y < SCREEN_ROWS && ((self.grid.curr[y] | self.grid.prev[y]) & col_mask) != 0 {
                    y += 1;
                }
                let span_end = y;
                let span_rows = span_end - span_start;

                if span_rows == 1 {
                    let cell_y = viewport.screen_y as i16 + (span_start * TILE_SIZE) as i16;
                    self.composite_cell(
                        cx,
                        span_start,
                        cell_x,
                        cell_y,
                        cell_x1,
                        cell_y + TILE_SIZE as i16 - 1,
                        items,
                        viewport,
                        cell_buf,
                    );
                    display::push_rect(
                        Rect {
                            x: cell_x as u16,
                            y: cell_y as u16,
                            width: TILE_SIZE as u16,
                            height: TILE_SIZE as u16,
                        },
                        cell_buf,
                    );
                } else {
                    for (local_idx, cy) in (span_start..span_end).enumerate() {
                        let cell_y = viewport.screen_y as i16 + (cy * TILE_SIZE) as i16;
                        self.composite_cell(
                            cx,
                            cy,
                            cell_x,
                            cell_y,
                            cell_x1,
                            cell_y + TILE_SIZE as i16 - 1,
                            items,
                            viewport,
                            cell_buf,
                        );

                        let dst_start = local_idx * CELL_AREA;
                        col_part[dst_start..dst_start + CELL_AREA].copy_from_slice(cell_buf);
                    }

                    display::push_rect(
                        Rect {
                            x: cell_x as u16,
                            y: (viewport.screen_y as i16 + (span_start * TILE_SIZE) as i16) as u16,
                            width: TILE_SIZE as u16,
                            height: (span_rows * TILE_SIZE) as u16,
                        },
                        &col_part[..span_rows * CELL_AREA],
                    );
                }
            }
        }
    }

    #[inline(always)]
    fn composite_cell<'a>(
        &self,
        cx: usize,
        cy: usize,
        cell_x: i16,
        cell_y: i16,
        cell_x1: i16,
        cell_y1: i16,
        items: &[Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
        viewport: &Viewport,
        buffer: &mut [Color; CELL_AREA],
    ) {
        let needs_clear = match items.first() {
            Some(Renderable::Tilemap(tm)) => tm.transparent,
            _ => true,
        };

        if needs_clear {
            buffer.fill(self.clear_color);
        }

        for item in items.iter() {
            match item {
                Renderable::Tilemap(tm) => {
                    tm.blit_to_cell(cx, cy, viewport.x, viewport.y, buffer, self.frame);
                }
                Renderable::Sprite(s) => {
                    let (sx0, sy0) =
                        viewport.world_to_screen(s.position[0] as i32, s.position[1] as i32);
                    let sx1 = sx0 + s.pixel_width() as i16 - 1;
                    let sy1 = sy0 + s.pixel_height() as i16 - 1;

                    if !(sx0 > cell_x1 || sx1 < cell_x || sy0 > cell_y1 || sy1 < cell_y) {
                        s.blit_to_cell_at(buffer, cell_x, cell_y, sx0, sy0);
                    }
                }
                Renderable::UiSprite(s) => {
                    let sx0 = s.position[0];
                    let sy0 = s.position[1];
                    let sx1 = sx0 + s.pixel_width() as i16 - 1;
                    let sy1 = sy0 + s.pixel_height() as i16 - 1;

                    if !(sx0 > cell_x1 || sx1 < cell_x || sy0 > cell_y1 || sy1 < cell_y) {
                        s.blit_to_cell_at(buffer, cell_x, cell_y, sx0, sy0);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum FilterKind {
    Rows(usize),
    Checkerboard(usize),
}

impl FilterKind {
    #[inline(always)]
    fn should_draw(&self, cx: usize, cy: usize) -> bool {
        match *self {
            FilterKind::Rows(phase) => (cy & 1) == phase,
            FilterKind::Checkerboard(phase) => ((cx + cy) & 1) == phase,
        }
    }
}
