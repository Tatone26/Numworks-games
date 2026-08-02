use numworks_utils::eadk::{
    display::{SCREEN_HEIGHT, SCREEN_WIDTH},
    Point,
};

use crate::{frogger_ui::X_GRID_OFFSET, moveable_obstacle::MoveableObstacle};

/// a world consist of a grid containing safe tiles, killer tiles (river), obstacles, and moveable obstacles.

const TILE_SIZE: u16 = 12;

const MAX_NUMBER_OF_MOVEABLES: usize = 5;
const GRID_WIDTH: usize = ((SCREEN_WIDTH - X_GRID_OFFSET * 2) / TILE_SIZE) as usize;
const GRID_HEIGHT: usize = (SCREEN_HEIGHT / TILE_SIZE) as usize;

#[derive(Copy, Clone)]
pub struct Tile {
    pub grid_point: Point,
    pub is_safe: bool,
    pub is_killer: bool,
    pub has_obstacle: bool,
    tileset_index: Point,
}

impl Tile {
    // make tileset_index publicly read-only
    pub fn tileset_index(&self) -> Point {
        self.tileset_index
    }
}

pub struct World {
    pub grid: [[Tile; GRID_WIDTH]; GRID_HEIGHT],
    pub moveable_obstacles: [Option<MoveableObstacle>; MAX_NUMBER_OF_MOVEABLES],
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
                    has_obstacle: y == 4, // quick test, row 4 is blocked by obstacles.
                    tileset_index: Point {
                        x: 0,
                        y: 1 + if y == 4 { 1 } else { 0 },
                    },
                };
            }
        }

        Self {
            grid,
            moveable_obstacles: [const { None }; MAX_NUMBER_OF_MOVEABLES],
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
