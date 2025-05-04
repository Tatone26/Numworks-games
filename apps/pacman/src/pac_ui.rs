use numworks_utils::eadk::{display::wait_for_vblank, Point};

use crate::game::{TILESET_WALLS, TILE_SIZE, X_GRID_OFFSET};

pub fn draw_maze(file: &str) {
    for (line, s) in file.lines().filter(|s| !s.is_empty()).enumerate() {
        wait_for_vblank();
        for (i, c) in s.chars().enumerate() {
            let pos = Point {
                x: i as u16 * TILE_SIZE + X_GRID_OFFSET,
                y: line as u16 * TILE_SIZE,
            };
            match c {
                '.' => {
                    TILESET_WALLS.draw_tile(pos, Point { x: 13, y: 2 }, 1, false);
                }
                '°' => {
                    TILESET_WALLS.draw_tile(pos, Point { x: 15, y: 2 }, 1, false);
                }
                ' ' => {
                    TILESET_WALLS.draw_tile(pos, Point { x: 12, y: 2 }, 1, false);
                }
                '0'..='9' => {
                    TILESET_WALLS.draw_tile(
                        pos,
                        Point {
                            x: (c as u8 - b'0') as u16,
                            y: 0,
                        },
                        1,
                        false,
                    );
                }
                'a'..='f' => {
                    TILESET_WALLS.draw_tile(
                        pos,
                        Point {
                            x: (c as u8 - b'a') as u16 + 10,
                            y: 0,
                        },
                        1,
                        false,
                    );
                }
                'g'..='v' => {
                    TILESET_WALLS.draw_tile(
                        pos,
                        Point {
                            x: (c as u8 - b'g') as u16,
                            y: 1,
                        },
                        1,
                        false,
                    );
                }
                'w'..='z' => {
                    TILESET_WALLS.draw_tile(
                        pos,
                        Point {
                            x: (c as u8 - b'w') as u16,
                            y: 2,
                        },
                        1,
                        false,
                    );
                }
                'A'..='L' => {
                    TILESET_WALLS.draw_tile(
                        pos,
                        Point {
                            x: (c as u8 - b'A') as u16 + 4,
                            y: 2,
                        },
                        1,
                        false,
                    );
                }
                _ => {
                    // Handle unexpected characters if necessary
                }
            }
        }
    }
}
