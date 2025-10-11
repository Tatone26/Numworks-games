use numworks_utils::{
    eadk::{
        display::{draw_string, push_rect_uniform, wait_for_vblank},
        Color, Point, Rect,
    },
    graphical::tiling::Tileset,
    include_bytes_align_as,
    utils::string_from_u16,
};

use crate::{
    game::{Grid, Space, GRID_WIDTH, TILE_SIZE, X_GRID_OFFSET},
    ghost::GhostType,
    moveable::{next_pos, Direction},
};

const WALL_IMAGES_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/walls.nppm");
const SPRITES_IMAGES_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/sprites.nppm");

/// I'm using two Tileset because one is using 8 pixels wide tiles and the second 16 pixels wide.
pub static TILESET_WALLS: Tileset = Tileset::new(TILE_SIZE, 16, WALL_IMAGES_BYTES);
pub static TILESET_SPRITES: Tileset = Tileset::new(TILE_SIZE * 2, 8, SPRITES_IMAGES_BYTES);

/// Gives the absolute pixel position from the given position on the grid.
const fn abs_from_pos(pos: Point) -> Point {
    Point {
        x: pos.x * TILE_SIZE + X_GRID_OFFSET,
        y: pos.y * TILE_SIZE,
    }
}

/// Draws the player. Help.
pub fn draw_player(next_po: Point, steps: u8, dir: &Direction, frames: u32, wrapping: bool) {
    let np = abs_from_pos(next_po);
    let offset = match dir {
        Direction::Up | Direction::Down => 0,
        Direction::Right | Direction::Left => 1,
    };
    let p = Point {
        x: (np.x as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().0) as u16,
        y: (np.y as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().1) as u16 + offset,
    };
    TILESET_SPRITES.draw_tile(
        p,
        Point {
            x: ((frames / 4) % 2) as u16,
            y: match dir {
                Direction::Up => 2,
                Direction::Down => 3,
                Direction::Right => 0,
                Direction::Left => 1,
            },
        },
        1,
        true,
    );
    if wrapping {
        // clear the part that overflows the play screen (wanted to do that less hard-coded but who cares)
        push_rect_uniform(
            Rect {
                x: if dir == &Direction::Left {
                    X_GRID_OFFSET - 14
                } else if dir == &Direction::Right {
                    X_GRID_OFFSET + GRID_WIDTH * TILE_SIZE
                } else {
                    p.x
                },
                y: p.y,
                width: 14,
                height: 14,
            },
            Color::BLACK,
        );
    }
}

/// Clear the player. Help bis
pub fn clear_player(pos: Point, steps: u8, dir: &Direction, grid: &Grid) {
    // TODO : Some problems when the player change direction to opposite ; can cut part of destination cell.
    let p = abs_from_pos(pos);
    let offset = match dir {
        Direction::Up | Direction::Down => 0,
        Direction::Right | Direction::Left => 1,
    };
    push_rect_uniform(
        Rect {
            x: (p.x as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().0) as u16 + 1,
            y: (p.y as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().1) as u16
                + offset,
            width: 14,
            height: 14,
        },
        Color::BLACK,
    );
    let (next, wrapping) = next_pos(pos, dir);
    for p in if wrapping { [pos, pos] } else { [next, pos] } {
        match grid.get((p.x + p.y * GRID_WIDTH) as usize) {
            Some(Space::Point) => {
                TILESET_WALLS.draw_tile(abs_from_pos(p), get_tile_position('.').unwrap(), 1, false)
            }
            Some(Space::Superball) => {
                TILESET_WALLS.draw_tile(abs_from_pos(p), get_tile_position('°').unwrap(), 1, false)
            }
            Some(Space::Empty) | None => {
                TILESET_WALLS.draw_tile(abs_from_pos(p), get_tile_position(' ').unwrap(), 1, false)
            }
            // TODO : fruits :)
            Some(_) => (),
        }
    }
}

/// Draws a ghost. Help.
pub fn draw_ghost(
    next_po: Point,
    steps: u8,
    dir: &Direction,
    frames: u32,
    wrapping: bool,
    gtype: &GhostType,
    at_home: bool,
) {
    let mut np = abs_from_pos(next_po);
    if at_home {
        np.y = np.y + TILE_SIZE / 2;
    }

    let p = Point {
        x: (np.x as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().0) as u16,
        y: (np.y as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().1) as u16,
    };
    TILESET_SPRITES.draw_tile(
        p,
        Point {
            x: match dir {
                Direction::Up => 4,
                Direction::Down => 6,
                Direction::Right => 0,
                Direction::Left => 2,
            } + ((frames / 4) % 2) as u16,
            y: match gtype {
                GhostType::Blinky => 4,
                GhostType::Pinky => 5,
                GhostType::Inky => 6,
                GhostType::Clyde => 7,
            },
        },
        1,
        true,
    );
    if wrapping {
        // clear the part that overflows the play screen (wanted to do that less hard-coded but who cares)
        push_rect_uniform(
            Rect {
                x: if dir == &Direction::Left {
                    X_GRID_OFFSET - 14
                } else if dir == &Direction::Right {
                    X_GRID_OFFSET + GRID_WIDTH * TILE_SIZE
                } else {
                    p.x
                },
                y: p.y,
                width: 15,
                height: 15,
            },
            Color::BLACK,
        );
    }
}

/// Clear the player. Help bis
pub fn clear_ghost(pos: Point, steps: u8, dir: &Direction, grid: &Grid, is_home: bool) {
    let mut p = abs_from_pos(pos);
    if is_home {
        p.y = p.y + TILE_SIZE / 2;
    }
    push_rect_uniform(
        Rect {
            x: (p.x as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().0) as u16,
            y: (p.y as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().1) as u16,
            width: 16,
            height: 16,
        },
        Color::BLACK,
    );
    let (next, wrapping) = next_pos(pos, dir);
    for p_ in if wrapping { [pos] } else { [pos] } {
        for off in [(0, 0), (0, 1), (0, -1), (1, 0), (-1, 0)] {
            // TODO : hating this but otherwise getting artifacts and broken balls
            let p = Point {
                x: (p_.x as i16 + off.0) as u16,
                y: (p_.y as i16 + off.1) as u16,
            };
            match grid.get((p.x as u16 + p.y as u16 * GRID_WIDTH) as usize) {
                Some(Space::Point) => TILESET_WALLS.draw_tile(
                    abs_from_pos(p),
                    get_tile_position('.').unwrap(),
                    1,
                    false,
                ),
                Some(Space::Superball) => TILESET_WALLS.draw_tile(
                    abs_from_pos(p),
                    get_tile_position('°').unwrap(),
                    1,
                    false,
                ),
                Some(Space::Empty) | None => TILESET_WALLS.draw_tile(
                    abs_from_pos(p),
                    get_tile_position(' ').unwrap(),
                    1,
                    false,
                ),
                // TODO : fruits :)
                Some(_) => (),
            }
        }
    }
}

/// Determines the tile position based on the character.
const fn get_tile_position(c: char) -> Option<Point> {
    match c {
        '.' => Some(Point { x: 13, y: 2 }),
        '°' => Some(Point { x: 15, y: 2 }),
        ' ' => Some(Point { x: 12, y: 2 }),
        '0'..='9' => Some(Point {
            x: (c as u8 - b'0') as u16,
            y: 0,
        }),
        'a'..='f' => Some(Point {
            x: (c as u8 - b'a') as u16 + 10,
            y: 0,
        }),
        'g'..='v' => Some(Point {
            x: (c as u8 - b'g') as u16,
            y: 1,
        }),
        'w'..='z' => Some(Point {
            x: (c as u8 - b'w') as u16,
            y: 2,
        }),
        'A'..='L' => Some(Point {
            x: (c as u8 - b'A') as u16 + 4,
            y: 2,
        }),
        _ => None, // Handle unexpected characters
    }
}

pub fn draw_score(score: u16) {
    draw_string(
        &string_from_u16(score),
        Point { x: 0, y: 0 },
        false,
        Color::WHITE,
        Color::BLACK,
    );
}

/// Draws the entirety of the maze (walls, points) based on a given file.
/// Used only at launch, doesn't need to be called again.
pub fn draw_maze(file: &str) {
    for (line, s) in file.lines().filter(|s| !s.is_empty()).enumerate() {
        wait_for_vblank();
        for (i, c) in s.chars().enumerate() {
            let pos = Point {
                x: i as u16 * TILE_SIZE + X_GRID_OFFSET,
                y: line as u16 * TILE_SIZE,
            };
            if let Some(tile_pos) = get_tile_position(c) {
                TILESET_WALLS.draw_tile(pos, tile_pos, 1, false);
            }
        }
    }
}
