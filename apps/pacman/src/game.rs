use heapless::{String, Vec};
use numworks_utils::{
    eadk::{
        display::{wait_for_vblank, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard,
        timing::msleep,
        Color, Point,
    },
    graphical::{draw_centered_string, draw_image, fading, ColorConfig},
    menu::{
        pause_menu, selection,
        settings::{write_values_to_file, Setting},
        start_menu, MenuConfig,
    },
    utils::{string_from_u16, LARGE_CHAR_HEIGHT},
};

use crate::{
    fruit::FruitManager,
    ghost::{Ghost, GhostType, HouseState, MovementMode},
    pac_ui::{
        clear_moveable, clear_potential_wrapping_stuff, draw_dead_pac, draw_fruit, draw_ghost,
        draw_maze, draw_player, draw_score, draw_space, TILESET_SPRITES,
    },
    player::{Player, SuperballEvent},
};

const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::WHITE,
    bckgrd: Color::BLACK,
    alt: Color::RED,
};

fn vis_addon() {
    draw_player(
        Point { x: 10, y: 10 },
        0,
        &crate::moveable::Direction::Right,
        3,
        false,
    );
    draw_ghost(
        Point { x: 15, y: 10 },
        0,
        &crate::moveable::Direction::Right,
        3,
        false,
        &GhostType::Clyde,
        HouseState::Outside,
        &MovementMode::Scatter,
    );
}

pub fn start() {
    let mut opt: [&mut Setting; 2] = [
        &mut Setting {
            name: "Starting Level\0",
            choice: 0,
            values: Vec::from_slice(&[1, 1, 19]).unwrap(),
            texts: Vec::new(),
            fixed_values: false,
            user_modifiable: true,
        },
        &mut Setting {
            name: "High-score\0",
            choice: 0,
            values: Vec::from_slice(&[0, 0, u32::MAX]).unwrap(),
            texts: Vec::new(),
            fixed_values: false,
            user_modifiable: false,
        },
    ];
    loop {
        let start = start_menu(
            "PACMAN\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./data/model_controls.txt"),
            "pacman",
        );
        if start == 0 {
            let mut high_score: u32 = opt[1].get_setting_value();
            loop {
                let action = game(opt[0].get_setting_value(), &mut high_score);
                opt[1].set_value(high_score);
                write_values_to_file(&mut opt, "pacman");
                if action == GAME_ACTION_QUIT {
                    return;
                } else if action == GAME_ACTION_MENU {
                    break;
                }
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

pub const MAZE_BYTES: &str = include_str!("./data/maze.txt");

#[derive(Clone, Copy, PartialEq)]
pub enum Space {
    Wall,
    Empty,
    Point,
    Superball,
    Fruit,
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

pub const STEPS_PER_CELL: f32 = 8.0;

const GAME_ACTION_MENU: u8 = 1;
const GAME_ACTION_QUIT: u8 = 2;

#[derive(Clone, Copy)]
struct GhostSnapshot {
    position: Point,
    steps: u8,
    house_state: HouseState,
}

impl GhostSnapshot {
    fn from_ghost(ghost: &Ghost) -> Self {
        Self {
            position: ghost.moveable.grid_position,
            steps: ghost.moveable.steps as u8,
            house_state: ghost.house_state,
        }
    }
}

fn apply_eaten_item(
    pac: &mut Player,
    grid: &mut Grid,
    score: &mut u32,
    level: u32,
    ghosts: &mut [Ghost; 4],
    fruit_manager: &mut FruitManager,
) {
    let (eaten, event) = pac.move_player(grid);

    match event {
        SuperballEvent::BlinkingStarted => {
            for ghost in ghosts.iter_mut() {
                ghost.set_frightened_blinking();
            }
        }
        SuperballEvent::Expired => {
            for ghost in ghosts.iter_mut() {
                ghost.stop_frightened();
            }
        }
        SuperballEvent::None => {}
    }

    match eaten {
        Some(Space::Superball) => {
            *score += 10;
            draw_score(*score as u16);
            for ghost in ghosts.iter_mut() {
                ghost.set_frightened();
            }
        }
        Some(Space::Point) => {
            *score += 1;
            draw_score(*score as u16);
            pac.dots_eaten += 1;
            fruit_manager.check_dot_spawn(pac.dots_eaten, grid);
        }
        Some(Space::Fruit) => {
            *score += fruit_manager.get_fruit_value(level);
            draw_score(*score as u16);
        }
        _ => (),
    }
}

fn handle_ghost_collision(pac: &Player, ghost: &mut Ghost, score: &mut u32) -> bool {
    let pac_pos = pac.moveable.grid_position;
    let pac_dest = pac.moveable.destination;
    let ghost_pos = ghost.moveable.grid_position;
    let ghost_dest = ghost.moveable.destination;

    let collision = (pac_pos.x == ghost_pos.x && pac_pos.y == ghost_pos.y)
        || (pac_pos.x == ghost_dest.x && pac_pos.y == ghost_dest.y)
        || (pac_dest.x == ghost_pos.x && pac_dest.y == ghost_pos.y)
        || (pac_dest.x == ghost_dest.x && pac_dest.y == ghost_dest.y);

    if collision {
        if ghost.movement_mode == MovementMode::Frightened
            || ghost.movement_mode == MovementMode::FrightenedBlinking
        {
            ghost.set_eaten();
            *score += 200;
            draw_score(*score as u16);
            return true;
        }
        return ghost.movement_mode == MovementMode::Eaten;
    }
    true
}

fn capture_ghost_snapshots(ghosts: &[Ghost; 4]) -> [GhostSnapshot; 4] {
    [
        GhostSnapshot::from_ghost(&ghosts[0]),
        GhostSnapshot::from_ghost(&ghosts[1]),
        GhostSnapshot::from_ghost(&ghosts[2]),
        GhostSnapshot::from_ghost(&ghosts[3]),
    ]
}

fn update_ghosts(ghosts: &mut [Ghost; 4], pac: &Player, grid: &mut Grid) {
    let blinky_pos = ghosts[0].moveable.grid_position;
    for ghost in ghosts.iter_mut() {
        ghost.update(
            pac.moveable.grid_position,
            &pac.moveable.direction,
            blinky_pos,
            grid,
        );
    }
}

fn render_frame(
    player_before: &Player,
    pac: &Player,
    ghosts: &[Ghost; 4],
    ghost_snapshots: &[GhostSnapshot; 4],
    fruit_manager: &FruitManager,
    level: u32,
    frames: u32,
    grid: &Grid,
) {
    clear_moveable(
        player_before.moveable.grid_position,
        player_before.moveable.steps as u8,
        &player_before.moveable.direction,
        grid,
        false,
    );
    for (index, ghost) in ghosts.iter().enumerate() {
        let snapshot = ghost_snapshots[index];
        clear_moveable(
            snapshot.position,
            snapshot.steps,
            &ghost.moveable.direction,
            grid,
            snapshot.house_state == HouseState::Inside,
        );
    }

    if grid[13 + 16 * GRID_WIDTH as usize] == Space::Fruit {
        draw_fruit(
            Point { x: 13, y: 16 },
            fruit_manager.get_current_fruit_type(level) as u8,
        );
    }

    for ghost in ghosts.iter() {
        draw_ghost(
            ghost.moveable.grid_position,
            ghost.moveable.steps as u8,
            &ghost.moveable.direction,
            frames,
            ghost.moveable.wrapping,
            &ghost.gtype,
            ghost.house_state,
            &ghost.movement_mode,
        );
    }
    draw_player(
        pac.moveable.grid_position,
        pac.moveable.steps as u8,
        &pac.moveable.direction,
        frames,
        pac.moveable.wrapping,
    );

    clear_potential_wrapping_stuff();
}

fn advance_game_state(
    pac: &mut Player,
    ghosts: &mut [Ghost; 4],
    grid: &mut Grid,
    score: &mut u32,
    level: u32,
    fruit_manager: &mut FruitManager,
) -> Option<usize> {
    pac.read_input(grid);
    apply_eaten_item(pac, grid, score, level, ghosts, fruit_manager);
    update_ghosts(ghosts, pac, grid);

    for (i, ghost) in ghosts.iter_mut().enumerate() {
        if !handle_ghost_collision(pac, ghost, score) {
            return Some(i);
        }
    }
    None
}

fn game_over_screen(score: u32, high_score: &mut u32) -> u8 {
    if score > *high_score {
        *high_score = score;
    }

    draw_centered_string(
        " GAME OVER! \0",
        SCREEN_HEIGHT / 3 - LARGE_CHAR_HEIGHT,
        true,
        &ColorConfig {
            text: Color::BLACK,
            bckgrd: Color::WHITE,
            alt: Color::RED,
        },
        true,
    );

    let mut score_text: String<20> = String::new();
    score_text.push_str(" Score : ").unwrap();
    score_text
        .push_str(
            string_from_u16(score as u16)
                .as_str()
                .split_terminator('\0')
                .next()
                .unwrap(),
        )
        .unwrap();
    score_text.push_str(" \0").unwrap();
    draw_centered_string(
        &score_text,
        SCREEN_HEIGHT / 3 + 2,
        true,
        &ColorConfig {
            text: Color::BLACK,
            bckgrd: Color::WHITE,
            alt: Color::RED,
        },
        true,
    );

    let action = selection(
        &ColorConfig {
            text: Color::BLACK,
            bckgrd: Color::WHITE,
            alt: Color::RED,
        },
        &MenuConfig {
            choices: &["Play again\0", "Menu\0", "Exit\0"],
            rect_margins: (20, 12),
            dimensions: (SCREEN_WIDTH * 7 / 15, LARGE_CHAR_HEIGHT * 7),
            offset: (0, 50),
            back_key_return: 2,
        },
        false,
    );
    if action != 0 {
        fading(500);
    }
    action
}

pub fn game(starting_level: u32, high_score: &mut u32) -> u8 {
    let mut grid = read_file(MAZE_BYTES);
    draw_maze();
    let mut pac = Player::new();

    let mut fruit_manager = FruitManager::new();

    let mut ghosts: [Ghost; 4] = [
        Ghost::new(Point { x: 13, y: 13 }, GhostType::Blinky),
        Ghost::new(Point { x: 14, y: 13 }, GhostType::Pinky),
        Ghost::new(Point { x: 12, y: 13 }, GhostType::Inky),
        Ghost::new(Point { x: 15, y: 13 }, GhostType::Clyde),
    ];

    let mut frames: u32 = 0;
    let mut score: u32 = 0;
    let mut level: u32 = starting_level;

    loop {
        let scan = keyboard::scan();
        if scan.key_down(key::OK) {
            let answer = pause_menu(&COLOR_CONFIG, 50);
            if answer != 0 {
                return answer;
            } else {
                draw_maze();
                let ghosts_snapshots = capture_ghost_snapshots(&ghosts);
                render_frame(
                    &pac,
                    &pac,
                    &ghosts,
                    &ghosts_snapshots,
                    &fruit_manager,
                    level,
                    frames,
                    &grid,
                );
                msleep(300);
            }
        }

        let player_before = pac;
        if let Some(i) = advance_game_state(
            &mut pac,
            &mut ghosts,
            &mut grid,
            &mut score,
            level,
            &mut fruit_manager,
        ) {
            draw_dead_pac(&pac, &ghosts[i], &grid);
            msleep(500);
            return game_over_screen(score, high_score);
        }

        let ghost_snapshots = capture_ghost_snapshots(&ghosts);
        wait_for_vblank();
        render_frame(
            &player_before,
            &pac,
            &ghosts,
            &ghost_snapshots,
            &fruit_manager,
            level,
            frames,
            &grid,
        );

        fruit_manager.update(&mut grid);
        frames = frames.wrapping_add(1);
    }
}
