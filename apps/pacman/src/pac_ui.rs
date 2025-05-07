use core::iter::Empty;

use numworks_utils::{
    eadk::{
        display::{push_rect_uniform, wait_for_vblank},
        Color, Point, Rect,
    },
    graphical::tiling::Tileset,
    include_bytes_align_as,
};

use crate::game::{Direction, Grid, Space, GRID_WIDTH, TILE_SIZE, X_GRID_OFFSET};

const WALL_IMAGES_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/walls.nppm");
const SPRITES_IMAGES_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/sprites.nppm");

/// I'm using two Tileset because one is using 8 pixels wide tiles and the second 16 pixels wide.
pub static TILESET_WALLS: Tileset = Tileset::new(TILE_SIZE, 16, WALL_IMAGES_BYTES);
pub static TILESET_SPRITES: Tileset = Tileset::new(TILE_SIZE * 2, 8, SPRITES_IMAGES_BYTES);

const fn abs_from_pos(pos: Point) -> Point {
    Point {
        x: pos.x * TILE_SIZE + X_GRID_OFFSET,
        y: pos.y * TILE_SIZE,
    }
}

pub fn draw_player(next_pos: Point, steps: u8, dir: &Direction, frames: u32) {
    let np = abs_from_pos(next_pos);
    let offset = match dir {
        Direction::Up | Direction::Down => 0,
        Direction::Right | Direction::Left => 1,
    };
    TILESET_SPRITES.draw_tile(
        Point {
            x: (np.x as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().0) as u16,
            y: (np.y as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().1) as u16
                + offset,
        },
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
}

pub fn clear_player(pos: Point, steps: u8, dir: &Direction, grid: &Grid) {
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
    let next = Point {
        x: (pos.x as i16 + dir.to_vector().0) as u16,
        y: (pos.y as i16 + dir.to_vector().1) as u16,
    };
    match grid.get((next.x + next.y * GRID_WIDTH) as usize) {
        Some(Space::Empty) => (),
        Some(Space::Point) => TILESET_WALLS.draw_tile(
            abs_from_pos(next),
            get_tile_position('.').unwrap(),
            1,
            false,
        ),
        Some(Space::Superball) => TILESET_WALLS.draw_tile(
            abs_from_pos(next),
            get_tile_position('°').unwrap(),
            1,
            false,
        ),
        Some(_) => (),
        None => panic!(),
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

/// Draws the entirety of the maze (walls, points) based on a given file.
/// Used only at launch, doesn't need to be called again next.
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
