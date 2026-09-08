use numworks_utils::eadk::{
    display::{push_rect, wait_for_vblank, SCREEN_HEIGHT, SCREEN_WIDTH},
    Color, Rect,
};

use crate::compositor::{Compositor, Renderable};

pub struct Engine<
    const TILE_SIZE: usize,
    const CELL_AREA: usize,
    const COLS: usize,
    const ROWS: usize,
> {
    pub compositor: Compositor<TILE_SIZE, CELL_AREA, COLS, ROWS>,
    pub frame: u32,
}

impl<const TILE_SIZE: usize, const CELL_AREA: usize, const COLS: usize, const ROWS: usize>
    Engine<TILE_SIZE, CELL_AREA, COLS, ROWS>
{
    pub fn new(clear_color: Color) -> Self {
        // Initial screen fill with clear color
        push_rect(
            Rect {
                x: 0,
                y: 0,
                width: SCREEN_WIDTH as u16,
                height: SCREEN_HEIGHT as u16,
            },
            &[clear_color; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize], // or tiled in small chunks if RAM is tight
        );

        Self {
            compositor: Compositor::new(clear_color),
            frame: 0,
        }
    }

    /// Mark all cells dirty (e.g. when changing level or clearing screen)
    pub fn mark_all_dirty(&mut self) {
        self.compositor.grid.mark_all();
    }

    /// Renders a frame containing any combination of Tilemaps and Sprites
    pub fn render_frame<'a>(
        &mut self,
        items: &mut [Renderable<'_, 'a, TILE_SIZE, CELL_AREA, COLS, ROWS>],
    ) {
        // 1. Tag dirty cells for moving sprites
        for item in items.iter() {
            if let Renderable::Sprite(s) = item {
                if s.moved {
                    self.compositor.grid.mark_rect(
                        s.prev_position,
                        s.pixel_width(),
                        s.pixel_height(),
                    );
                    self.compositor
                        .grid
                        .mark_rect(s.position, s.pixel_width(), s.pixel_height());
                }
            }
        }

        // 2. Hardware VSync
        wait_for_vblank();

        // 3. Composite dirty cells
        self.compositor.render(items);

        self.frame = self.frame.wrapping_add(1);
    }
}
