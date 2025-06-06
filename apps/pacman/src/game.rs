use heapless::Vec;
use numworks_utils::{
    eadk::{
        display::{push_rect_uniform, wait_for_vblank, SCREEN_WIDTH},
        key, keyboard, Color, Point, Rect,
    },
    graphical::ColorConfig,
    menu::{
        settings::{write_values_to_file, Setting},
        start_menu,
    },
};

use crate::pac_ui::{clear_player, draw_maze, draw_player};

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
pub const GRID_WIDTH: u16 = 28;
pub const GRID_HEIGHT: u16 = 30;
const ARRAY_SIZE: usize = (GRID_WIDTH * GRID_HEIGHT) as usize;
pub const X_GRID_OFFSET: u16 = (SCREEN_WIDTH - GRID_WIDTH * TILE_SIZE) / 2;

// Sadly, had to remove a line as the screen was 8 pixels too short...
const MAZE_BYTES: &str = include_str!("./data/maze.txt");

#[derive(Clone, Copy)]
pub enum Space {
    Wall,
    Empty,
    Point,
    Superball,
    Fruit,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    pub const fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }

    pub const fn to_vector(self) -> (i16, i16) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Right => (1, 0),
            Direction::Left => (-1, 0),
        }
    }
}

pub type Grid = [Space; ARRAY_SIZE];

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
    grid
}

const STEPS_PER_CELL: f32 = 8.0;

#[derive(Clone, Copy)]
struct Player {
    grid_position: Point,
    destination: Point,
    steps: f32,
    direction: Direction,
    speed: f32,
    superball_active: bool,
    wrapping: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            grid_position: Point { x: 1, y: 1 },
            destination: Point { x: 2, y: 1 },
            steps: 0.0,
            speed: 1.0,
            direction: Direction::Right,
            superball_active: false,
            wrapping: false,
        }
    }

    /// Returns the type of thing that has been eaten, Empty if none
    pub fn move_player(&mut self, grid: &mut Grid) -> Space {
        if self.grid_position.x == self.destination.x && self.grid_position.y == self.destination.y
        {
            return Space::Empty;
        }
        self.steps += if self.superball_active {
            self.speed * 1.5
        } else {
            self.speed
        };
        // eating if there is something to eat (on the next cell)
        let eaten = match grid.get((self.destination.x + self.destination.y * GRID_WIDTH) as usize)
        {
            Some(Space::Point) => {
                // eating point
                if self.steps >= STEPS_PER_CELL / 4.0 {
                    grid[(self.destination.x + self.destination.y * GRID_WIDTH) as usize] =
                        Space::Empty;
                    Space::Point
                } else {
                    Space::Empty
                }
            }
            Some(Space::Superball) => {
                // eating superball
                if self.steps >= 3.0 * (STEPS_PER_CELL / 4.0) {
                    grid[(self.destination.x + self.destination.y * GRID_WIDTH) as usize] =
                        Space::Empty;
                    self.superball_active = true;
                    Space::Superball
                } else {
                    Space::Empty
                }
            }
            _ => Space::Empty,
        };
        // if reached end of current cell, then move to next cell
        if self.steps >= STEPS_PER_CELL {
            // if arrived to next node
            self.grid_position = self.destination;
            let (next_pos, wrap) = next_pos(self.grid_position, &self.direction);
            self.wrapping = wrap;
            (self.destination, self.steps) =
                match grid.get((next_pos.x + next_pos.y * GRID_WIDTH) as usize) {
                    Some(Space::Wall) => (self.grid_position, 0.0),
                    Some(_) => (next_pos, self.steps % STEPS_PER_CELL as f32),
                    _ => (self.grid_position, 0.0),
                };
        }
        eaten
    }

    pub fn read_input(&mut self, grid: &Grid) {
        let scan = keyboard::scan();
        let new_dir = if scan.key_down(key::UP)
            && can_go_to(self.grid_position, &Direction::Up, grid)
        {
            Direction::Up
        } else if scan.key_down(key::DOWN) && can_go_to(self.grid_position, &Direction::Down, grid)
        {
            Direction::Down
        } else if scan.key_down(key::RIGHT)
            && can_go_to(self.grid_position, &Direction::Right, grid)
        {
            Direction::Right
        } else if scan.key_down(key::LEFT) && can_go_to(self.grid_position, &Direction::Left, grid)
        {
            Direction::Left
        } else {
            self.direction
        };
        if new_dir.opposite() == self.direction {
            let stopped = self.grid_position.x == self.destination.x
                && self.grid_position.y == self.destination.y;
            self.direction = new_dir;
            self.grid_position = self.destination;
            (self.destination, self.wrapping) = next_pos(self.grid_position, &self.direction);
            if !stopped {
                self.steps = f32::abs(STEPS_PER_CELL - self.steps);
            } else {
                self.steps = 0.0;
            }
        } else if new_dir != self.direction {
            self.steps = 0.0;
            self.direction = new_dir;
            (self.destination, self.wrapping) = next_pos(self.grid_position, &self.direction);
        }
    }
}

/// The entire game is here.
pub fn game(_exemple: bool, _high_score: &mut u32) -> u8 {
    let mut grid = read_file(MAZE_BYTES);
    draw_maze(MAZE_BYTES);
    let mut pac = Player::new();
    let mut frames: u32 = 0;
    // TODO : all the pacman rules. Yay !
    let mut score: u32 = 0;
    loop {
        let scan = keyboard::scan();
        if scan.key_down(key::OK) {
            break;
        }
        let old = pac.clone();
        pac.read_input(&grid);
        let eaten = pac.move_player(&mut grid);
        match eaten {
            Space::Superball | Space::Point => score += 1,
            _ => (),
        }
        wait_for_vblank();
        clear_player(old.grid_position, old.steps as u8, &old.direction, &grid);
        draw_player(
            pac.grid_position,
            pac.steps as u8,
            &pac.direction,
            frames,
            pac.wrapping,
        );

        frames = frames.wrapping_add(1);
    }
    1
}

fn can_go_to(from: Point, dir: &Direction, grid: &Grid) -> bool {
    let (next, _) = next_pos(from, dir);
    match grid.get((next.x + next.y * GRID_WIDTH) as usize) {
        Some(Space::Wall) => false,
        Some(_) => true,
        None => true,
    }
}

pub fn next_pos(from: Point, dir: &Direction) -> (Point, bool) {
    let next = (
        (from.x as i16 + dir.to_vector().0),
        (from.y as i16 + dir.to_vector().1),
    );
    if next.0 < 0 || next.1 < 0 || next.0 >= GRID_WIDTH as i16 || next.1 >= GRID_HEIGHT as i16 {
        // wrapping
        (
            Point {
                x: ((GRID_WIDTH as i16 + next.0) % GRID_WIDTH as i16) as u16,
                y: ((GRID_HEIGHT as i16 + next.1) % GRID_HEIGHT as i16) as u16,
            },
            true,
        )
    } else {
        (
            Point {
                x: next.0 as u16,
                y: next.1 as u16,
            },
            false,
        )
    }
}
