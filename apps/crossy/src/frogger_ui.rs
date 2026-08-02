use numworks_utils::{
    eadk::{Color, Point},
    graphical::tiling::Tileset,
    include_bytes_align_as,
};

use crate::{
    frog::Frog,
    world::{Tile, World},
};

const IMAGE_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/sprites.nppm");

const TILE_SIZE: u16 = 12;
static TILESET: Tileset = Tileset::new(TILE_SIZE, 10, IMAGE_BYTES);

pub const X_GRID_OFFSET: u16 = 36;

impl Tile {
    pub fn draw_tile(&self, y_offset: u16) {
        TILESET.draw_tile(
            Point {
                x: self.grid_point.x * TILE_SIZE + X_GRID_OFFSET,
                y: self.grid_point.y * TILE_SIZE + y_offset,
            },
            self.tileset_index(),
            1,
            false,
        );
    }
}

impl Frog {
    pub fn draw_frog(&self) {
        let grid_pos = self.grid_pos();
        TILESET.draw_tile(
            Point {
                x: grid_pos.x * TILE_SIZE + X_GRID_OFFSET,
                y: grid_pos.y * TILE_SIZE,
            },
            Point { x: 0, y: 0 },
            1,
            true,
        );
    }
}

pub struct ScrollController {
    active_step: u8,
    is_scrolling: bool,
}

impl ScrollController {
    pub fn new() -> Self {
        Self {
            active_step: 0,
            is_scrolling: false,
        }
    }

    pub fn is_scrolling(&self) -> bool {
        self.is_scrolling
    }

    pub fn trigger_scroll(&mut self) {
        self.is_scrolling = true;
        self.active_step = 0;
    }

    /// Returns the active Y pixel offset for the current frame step (4px, 8px, 12px)
    pub fn current_y_offset(&self) -> Option<u16> {
        if !self.is_scrolling {
            return None;
        }

        match self.active_step {
            0 => Some(4),
            1 => Some(8),
            2 => Some(12),
            _ => None,
        }
    }

    pub fn advance(&mut self) {
        if self.is_scrolling {
            self.active_step += 1;
            if self.active_step >= 3 {
                self.is_scrolling = false;
                self.active_step = 0;
            }
        }
    }
}

impl World {
    /// Draws the entire grid shifted down by y_offset pixels
    pub fn draw_world(&self, y_offset: u16) {
        for y in 0..self.grid.len() {
            for x in 0..self.grid[y].len() {
                self.grid[y][x].draw_tile(y_offset);
            }
        }
    }
}
