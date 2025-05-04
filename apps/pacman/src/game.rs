use core::panic::PanicInfo;

use heapless::Vec;
use numworks_utils::{
    eadk::{
        display::{push_rect_uniform, wait_for_vblank, SCREEN_WIDTH},
        key, keyboard, Color, Point, Rect,
    },
    graphical::{tiling::Tileset, ColorConfig},
    include_bytes_align_as,
    menu::{
        settings::{write_values_to_file, Setting},
        start_menu,
    },
};

use crate::pac_ui::draw_maze;

// This dictates the principal colors that will be used
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: Color::WHITE,
    alt: Color::RED,
};

fn vis_addon() {
    push_rect_uniform(
        Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        },
        Color::BLACK,
    );
}
/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut Setting; 2] = [
        &mut Setting {
            name: "Modifiable option !\0",
            choice: 0,
            values: Vec::from_slice(&[1, 0]).unwrap(),
            texts: Vec::from_slice(&["True\0", "False\0"]).unwrap(),
            fixed_values: true,
            user_modifiable: true,
        },
        &mut Setting {
            name: "High-score option !\0",
            choice: 0,                              // forced
            values: Vec::from_slice(&[0]).unwrap(), // default value
            texts: Vec::new(),
            fixed_values: false,    // allows using any value
            user_modifiable: false, // will not appear in "setting" page
        },
    ];
    loop {
        let start = start_menu(
            "TEST\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./data/model_controls.txt"),
            "pacman", // filename to store settings
        );
        // The menu does everything itself !
        if start == 0 {
            // exemple of a way to have a stored value modified by the game (like a high_score)
            let mut high_score: u32 = opt[1].get_setting_value();
            loop {
                // a loop where the game is played again and again, which means it should be 100% contained after the menu
                // calling the game based on the parameters is better
                let action = game(opt[0].get_setting_value() != 0, &mut high_score);
                // necessary to store the high_score (or other similar data):
                opt[1].set_value(high_score);
                write_values_to_file(&mut opt, "pacman");
                // this shoudln't change
                if action == 2 {
                    // 2 means quitting
                    return;
                } else if action == 1 {
                    // 1 means back to menu
                    break;
                } // if action == 0 : rejouer
            }
        } else {
            return;
        }
    }
}

pub const TILE_SIZE: u16 = 8;
const GRID_WIDTH: u16 = 28;
const GRID_HEIGHT: u16 = 30;
const ARRAY_SIZE: usize = (GRID_WIDTH * GRID_HEIGHT) as usize;
pub const X_GRID_OFFSET: u16 = (SCREEN_WIDTH - GRID_WIDTH * TILE_SIZE) / 2;

const CELL_SIZE: u16 = 16;
const GAME_GRID_WIDTH: u16 = GRID_WIDTH / 2;
const GAME_GRID_HEIGHT: u16 = GRID_HEIGHT / 2;

const WALL_IMAGES_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/walls.nppm");
const SPRITES_IMAGES_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/sprites.nppm");
// Sadly, had to remove a line as the screen was 8 pixels too short...
const MAZE_BYTES: &str = include_str!("./data/maze.txt");

/// I'm using two Tileset because one is using 8 pixels wide tiles and the second 16 pixels wide.
pub static TILESET_WALLS: Tileset = Tileset::new(TILE_SIZE, 16, WALL_IMAGES_BYTES);
pub static TILESET_SPRITES: Tileset = Tileset::new(CELL_SIZE, 8, SPRITES_IMAGES_BYTES);

#[derive(Clone, Copy)]
enum Space {
    Wall,
    Empty,
    Point,
    Superball,
    Fruit,
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}

struct Grid {
    grid: [Space; ARRAY_SIZE],
}

impl Grid {
    pub fn new(maze_file: &str) -> Grid {
        Grid {
            grid: Grid::read_file(maze_file),
        }
    }

    fn read_file(maze_file: &str) -> [Space; ARRAY_SIZE] {
        let mut grid: [Space; ARRAY_SIZE] = [Space::Empty; ARRAY_SIZE];
        for (line, s) in maze_file
            .lines()
            .filter(|s| !s.is_empty())
            .enumerate()
            .take(GRID_HEIGHT as usize)
        {
            for (i, c) in s.chars().enumerate().take(GRID_WIDTH as usize) {
                match c {
                    '.' => grid[line * GRID_WIDTH as usize + i] = Space::Point,
                    '°' => grid[line * GRID_WIDTH as usize + i] = Space::Superball,
                    ' ' => grid[line * GRID_WIDTH as usize + i] = Space::Empty,
                    _ => grid[line * GRID_WIDTH as usize + i] = Space::Wall,
                }
            }
        }
        return grid;
    }
}

/// The entire game is here.
pub fn game(_exemple: bool, _high_score: &mut u32) -> u8 {
    let mut grid = Grid::new(MAZE_BYTES);
    draw_maze(MAZE_BYTES);
    let mut direction = Direction::Right;
    let mut pac_position = Point {
        x: TILE_SIZE / 2 + TILE_SIZE * 1,
        y: TILE_SIZE / 2 + TILE_SIZE * 1,
    };
    loop {
        let scan = keyboard::scan();
        if scan.key_down(key::OK) {
            break;
        }
        direction = read_input(&direction);
        let next = next_pos(&pac_position, &direction, &grid);
        move_to(pac_position, next);
        pac_position = next;
    }
    1
}

const fn direction_coordinates(dir: &Direction) -> (i16, i16) {
    match dir {
        Direction::Up => (0, -1),
        Direction::Down => (0, 1),
        Direction::Right => (1, 0),
        Direction::Left => (-1, 0),
    }
}

// TODO : changement de direction seulement à la bonne position.
fn read_input(current_dir: &Direction) -> Direction {
    let scan = keyboard::scan();
    if scan.key_down(key::UP) && !scan.key_down(key::DOWN) {
        return Direction::Up;
    } else if scan.key_down(key::DOWN) && !scan.key_down(key::UP) {
        return Direction::Down;
    } else if scan.key_down(key::RIGHT) && !scan.key_down(key::LEFT) {
        return Direction::Right;
    } else if scan.key_down(key::LEFT) && !scan.key_down(key::RIGHT) {
        return Direction::Left;
    }

    *current_dir
}

fn get_grid_value_from_abs_pos(pos: &Point, grid: &Grid, direction: (i16, i16)) -> Space {
    let x = (pos.x as i16 + (TILE_SIZE / 2) as i16 * direction.0) as u16 / TILE_SIZE;
    let y = (pos.y as i16 + (TILE_SIZE / 2) as i16 * direction.1) as u16 / TILE_SIZE;
    return *grid.grid.get((x + y * GRID_WIDTH) as usize).unwrap();
}

fn next_pos(current_pos: &Point, current_dir: &Direction, grid: &Grid) -> Point {
    let dir = direction_coordinates(current_dir);
    let next_pos = Point {
        x: (current_pos.x as i16 + dir.0) as u16,
        y: (current_pos.y as i16 + dir.1) as u16,
    };
    match get_grid_value_from_abs_pos(&next_pos, grid, direction_coordinates(current_dir)) {
        Space::Wall => *current_pos,
        _ => next_pos,
    }
}

fn move_to(pos: Point, next_pos: Point) -> Point {
    wait_for_vblank();
    push_rect_uniform(
        Rect {
            x: pos.x + X_GRID_OFFSET - TILE_SIZE / 2 - 1,
            y: pos.y - TILE_SIZE / 2 - 1,
            width: 13,
            height: 14,
        },
        Color::BLACK,
    );
    TILESET_SPRITES.draw_tile(
        Point {
            x: next_pos.x + X_GRID_OFFSET - TILE_SIZE / 2 - 1,
            y: next_pos.y - TILE_SIZE / 2 - 1,
        },
        Point { x: 0, y: 0 },
        1,
        true,
    );
    next_pos
}
