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
const GRID_WIDTH: u16 = 28;
const GRID_HEIGHT: u16 = 30;
const ARRAY_SIZE: usize = (GRID_WIDTH * GRID_HEIGHT) as usize;
pub const X_GRID_OFFSET: u16 = (SCREEN_WIDTH - GRID_WIDTH * TILE_SIZE) / 2;

// Sadly, had to remove a line as the screen was 8 pixels too short...
const MAZE_BYTES: &str = include_str!("./data/maze.txt");

#[derive(Clone, Copy)]
enum Space {
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
        grid
    }
}

const STEPS_PER_CELL: u8 = 8;
struct Player {
    grid_position: Point,
    destination: Point,
    steps: u8,
    direction: Direction,
    speed: u8,
}

impl Player {
    pub fn new() -> Self {
        Self {
            grid_position: Point { x: 1, y: 1 },
            destination: Point { x: 2, y: 1 },
            steps: 0,
            speed: 1,
            direction: Direction::Right,
        }
    }

    pub fn move_player(&mut self, grid: &Grid) {
        if self.grid_position.x == self.destination.x && self.grid_position.y == self.destination.y
        {
            return;
        }
        self.steps += self.speed;
        if self.steps >= STEPS_PER_CELL {
            // if arrived to next node
            self.grid_position = self.destination;
            // looking for new destination
            let next_pos = Point {
                x: (self.grid_position.x as i16 + self.direction.to_vector().0) as u16,
                y: (self.grid_position.y as i16 + self.direction.to_vector().1) as u16,
            };
            // if can continue, then continue else stop
            (self.destination, self.steps) = match grid
                .grid
                .get((next_pos.x + next_pos.y * GRID_WIDTH) as usize)
            {
                Some(Space::Wall) => (self.grid_position, 0),
                Some(_) => (next_pos, self.steps % STEPS_PER_CELL),
                _ => (self.grid_position, 0),
            };
        }
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
            self.direction = new_dir;
            self.grid_position = self.destination;
            self.destination = Point {
                x: (self.grid_position.x as i16 + self.direction.to_vector().0) as u16,
                y: (self.grid_position.y as i16 + self.direction.to_vector().1) as u16,
            };
            self.steps = u8::abs_diff(STEPS_PER_CELL, self.steps);
        } else if new_dir != self.direction {
            self.steps = 0;
            self.direction = new_dir;
            self.destination = Point {
                x: (self.grid_position.x as i16 + self.direction.to_vector().0) as u16,
                y: (self.grid_position.y as i16 + self.direction.to_vector().1) as u16,
            };
        }
    }
}

/// The entire game is here.
pub fn game(_exemple: bool, _high_score: &mut u32) -> u8 {
    let mut grid = Grid::new(MAZE_BYTES);
    draw_maze(MAZE_BYTES);
    let mut pac = Player::new();
    loop {
        let scan = keyboard::scan();
        if scan.key_down(key::OK) {
            break;
        }
        wait_for_vblank();
        clear_player(pac.grid_position, pac.steps, &pac.direction);
        pac.read_input(&grid);
        pac.move_player(&grid);
        draw_player(pac.grid_position, pac.steps, &pac.direction);
    }
    1
}

fn can_go_to(from: Point, dir: &Direction, grid: &Grid) -> bool {
    let next = Point {
        x: (from.x as i16 + dir.to_vector().0) as u16,
        y: (from.y as i16 + dir.to_vector().1) as u16,
    };
    match grid.grid.get((next.x + next.y * GRID_WIDTH) as usize) {
        Some(Space::Wall) => false,
        Some(_) => true,
        None => false,
    }
}
