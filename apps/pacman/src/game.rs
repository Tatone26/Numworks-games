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

use crate::{
    ghost::Ghost,
    moveable::{can_go_to, Direction, Moveable},
    pac_ui::{clear_ghost, draw_ghost},
};
use crate::{
    ghost::GhostType,
    pac_ui::{clear_player, draw_maze, draw_player, draw_score},
};

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

/// Represents the type of the cell.
#[derive(Clone, Copy)]
pub enum Space {
    Wall,
    Empty,
    Point,
    Superball,
    Fruit,
}

pub type Grid = [Space; ARRAY_SIZE];

/// Reads the maze file, given as text. Returns a [Space; ARRAY_SIZE]
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

pub const STEPS_PER_CELL: f32 = 8.0;

#[derive(Clone, Copy)]
struct Player {
    moveable: Moveable,
    superball_active: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            moveable: Moveable::new(
                Point { x: 13, y: 22 },
                Point { x: 14, y: 22 },
                Direction::Right,
                1.0,
            ),
            superball_active: false,
        }
    }

    /// Returns the type of thing that has been eaten, Empty if none
    pub fn move_player(&mut self, grid: &mut Grid) -> Option<Space> {
        let on = self.moveable.move_moveable(grid, false);
        let eaten = match on {
            Space::Superball => {
                if self.moveable.steps >= 3.0 * (STEPS_PER_CELL / 4.0) {
                    grid[(self.moveable.destination.x + self.moveable.destination.y * GRID_WIDTH)
                        as usize] = Space::Empty;
                    self.superball_active = true;
                    Some(Space::Superball)
                } else {
                    None
                }
            }
            Space::Point => {
                if self.moveable.steps >= STEPS_PER_CELL / 4.0 {
                    grid[(self.moveable.destination.x + self.moveable.destination.y * GRID_WIDTH)
                        as usize] = Space::Empty;
                    Some(Space::Point)
                } else {
                    None
                }
            }
            _ => None,
        };
        eaten
    }

    pub fn read_input(&mut self, grid: &Grid) {
        let scan = keyboard::scan();
        let new_dir = if scan.key_down(key::UP)
            && can_go_to(self.moveable.grid_position, &Direction::Up, grid)
        {
            Direction::Up
        } else if scan.key_down(key::DOWN)
            && can_go_to(self.moveable.grid_position, &Direction::Down, grid)
        {
            Direction::Down
        } else if scan.key_down(key::RIGHT)
            && can_go_to(self.moveable.grid_position, &Direction::Right, grid)
        {
            Direction::Right
        } else if scan.key_down(key::LEFT)
            && can_go_to(self.moveable.grid_position, &Direction::Left, grid)
        {
            Direction::Left
        } else {
            self.moveable.direction
        };
        self.moveable.change_direction(new_dir);
    }
}

/// The entire game is here.
pub fn game(_exemple: bool, _high_score: &mut u32) -> u8 {
    let mut grid = read_file(MAZE_BYTES);
    draw_maze(MAZE_BYTES);
    let mut pac = Player::new();

    let mut ghosts: [Ghost; 4] = [
        Ghost::new(Point { x: 13, y: 13 }, GhostType::Blinky),
        Ghost::new(Point { x: 14, y: 13 }, GhostType::Pinky),
        Ghost::new(Point { x: 12, y: 13 }, GhostType::Inky),
        Ghost::new(Point { x: 15, y: 13 }, GhostType::Clyde),
    ];

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
            Some(Space::Superball) => {
                score += 10;
                draw_score(score as u16);
                pac.superball_active = true;
                pac.moveable.speed = 1.5;
            }
            Some(Space::Point) => {
                score += 1;
                draw_score(score as u16);
            }
            // TODO : fruits
            _ => (),
        }

        let blinky_pos = ghosts[0].moveable.grid_position;
        let old_ghosts: [(Point, u8, bool); 4] = [
            (
                ghosts[0].moveable.grid_position,
                ghosts[0].moveable.steps as u8,
                ghosts[0].is_home,
            ),
            (
                ghosts[1].moveable.grid_position,
                ghosts[1].moveable.steps as u8,
                ghosts[1].is_home,
            ),
            (
                ghosts[2].moveable.grid_position,
                ghosts[2].moveable.steps as u8,
                ghosts[2].is_home,
            ),
            (
                ghosts[3].moveable.grid_position,
                ghosts[3].moveable.steps as u8,
                ghosts[3].is_home,
            ),
        ];

        for g in ghosts.iter_mut() {
            g.update(
                pac.moveable.grid_position,
                &pac.moveable.direction,
                blinky_pos,
                &mut grid,
            );
        }

        wait_for_vblank();
        // the player will always change position or direction (speed >= 1.0)
        clear_player(
            old.moveable.grid_position,
            old.moveable.steps as u8,
            &old.moveable.direction,
            &grid,
        );
        draw_player(
            pac.moveable.grid_position,
            pac.moveable.steps as u8,
            &pac.moveable.direction,
            frames,
            pac.moveable.wrapping,
        );
        for (i, g) in ghosts.iter().enumerate() {
            clear_ghost(
                old_ghosts[i].0,
                old_ghosts[i].1,
                &g.moveable.direction,
                &grid,
                old_ghosts[i].2,
            );
        }
        for g in ghosts.iter() {
            draw_ghost(
                g.moveable.grid_position,
                g.moveable.steps as u8,
                &g.moveable.direction,
                frames,
                g.moveable.wrapping,
                &g.gtype,
                g.is_home,
            );
        }

        frames = frames.wrapping_add(1);
    }
    1
}
