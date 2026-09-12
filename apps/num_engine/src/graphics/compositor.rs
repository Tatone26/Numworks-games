//! Low-level rasterizer, layer sorter, reverse-Z occlusion culler, and display batcher.

use crate::graphics::dirty_grid::DirtyGrid;
use crate::graphics::particles::ParticleSystem;
use crate::graphics::sprite::Sprite;
use crate::graphics::tilemap::Tilemap;
use crate::graphics::viewport::Viewport;
use numworks_utils::eadk::{display, Color, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum InterlaceMode {
    #[default]
    None,
    Columns,
    Rows,
    Checkerboard,
}

impl InterlaceMode {
    #[inline(always)]
    pub fn allows_column(self, cx: usize, phase: usize) -> bool {
        match self {
            Self::Columns => (cx % 2) == phase,
            _ => true,
        }
    }

    #[inline(always)]
    pub fn allows_cell(self, cx: usize, cy: usize, phase: usize) -> bool {
        match self {
            Self::None | Self::Columns => true,
            Self::Rows => (cy % 2) == phase,
            Self::Checkerboard => ((cx + cy) % 2) == phase,
        }
    }
}

pub enum Renderable<'r, 'a, const TILE_SIZE: usize, const CELL_AREA: usize> {
    Tilemap(&'r mut Tilemap<'a, TILE_SIZE, CELL_AREA>),
    Sprite(&'r mut Sprite<'a, TILE_SIZE, CELL_AREA>),
    UiSprite(&'r mut Sprite<'a, TILE_SIZE, CELL_AREA>),
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

    #[inline(always)]
    pub fn is_opaque(&self) -> bool {
        match self {
            Renderable::Tilemap(tm) => !tm.transparent,
            _ => false,
        }
    }

    #[inline(always)]
    pub fn screen_bounds(&self, viewport: &Viewport) -> Option<[i16; 4]> {
        match self {
            Renderable::Tilemap(tm) => tm.screen_bounds(viewport),
            Renderable::Sprite(s) => {
                let (sx0, sy0) =
                    viewport.world_to_screen(s.position[0] as i32, s.position[1] as i32);
                let sx1 = sx0 + s.pixel_width() as i16;
                let sy1 = sy0 + s.pixel_height() as i16;
                Some([sx0, sy0, sx1, sy1])
            }
            Renderable::UiSprite(s) => {
                let sx0 = s.position[0];
                let sy0 = s.position[1];
                let sx1 = sx0 + s.pixel_width() as i16;
                let sy1 = sy0 + s.pixel_height() as i16;
                Some([sx0, sy0, sx1, sy1])
            }
        }
    }

    #[inline(always)]
    pub(crate) fn commit_frame(&mut self) {
        match self {
            Renderable::Tilemap(tm) => tm.commit_frame(),
            Renderable::Sprite(s) => s.commit_frame(),
            Renderable::UiSprite(s) => s.commit_frame(),
        }
    }
}

pub(crate) struct Compositor<
    const TILE_SIZE: usize,
    const CELL_AREA: usize,
    const SCREEN_COLS: usize,
    const SCREEN_ROWS: usize,
    const R_CAP: usize,
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
        const R_CAP: usize,
    > Compositor<TILE_SIZE, CELL_AREA, SCREEN_COLS, SCREEN_ROWS, R_CAP>
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

    pub fn render<'a, const PCAP: usize>(
        &mut self,
        items: &mut [Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
        particles: &[&ParticleSystem<PCAP>],
        viewport: &Viewport,
        scratch: &mut [Color],
        _viewport_moved: bool,
        interlace: InterlaceMode,
    ) {
        items.sort_unstable_by_key(|item| item.z());
        let phase = (self.frame % 2) as usize;

        let item_count = items.len().min(R_CAP);
        let mut bounds = [None; R_CAP];

        for (idx, item) in items[..item_count].iter().enumerate() {
            bounds[idx] = item.screen_bounds(viewport);
        }

        let (mask_scratch, pixel_scratch) = scratch.split_at_mut(CELL_AREA);
        let covered: &mut [bool; CELL_AREA] =
            unsafe { &mut *(mask_scratch.as_mut_ptr() as *mut [bool; CELL_AREA]) };

        for cx in (0..SCREEN_COLS).rev() {
            if !interlace.allows_column(cx, phase) {
                continue;
            }

            let col_mask = 1u32 << cx;

            let mut col_has_dirty = false;
            for r in 0..SCREEN_ROWS {
                if (self.grid.curr[r] & col_mask) != 0 {
                    col_has_dirty = true;
                    break;
                }
            }
            if !col_has_dirty {
                continue;
            }

            let cell_x = viewport.screen_x as i16 + (cx * TILE_SIZE) as i16;
            let cell_x1 = cell_x + TILE_SIZE as i16;

            let mut y = 0;
            while let Some((span_start, span_end)) =
                self.find_next_dirty_span(cx, y, col_mask, interlace, phase)
            {
                self.render_vertical_span(
                    cx,
                    cell_x,
                    cell_x1,
                    span_start,
                    span_end,
                    col_mask,
                    &items[..item_count],
                    &bounds[..item_count],
                    particles,
                    viewport,
                    pixel_scratch,
                    covered,
                );
                y = span_end;
            }
        }
    }

    #[inline(always)]
    fn find_next_dirty_span(
        &self,
        cx: usize,
        mut y: usize,
        col_mask: u32,
        interlace: InterlaceMode,
        phase: usize,
    ) -> Option<(usize, usize)> {
        while y < SCREEN_ROWS {
            let is_dirty = (self.grid.curr[y] & col_mask) != 0;
            if is_dirty && interlace.allows_cell(cx, y, phase) {
                break;
            }
            y += 1;
        }

        if y >= SCREEN_ROWS {
            return None;
        }

        let span_start = y;

        while y < SCREEN_ROWS {
            let is_dirty = (self.grid.curr[y] & col_mask) != 0;
            if !is_dirty || !interlace.allows_cell(cx, y, phase) {
                break;
            }
            y += 1;
        }

        Some((span_start, y))
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn render_vertical_span<'a, const PCAP: usize>(
        &mut self,
        cx: usize,
        cell_x: i16,
        cell_x1: i16,
        span_start: usize,
        span_end: usize,
        col_mask: u32,
        items: &[Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
        bounds: &[Option<[i16; 4]>],
        particles: &[&ParticleSystem<PCAP>],
        viewport: &Viewport,
        pixel_scratch: &mut [Color],
        covered: &mut [bool; CELL_AREA],
    ) {
        let span_rows = span_end - span_start;
        let start_screen_y = viewport.screen_y as i16 + (span_start * TILE_SIZE) as i16;

        for (idx, cy) in (span_start..span_end).enumerate() {
            let cell_y = start_screen_y + (idx * TILE_SIZE) as i16;
            let dst_offset = idx * CELL_AREA;

            let cell_buf: &mut [Color; CELL_AREA] = unsafe {
                &mut *(pixel_scratch.as_mut_ptr().add(dst_offset) as *mut [Color; CELL_AREA])
            };

            self.composite_cell(
                cx,
                cy,
                [cell_x, cell_y],
                cell_x1,
                items,
                bounds,
                particles,
                viewport,
                cell_buf,
                covered,
            );

            self.grid.curr[cy] &= !col_mask;
        }

        display::push_rect(
            Rect {
                x: cell_x as u16,
                y: start_screen_y as u16,
                width: TILE_SIZE as u16,
                height: (span_rows * TILE_SIZE) as u16,
            },
            &pixel_scratch[..span_rows * CELL_AREA],
        );
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn composite_cell<'a, const PCAP: usize>(
        &self,
        cx: usize,
        cy: usize,
        cell_pos: [i16; 2],
        cell_x1: i16,
        items: &[Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
        bounds: &[Option<[i16; 4]>],
        particles: &[&ParticleSystem<PCAP>],
        viewport: &Viewport,
        buffer: &mut [Color; CELL_AREA],
        covered: &mut [bool; CELL_AREA],
    ) {
        covered.fill(false);
        let mut pixels_covered: usize = 0;

        let [cell_x, cell_y] = cell_pos;
        let cell_y1 = cell_y + TILE_SIZE as i16;

        for (idx, item) in items.iter().enumerate().rev() {
            if pixels_covered >= CELL_AREA {
                break;
            }

            let Some([sx0, sy0, sx1, sy1]) = bounds[idx] else {
                continue;
            };

            if sx0 >= cell_x1 || sx1 <= cell_x || sy0 >= cell_y1 || sy1 <= cell_y {
                continue;
            }

            match item {
                Renderable::Tilemap(tm) => {
                    tm.blit_to_cell_reverse_z(
                        [cx, cy],
                        [viewport.x, viewport.y],
                        buffer,
                        covered,
                        &mut pixels_covered,
                        self.frame,
                    );
                }
                Renderable::Sprite(s) => {
                    s.blit_to_cell_reverse_z(
                        buffer,
                        covered,
                        &mut pixels_covered,
                        cell_pos,
                        [sx0, sy0],
                    );
                }
                Renderable::UiSprite(s) => {
                    s.blit_to_cell_reverse_z(
                        buffer,
                        covered,
                        &mut pixels_covered,
                        cell_pos,
                        [sx0, sy0],
                    );
                }
            }

            // Once the cell is full (from an opaque tilemap tile, or sprite stack), stop!
            if pixels_covered >= CELL_AREA {
                break;
            }
        }

        if pixels_covered < CELL_AREA {
            if pixels_covered == 0 {
                buffer.fill(self.clear_color);
            } else {
                for (idx, &is_covered) in covered.iter().enumerate() {
                    if !is_covered {
                        buffer[idx] = self.clear_color;
                    }
                }
            }
        }

        // Overlay mathematically evaluated particles bypassing the `covered` mask logic
        for ps in particles {
            if ps.active {
                ps.blit_to_cell::<TILE_SIZE, CELL_AREA>(cell_pos, viewport, buffer);
            }
        }
    }
}
