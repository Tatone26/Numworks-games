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
    pub fn draw_tile(&self) {
        TILESET.draw_tile(
            Point {
                x: self.grid_point.x * TILE_SIZE + X_GRID_OFFSET,
                y: self.grid_point.y * TILE_SIZE,
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

impl World {
    pub fn draw_world(&self) {
        for y in 0..self.grid.len() {
            for x in 0..self.grid[y].len() {
                self.grid[y][x].draw_tile();
            }
        }
    }
}
