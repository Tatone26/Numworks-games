use numworks_utils::eadk::{
    display::{SCREEN_HEIGHT, SCREEN_WIDTH},
    Point,
};

use crate::{frogger_ui::X_GRID_OFFSET, moveable_obstacle::MoveableObstacle};

const TILE_SIZE: u16 = 12;

const MAX_NUMBER_OF_MOVEABLES: usize = 5;
pub const GRID_WIDTH: usize = ((SCREEN_WIDTH - X_GRID_OFFSET * 2) / TILE_SIZE) as usize;
pub const GRID_HEIGHT: usize = (SCREEN_HEIGHT / TILE_SIZE) as usize;

#[derive(Copy, Clone)]
pub struct Tile {
    pub grid_point: Point,
    pub is_safe: bool,
    pub is_killer: bool,
    pub has_obstacle: bool,
    tileset_index: Point,
}

impl Tile {
    pub fn tileset_index(&self) -> Point {
        self.tileset_index
    }
}

pub struct World {
    pub grid: [[Tile; GRID_WIDTH]; GRID_HEIGHT],
    pub moveable_obstacles: [Option<MoveableObstacle>; MAX_NUMBER_OF_MOVEABLES],
    pub scrolled_rows: usize,
}

impl World {
    pub fn new() -> Self {
        let mut grid = [[Tile {
            grid_point: Point { x: 0, y: 0 },
            is_safe: true,
            is_killer: false,
            has_obstacle: false,
            tileset_index: Point { x: 0, y: 0 },
        }; GRID_WIDTH]; GRID_HEIGHT];

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                grid[y][x] = Tile {
                    grid_point: Point {
                        x: x as u16,
                        y: y as u16,
                    },
                    is_safe: true,
                    is_killer: false,
                    has_obstacle: false,
                    tileset_index: Point {
                        x: 0,
                        y: 1 + if y % 2 == 0 { 1 } else { 0 },
                    },
                };
            }
        }

        Self {
            grid,
            moveable_obstacles: [const { None }; MAX_NUMBER_OF_MOVEABLES],
            scrolled_rows: 0,
        }
    }

    pub fn shift_down(&mut self) {
        for y in (1..GRID_HEIGHT).rev() {
            self.grid[y] = self.grid[y - 1];
            for x in 0..GRID_WIDTH {
                self.grid[y][x].grid_point.y = y as u16;
            }
        }

        self.scrolled_rows += 1;
        let count = self.scrolled_rows;

        for x in 0..GRID_WIDTH {
            // new tiles creation TODO
            self.grid[0][x] = Tile {
                grid_point: Point { x: x as u16, y: 0 },
                is_safe: true,
                is_killer: false,
                has_obstacle: false,
                tileset_index: Point {
                    x: 0,
                    y: 1 + if count % 2 == 0 { 1 } else { 0 },
                },
            };
        }
    }

    pub fn get_tile_at(&self, point: Point) -> Option<&Tile> {
        if point.x < GRID_WIDTH as u16 && point.y < GRID_HEIGHT as u16 {
            Some(&self.grid[point.y as usize][point.x as usize])
        } else {
            None
        }
    }
}
