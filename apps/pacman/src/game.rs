use heapless::{String, Vec};
use numworks_utils::{
    eadk::{
        display::{wait_for_vblank, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard,
        timing::{self, msleep},
        Color, Point,
    },
    graphical::{draw_centered_string, fading, fill_screen, ColorConfig},
    menu::{
        pause_menu, selection,
        settings::{write_values_to_file, Setting},
        start_menu, MenuConfig,
    },
    utils::{string_from_u32, LARGE_CHAR_HEIGHT},
};

use crate::{
    ghost::{Ghost, GhostType, HouseState, MovementMode},
    levels::{LevelConfig, LevelManager, TOTAL_DOTS},
    pac_ui::{
        clear_moveable, clear_potential_wrapping_stuff, draw_constant_ui, draw_dead_pac,
        draw_fruit, draw_ghost, draw_level, draw_lives, draw_maze, draw_player, draw_score,
        draw_space,
    },
    player::{Player, SuperballEvent},
};

const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::WHITE,
    bckgrd: Color::BLACK,
    alt: Color::RED,
};

fn vis_addon() {
    for x in 0..=4 {
        draw_space(
            Point {
                x: X_GRID_OFFSET + 8 * TILE_SIZE + x * TILE_SIZE as u16,
                y: 8 * TILE_SIZE,
            },
            Space::Point,
        );
    }
    draw_space(
        Point {
            x: 13 * TILE_SIZE + X_GRID_OFFSET,
            y: 8 * TILE_SIZE,
        },
        Space::Superball,
    );
    draw_fruit(Point { x: 20, y: 8 }, 0);
    draw_player(
        Point { x: 6, y: 8 },
        0,
        crate::moveable::Direction::Right,
        3,
    );
    draw_ghost(
        Point { x: 16, y: 8 },
        0,
        crate::moveable::Direction::Left,
        3,
        &GhostType::Blinky,
        HouseState::Outside,
        &MovementMode::Scatter,
    );
    draw_ghost(
        Point { x: 2, y: 8 },
        0,
        crate::moveable::Direction::Right,
        3,
        &GhostType::Clyde,
        HouseState::Outside,
        &MovementMode::Scatter,
    );
}

pub fn start() {
    let mut opt: [&mut Setting; 5] = [
        &mut Setting {
            name: "Game speed\0",
            choice: 1, // 0.85x, 1.0x, 1.15x
            values: Vec::from_slice(&[0, 1, 2]).unwrap(),
            texts: Vec::from_slice(&["Slow\0", "Normal\0", "Fast\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "Starting lives\0",
            choice: 2, // Default: 3
            values: Vec::from_slice(&[1, 2, 3]).unwrap(),
            texts: Vec::from_slice(&["1\0", "2\0", "3\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "No collisions\0",
            choice: 0,
            values: Vec::from_slice(&[0, 1]).unwrap(),
            texts: Vec::from_slice(&["No\0", "Yes (CHEAT)\0"]).unwrap(),
            user_modifiable: false,
            fixed_values: true,
        },
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
            loop {
                let speed_mult = [0.60, 1.0, 1.30][(opt[0].get_setting_value()) as usize];
                let initial_lives = opt[1].get_setting_value() as u8;
                let god_mode = opt[2].get_setting_value() != 0;
                let starting_level = opt[3].get_setting_value();
                let mut high_score = opt[4].get_setting_value();

                let action = game(
                    speed_mult,
                    initial_lives,
                    god_mode,
                    starting_level,
                    &mut high_score,
                );

                opt[4].set_value(high_score);
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

pub(crate) fn grid_index(pos: Point) -> usize {
    (pos.x + pos.y * GRID_WIDTH) as usize
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

fn apply_speed_multiplier(config: &mut LevelConfig, mult: f32) {
    config.pac_speed *= mult;
    config.ghost_speed *= mult;
    config.frightened_ghost_speed *= mult;
    config.tunnel_ghost_speed *= mult;
    config.eaten_ghost_speed *= mult;
}

fn make_ghosts(config: &LevelConfig) -> [Ghost; 4] {
    [
        Ghost::new(Point { x: 13, y: 13 }, GhostType::Blinky, config),
        Ghost::new(Point { x: 14, y: 13 }, GhostType::Pinky, config),
        Ghost::new(Point { x: 12, y: 13 }, GhostType::Inky, config),
        Ghost::new(Point { x: 15, y: 13 }, GhostType::Clyde, config),
    ]
}

fn reset_level_positions(pac: &mut Player, ghosts: &mut [Ghost; 4], level_mgr: &LevelManager) {
    let saved_dots = pac.dots_eaten;
    *pac = Player::new(&level_mgr.config);
    pac.dots_eaten = saved_dots;

    *ghosts = make_ghosts(&level_mgr.config);
}

fn apply_eaten_item(
    pac: &mut Player,
    grid: &mut Grid,
    score: &mut u32,
    ghosts: &mut [Ghost; 4],
    level_mgr: &mut LevelManager,
) {
    let (eaten, event) = pac.move_player(&level_mgr.config, grid);

    match event {
        SuperballEvent::BlinkingStarted => {
            for ghost in ghosts.iter_mut() {
                ghost.set_frightened_blinking();
            }
        }
        SuperballEvent::Expired => {
            for ghost in ghosts.iter_mut() {
                ghost.stop_frightened(&level_mgr.config);
            }
        }
        SuperballEvent::None => {}
    }

    match eaten {
        Some(Space::Superball) => {
            *score += 50;
            for ghost in ghosts.iter_mut() {
                ghost.set_frightened(&level_mgr.config);
            }
        }
        Some(Space::Point) => {
            *score += 10;
            pac.dots_eaten += 1;
            level_mgr.check_fruit_spawn(pac.dots_eaten, timing::millis(), grid);
        }
        Some(Space::Fruit) => {
            let fruit_points = level_mgr.collect_fruit(grid);
            *score += fruit_points;
        }
        _ => (),
    }
}

fn handle_ghost_collision(
    pac: &Player,
    ghost: &mut Ghost,
    score: &mut u32,
    config: &LevelConfig,
    god_mode: bool,
) -> bool {
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
            ghost.set_eaten(config);
            *score += 200;
            draw_score(*score);
            return true;
        }
        if god_mode {
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

fn update_ghosts(ghosts: &mut [Ghost; 4], pac: &Player, grid: &mut Grid, level_mgr: &LevelManager) {
    let blinky_pos = ghosts[0].moveable.grid_position;
    for ghost in ghosts.iter_mut() {
        ghost.update(
            pac.moveable.grid_position,
            &pac.moveable.direction,
            blinky_pos,
            grid,
            &level_mgr.config,
        );
    }
}

struct FrameRenderContext<'a> {
    player_before: &'a Player,
    pac: &'a Player,
    ghosts: &'a [Ghost; 4],
    ghost_snapshots: &'a [GhostSnapshot; 4],
    level_mgr: &'a LevelManager,
    frames: u32,
    score: u32,
    grid: &'a Grid,
}

impl<'a> FrameRenderContext<'a> {
    fn new(
        player_before: &'a Player,
        pac: &'a Player,
        ghosts: &'a [Ghost; 4],
        ghost_snapshots: &'a [GhostSnapshot; 4],
        level_mgr: &'a LevelManager,
        frames: u32,
        score: u32,
        grid: &'a Grid,
    ) -> Self {
        Self {
            player_before,
            pac,
            ghosts,
            ghost_snapshots,
            level_mgr,
            frames,
            score,
            grid,
        }
    }

    fn render(&self) {
        clear_moveable(
            self.player_before.moveable.grid_position,
            self.player_before.moveable.steps as u8,
            self.player_before.moveable.direction,
            self.grid,
            false,
        );
        for (index, ghost) in self.ghosts.iter().enumerate() {
            let snapshot = self.ghost_snapshots[index];
            clear_moveable(
                snapshot.position,
                snapshot.steps,
                ghost.moveable.direction,
                self.grid,
                snapshot.house_state == HouseState::Inside,
            );
        }

        if self.level_mgr.fruit_active {
            draw_fruit(
                self.level_mgr.fruit_spawn_pos,
                self.level_mgr.config.fruit_id,
            );
        }

        for ghost in self.ghosts.iter() {
            draw_ghost(
                ghost.moveable.grid_position,
                ghost.moveable.steps as u8,
                ghost.moveable.direction,
                self.frames,
                &ghost.gtype,
                ghost.house_state,
                &ghost.movement_mode,
            );
        }
        draw_player(
            self.pac.moveable.grid_position,
            self.pac.moveable.steps as u8,
            self.pac.moveable.direction,
            self.frames,
        );

        clear_potential_wrapping_stuff();
        draw_score(self.score);
    }
}

fn advance_game_state(
    pac: &mut Player,
    ghosts: &mut [Ghost; 4],
    grid: &mut Grid,
    score: &mut u32,
    level_mgr: &mut LevelManager,
    god_mode: bool,
) -> Option<usize> {
    pac.read_input(grid);
    apply_eaten_item(pac, grid, score, ghosts, level_mgr);
    update_ghosts(ghosts, pac, grid, level_mgr);

    for (i, ghost) in ghosts.iter_mut().enumerate() {
        if !handle_ghost_collision(pac, ghost, score, &level_mgr.config, god_mode) {
            return Some(i);
        }
    }
    None
}

fn draw_hud(high_score: u32, level: u16, lives: u8) {
    draw_constant_ui(high_score);
    draw_level(level);
    draw_lives(lives);
}

fn game_over_screen(score: u32, high_score: &mut u32) -> u8 {
    if score > *high_score {
        *high_score = score;
    }

    draw_centered_string(
        " GAME OVER! \0",
        SCREEN_HEIGHT / 3 - LARGE_CHAR_HEIGHT,
        true,
        &COLOR_CONFIG,
        true,
    );

    let mut score_text: String<20> = String::new();
    score_text.push_str(" Score : ").unwrap();
    score_text
        .push_str(
            string_from_u32(score)
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
        &COLOR_CONFIG,
        true,
    );

    let action = selection(
        &COLOR_CONFIG,
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

pub fn game(
    speed_mult: f32,
    initial_lives: u8,
    god_mode: bool,
    starting_level: u32,
    high_score: &mut u32,
) -> u8 {
    let mut grid = read_file(MAZE_BYTES);

    let mut level_mgr = LevelManager::new(starting_level as u16);
    apply_speed_multiplier(&mut level_mgr.config, speed_mult);

    let mut pac = Player::new(&level_mgr.config);

    let mut ghosts = make_ghosts(&level_mgr.config);

    let mut frames: u32 = 0;
    let mut score: u32 = 0;

    let mut lives: u8 = initial_lives;
    let mut extra_life_awarded = false;

    fill_screen(Color::BLACK);
    draw_maze(&grid);
    draw_hud(*high_score, level_mgr.current_level, lives);

    loop {
        if !extra_life_awarded && score >= 10000 {
            lives += 1;
            extra_life_awarded = true;
            draw_lives(lives);
        }

        let scan = keyboard::scan();
        if scan.key_down(key::OK) {
            let answer = pause_menu(&COLOR_CONFIG, 50);
            if answer != 0 {
                return answer;
            }

            draw_maze(&grid);
            let ghosts_snapshots = capture_ghost_snapshots(&ghosts);
            FrameRenderContext::new(
                &pac,
                &pac,
                &ghosts,
                &ghosts_snapshots,
                &level_mgr,
                frames,
                score,
                &grid,
            )
            .render();
            msleep(300);
        }

        let now = timing::millis();
        level_mgr.update(now, &mut grid);

        let player_before = pac;
        let ghost_snapshots: [GhostSnapshot; 4] = capture_ghost_snapshots(&ghosts);

        if let Some(i) = advance_game_state(
            &mut pac,
            &mut ghosts,
            &mut grid,
            &mut score,
            &mut level_mgr,
            god_mode,
        ) {
            draw_dead_pac(&pac, &ghosts[i], &grid);
            msleep(1000);

            lives = lives.saturating_sub(1);
            draw_lives(lives);

            if lives > 0 {
                reset_level_positions(&mut pac, &mut ghosts, &level_mgr);

                draw_maze(&grid);
                draw_hud(*high_score, level_mgr.current_level, lives);

                let ghost_snapshots = capture_ghost_snapshots(&ghosts);
                FrameRenderContext::new(
                    &pac,
                    &pac,
                    &ghosts,
                    &ghost_snapshots,
                    &level_mgr,
                    frames,
                    score,
                    &grid,
                )
                .render();
                msleep(1500);
                continue;
            } else {
                return game_over_screen(score, high_score);
            }
        }

        // --- LEVEL VICTORY / TRANSITION CHECK ---
        if pac.dots_eaten >= TOTAL_DOTS {
            msleep(250);

            draw_centered_string(
                "Level complete!\0",
                SCREEN_HEIGHT / 2 - LARGE_CHAR_HEIGHT,
                true,
                &COLOR_CONFIG,
                false,
            );

            msleep(750);

            // Advance level config and re-apply global speed multiplier
            level_mgr.advance_level();
            apply_speed_multiplier(&mut level_mgr.config, speed_mult);

            grid = read_file(MAZE_BYTES);
            pac = Player::new(&level_mgr.config);

            // Reset ghosts with new LevelConfig timings and speeds
            ghosts = make_ghosts(&level_mgr.config);

            // Redraw screen, HUD, and new level indicator
            draw_maze(&grid);
            draw_hud(*high_score, level_mgr.current_level, lives);

            let new_snapshots = capture_ghost_snapshots(&ghosts);
            FrameRenderContext::new(
                &pac,
                &pac,
                &ghosts,
                &new_snapshots,
                &level_mgr,
                frames,
                score,
                &grid,
            )
            .render();

            msleep(1500);
            continue;
        }

        wait_for_vblank();
        FrameRenderContext::new(
            &player_before,
            &pac,
            &ghosts,
            &ghost_snapshots,
            &level_mgr,
            frames,
            score,
            &grid,
        )
        .render();

        frames = frames.wrapping_add(1);
    }
}
