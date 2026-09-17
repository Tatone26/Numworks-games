use heapless::Vec;
use num_engine::{
    define_scratch_buffer, define_tilemap_buffer,
    engine::InterlaceTuning,
    graphics::{
        compositor::InterlaceMode,
        tilemap::{Parallax, WrapMode},
    },
    init_engine_window, inset_hitbox,
    world::World,
    world_spawn, world_tilemap, Engine,
};
use numworks_utils::{
    eadk::{
        display::{push_rect_uniform, SCREEN_HEIGHT},
        key, keyboard, timing, Color,
    },
    graphical::{draw_centered_string, fill_screen, ColorConfig},
    menu::{
        pause_menu, selection,
        settings::{write_values_to_file, Setting},
        start_menu_with_save, MenuConfig, StartMenuAction,
    },
    storage::save::{delete_save, write_save, GameSave, SaveField, SaveSlotMode, MAX_SAVE_FIELDS},
    storage::MAX_STORAGE_VALUES,
    utils::{CENTER, LARGE_CHAR_HEIGHT},
};

use crate::{
    ghost::{Ghost, GhostType, HouseState, MovementMode},
    levels::LevelManager,
    pac_ui::{
        draw_constant_ui, draw_level, draw_lives, draw_score, fruit_anim, menu_vis_addon,
        ANIM_PAC_DEAD, ANIM_PAC_RIGHT, TILESET_WALLS,
    },
    player::{Player, SuperballEvent},
};

pub const TILE_SIZE: usize = 8;
pub const CELL_AREA: usize = 64;
pub const WINDOW_COLS: usize = 28;
pub const WINDOW_ROWS: usize = 30;

pub const GRID_WIDTH: usize = WINDOW_COLS;
pub const GRID_HEIGHT: usize = WINDOW_ROWS;
pub const ARRAY_SIZE: usize = GRID_WIDTH * GRID_HEIGHT;

pub const VIEW_SCREEN_X: u16 = (320 - (WINDOW_COLS * TILE_SIZE) as u16) / 2; // 48
pub const VIEW_SCREEN_Y: u16 = 0;
pub const VIEW_SCREEN_W: u16 = (WINDOW_COLS * TILE_SIZE) as u16; // 224
pub const VIEW_SCREEN_H: u16 = (WINDOW_ROWS * TILE_SIZE) as u16; // 240

pub const MAZE_BYTES: &str = include_str!("./data/maze.txt");

const DEBUG_MODE: bool = false;
const WORLD_ENT_CAP: usize = 16;
const WORLD_MAP_CAP: usize = 2;
const WORLD_PARTS_PER_ENT: usize = 2;
const WORLD_RENDER_CAP: usize = 32;
const WORLD_PARTICLE_CAP: usize = 8;
const WORLD_PARTICLE_SYS_CAP: usize = 1;

pub type GameEngine<'s> =
    Engine<'s, TILE_SIZE, CELL_AREA, WINDOW_COLS, WINDOW_ROWS, WORLD_RENDER_CAP, DEBUG_MODE>;
pub type GameWorld<'a> = World<
    'a,
    TILE_SIZE,
    CELL_AREA,
    WORLD_ENT_CAP,
    WORLD_MAP_CAP,
    WORLD_PARTS_PER_ENT,
    WORLD_RENDER_CAP,
    WORLD_PARTICLE_CAP,
    WORLD_PARTICLE_SYS_CAP,
>;

define_scratch_buffer!(SCRATCH_BUFFER, CELL_AREA, WINDOW_COLS, WINDOW_ROWS);
define_tilemap_buffer!(MAZE_DATA, WINDOW_COLS, WINDOW_ROWS);

pub const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::WHITE,
    bckgrd: Color::BLACK,
    alt: Color::RED,
};

const READY_COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::from_rgb888(255, 255, 0),
    bckgrd: Color::BLACK,
    alt: Color::BLACK,
};

const COMPLETE_COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::from_rgb888(0, 255, 255),
    bckgrd: Color::BLACK,
    alt: Color::BLACK,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Space {
    Wall,
    Empty,
    Point,
    Superball,
    Fruit,
}

pub type Grid = [Space; ARRAY_SIZE];

pub const EMPTY_TILE: [u8; 2] = [12, 2];

pub fn char_to_tile(c: char) -> [u8; 2] {
    match c {
        '.' => [13, 2],
        '°' => [15, 2],
        ' ' => [12, 2],
        '0'..='9' => [c as u8 - b'0', 0],
        'a'..='f' => [(c as u8 - b'a') + 10, 0],
        'g'..='v' => [c as u8 - b'g', 1],
        'w'..='z' => [c as u8 - b'w', 2],
        'A'..='L' => [(c as u8 - b'A') + 4, 2],
        _ => [12, 2],
    }
}

pub fn load_maze_into_buffers(maze_str: &str, grid: &mut Grid) -> u16 {
    let buf = unsafe { &mut *(&raw mut MAZE_DATA) };
    buf.fill(Some(EMPTY_TILE));
    grid.fill(Space::Empty);

    let mut dot_count = 0;

    for (row, line) in maze_str
        .lines()
        .filter(|s| !s.is_empty())
        .enumerate()
        .take(WINDOW_ROWS)
    {
        for (col, ch) in line.chars().enumerate().take(WINDOW_COLS) {
            let idx = row * WINDOW_COLS + col;
            buf[idx] = Some(char_to_tile(ch));
            grid[idx] = match ch {
                '.' => {
                    dot_count += 1;
                    Space::Point
                }
                '°' => {
                    dot_count += 1;
                    Space::Superball
                }
                ' ' => Space::Empty,
                _ => Space::Wall,
            };
        }
    }

    dot_count
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacmanSave {
    pub level: u16,
    pub score: u32,
    pub lives: u8,
}

impl GameSave for PacmanSave {
    fn to_words(&self) -> Vec<u32, MAX_STORAGE_VALUES> {
        let mut v = Vec::new();
        let _ = v.push(self.level as u32);
        let _ = v.push(self.score);
        let _ = v.push(self.lives as u32);
        v
    }

    fn from_words(words: &[u32]) -> Option<Self> {
        if words.len() >= 3 {
            Some(Self {
                level: words[0] as u16,
                score: words[1],
                lives: words[2] as u8,
            })
        } else {
            None
        }
    }

    fn describe_fields(&self) -> Vec<SaveField, MAX_SAVE_FIELDS> {
        let mut fields = Vec::new();
        let _ = fields.push(SaveField {
            name: "Save: Level\0",
            min: 1,
            max: 255,
            value: self.level as u32,
        });
        let _ = fields.push(SaveField {
            name: "Save: Score\0",
            min: 0,
            max: u32::MAX,
            value: self.score,
        });
        let _ = fields.push(SaveField {
            name: "Save: Lives\0",
            min: 1,
            max: 5,
            value: self.lives as u32,
        });
        fields
    }

    fn update_from_fields(&mut self, fields: &[SaveField]) {
        if fields.len() >= 3 {
            self.level = fields[0].value.max(1) as u16;
            self.score = fields[1].value;
            self.lives = fields[2].value.clamp(1, 5) as u8;
        }
    }
}

pub fn start() {
    let mut opt: [&mut Setting; 6] = [
        // 0: Game Speed
        &mut Setting {
            name: "Game speed\0",
            choice: 1,
            values: Vec::from_slice(&[0, 1, 2]).unwrap(),
            texts: Vec::from_slice(&["Slow\0", "Normal\0", "Fast\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 1: Starting Lives
        &mut Setting {
            name: "Starting lives\0",
            choice: 2,
            values: Vec::from_slice(&[1, 2, 3]).unwrap(),
            texts: Vec::from_slice(&["1\0", "2\0", "3\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 2: Input Buffer Window
        &mut Setting {
            name: "Input buffer\0",
            choice: 2,
            values: Vec::from_slice(&[0, 6, 15, 25, 255]).unwrap(),
            texts: Vec::from_slice(&[
                "Strict\0",
                "Snappy\0",
                "Normal\0",
                "Forgiving\0",
                "Infinite\0",
            ])
            .unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 3: Starting Level Slider
        &mut Setting {
            name: "Starting Level\0",
            choice: 1,
            values: Vec::from_slice(&[1, 1, 19]).unwrap(),
            texts: Vec::new(),
            fixed_values: false,
            user_modifiable: true,
        },
        // 4: God Mode Cheat
        &mut Setting {
            name: "No collisions\0",
            choice: 0,
            values: Vec::from_slice(&[0, 1]).unwrap(),
            texts: Vec::from_slice(&["No\0", "Yes (CHEAT)\0"]).unwrap(),
            user_modifiable: false,
            fixed_values: true,
        },
        // 5: Persistent High Score
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
        let menu_action: StartMenuAction<PacmanSave> = start_menu_with_save(
            "PACMAN\0",
            &mut opt,
            &COLOR_CONFIG,
            menu_vis_addon,
            include_str!("./data/model_controls.txt"),
            "pacman",
            SaveSlotMode::Single,
        );

        match menu_action {
            StartMenuAction::Continue(save, _) => {
                run_session(&mut opt, save.level, save.score, save.lives);
            }
            StartMenuAction::NewGame(_) => {
                let starting_level = (opt[3].get_setting_value() as u16).max(1);
                let initial_lives = opt[1].get_setting_value() as u8;
                run_session(&mut opt, starting_level, 0, initial_lives);
            }
            StartMenuAction::Exit => return,
        }
    }
}

fn run_session(opt: &mut [&mut Setting; 6], start_lvl: u16, start_score: u32, start_lives: u8) {
    let mut current_lvl = start_lvl;
    let mut current_score = start_score;
    let mut current_lives = start_lives;

    loop {
        let speed_mult = [0.80, 1.0, 1.25][opt[0].get_setting_value() as usize];
        let input_buffer_frames = opt[2].get_setting_value() as u8;
        let god_mode = opt[4].choice == 1;
        let mut high_score = opt[5].get_setting_value();

        let action = game(
            speed_mult,
            current_lives,
            input_buffer_frames,
            god_mode,
            current_lvl,
            current_score,
            &mut high_score,
            &mut current_lvl,
            &mut current_score,
            &mut current_lives,
        );

        opt[5].set_value(high_score);
        write_values_to_file(opt, "pacman");

        if action == 2 {
            return;
        } else if action == 1 {
            break;
        }
    }
}

pub fn game(
    speed_mult: f32,
    initial_lives: u8,
    input_buffer_frames: u8,
    god_mode: bool,
    starting_level: u16,
    starting_score: u32,
    high_score: &mut u32,
    out_level: &mut u16,
    out_score: &mut u32,
    out_lives: &mut u8,
) -> u8 {
    let mut grid: Grid = [Space::Empty; ARRAY_SIZE];
    let _ = load_maze_into_buffers(MAZE_BYTES, &mut grid);

    let mut level_mgr = LevelManager::new(starting_level);
    level_mgr.config.pac_speed *= speed_mult;
    level_mgr.config.pac_dot_speed *= speed_mult;
    level_mgr.config.ghost_speed *= speed_mult;
    level_mgr.config.frightened_ghost_speed *= speed_mult;
    level_mgr.config.tunnel_ghost_speed *= speed_mult;
    level_mgr.config.eaten_ghost_speed *= speed_mult;

    let mut engine: GameEngine = init_engine_window!(
        color: Color::BLACK,
        window: [VIEW_SCREEN_X, VIEW_SCREEN_Y, VIEW_SCREEN_W, VIEW_SCREEN_H],
        interlace: Some(InterlaceMode::Columns),
        scratch: SCRATCH_BUFFER,
    );
    engine.set_interlace_tuning(InterlaceTuning {
        target_budget_us: 20_000,
        hysteresis_us: 2_000,
        tripwire_frames: 1,
        cooldown_frames: 1,
    });

    let mut world: GameWorld = GameWorld::new();

    // 1. Maze Tilemap Layer (z = 0)
    let maze_map_id = world_tilemap!(
        world: world,
        tileset: &TILESET_WALLS,
        buffer: MAZE_DATA,
        cols: WINDOW_COLS,
        rows: WINDOW_ROWS,
        z: 0,
        transparent: false,
        parallax: Parallax::FIXED,
        wrap: WrapMode::None,
    )
    .unwrap();

    // 2. Pac-Man Entity (z = 14)
    let mut pac = Player::new(&level_mgr.config);
    pac.buffer_window_frames = input_buffer_frames;
    pac.entity_id = world_spawn!(
        world: world,
        anim: &ANIM_PAC_RIGHT,
        pos: [100.0, 172.0],
        z: 14,
        hitbox: inset_hitbox!(16, 16, 4),
    )
    .unwrap();

    // 3. Four Ghosts (z = 10)
    let mut ghosts = [
        Ghost::new(GhostType::Blinky, &level_mgr.config),
        Ghost::new(GhostType::Pinky, &level_mgr.config),
        Ghost::new(GhostType::Inky, &level_mgr.config),
        Ghost::new(GhostType::Clyde, &level_mgr.config),
    ];

    for g in ghosts.iter_mut() {
        let spawn_pos = g.spawn_coords();
        g.entity_id = world_spawn!(
            world: world,
            anim: g.current_animation(),
            pos: [spawn_pos[0], spawn_pos[1]],
            z: 10,
            hitbox: inset_hitbox!(16, 16, 4),
        )
        .unwrap();
    }

    // 4. Bonus Fruit (z = 5)
    let fruit_id = world_spawn!(
        world: world,
        anim: fruit_anim(level_mgr.config.fruit_id),
        pos: [100.0, 124.0],
        z: 5,
    )
    .unwrap();
    world[fruit_id].active = false;

    let mut score: u32 = starting_score;
    let mut lives: u8 = initial_lives;
    let mut ghost_multiplier: u32 = 1;

    fill_screen(Color::BLACK);
    draw_constant_ui(*high_score);
    draw_score(score);
    draw_level(level_mgr.current_level);
    draw_lives(lives);

    // Initial "READY!" Preview
    engine.mark_all_dirty();
    engine.render(&mut world);
    draw_centered_string(
        "READY!\0",
        CENTER.y - LARGE_CHAR_HEIGHT / 2,
        true,
        &READY_COLOR_CONFIG,
        false,
    );
    timing::msleep(1500);
    engine.mark_all_dirty();

    'gameloop: loop {
        let scan = keyboard::scan();
        if scan.key_down(key::BACK) || scan.key_down(key::HOME) {
            let act = pause_menu(&COLOR_CONFIG, 50);
            if act != 0 {
                *out_level = level_mgr.current_level;
                *out_score = score;
                *out_lives = lives;
                return act;
            }
            fill_screen(Color::BLACK);
            draw_constant_ui(*high_score);
            draw_score(score);
            draw_level(level_mgr.current_level);
            draw_lives(lives);
            engine.mark_all_dirty();
            continue;
        }

        let frame = engine.get_frame();
        let now = timing::millis();

        // 1. Fruit Lifetime Tick
        if level_mgr.update(now, &mut grid) {
            world[fruit_id].active = false;
        }

        // 2. Prepare entity kinetics on engine
        pac.read_input();
        pac.prepare_kinetics(&mut world, &level_mgr.config);

        for g in ghosts.iter_mut() {
            g.prepare_kinetics(&mut world, &level_mgr.config);
        }

        // 3. Engine Kinematic Integration & Animations
        world.update(frame);

        // 4. Post-Physics Corridor Alignments & Items
        let (eaten, event) = pac.post_physics(&mut world, &mut grid, &level_mgr.config);

        match event {
            SuperballEvent::BlinkingStarted => {
                for g in ghosts.iter_mut() {
                    g.set_frightened_blinking();
                }
            }
            SuperballEvent::Expired => {
                for g in ghosts.iter_mut() {
                    g.stop_frightened(&level_mgr.config);
                }
                ghost_multiplier = 1;
            }
            SuperballEvent::None => {}
        }

        if let Some((tx, ty, item)) = eaten {
            world
                .tilemap_mut(maze_map_id)
                .set_tile(tx, ty, Some(EMPTY_TILE));
            match item {
                Space::Point => {
                    score += 10;
                    if level_mgr.check_fruit_spawn(pac.dots_eaten, now, &mut grid) {
                        world[fruit_id].set_animation(fruit_anim(level_mgr.config.fruit_id));
                        world[fruit_id].set_pos(100.0, 124.0);
                        world[fruit_id].active = true;
                    }
                }
                Space::Superball => {
                    score += 50;
                    ghost_multiplier = 1;
                    if level_mgr.config.frightened_duration_ms > 0 {
                        for g in ghosts.iter_mut() {
                            g.set_frightened(&level_mgr.config);
                        }
                    } else {
                        for g in ghosts.iter_mut() {
                            if g.house_state == HouseState::Outside
                                && g.movement_mode != MovementMode::Eaten
                            {
                                g.dir = g.dir.opposite();
                                g.last_turn_node = (-1, -1);
                            }
                        }
                    }
                }
                Space::Fruit => {
                    score += level_mgr.collect_fruit(&mut grid);
                    world[fruit_id].active = false;
                }
                _ => {}
            }
            draw_score(score);
        }

        // Ghost AI Decision Making
        let pac_tx = (((world[pac.entity_id].x() + 4.0) / 8.0) + 0.5) as i16;
        let pac_ty = (((world[pac.entity_id].y() + 4.0) / 8.0) + 0.5) as i16;
        let blinky_tx = (((world[ghosts[0].entity_id].x() + 4.0) / 8.0) + 0.5) as i16;
        let blinky_ty = (((world[ghosts[0].entity_id].y() + 4.0) / 8.0) + 0.5) as i16;

        for g in ghosts.iter_mut() {
            g.post_physics(
                &mut world, pac_tx, pac_ty, pac.dir, blinky_tx, blinky_ty, &grid,
            );
        }

        // 5. Collisions
        for i in 0..ghosts.len() {
            let g = &mut ghosts[i];

            if g.house_state != HouseState::Outside {
                continue;
            }

            if world.collides(pac.entity_id, g.entity_id) {
                if matches!(
                    g.movement_mode,
                    MovementMode::Frightened | MovementMode::FrightenedBlinking
                ) {
                    g.set_eaten(&level_mgr.config);
                    score += 200 * ghost_multiplier;
                    ghost_multiplier *= 2;
                    draw_score(score);
                } else if g.movement_mode != MovementMode::Eaten && !god_mode {
                    world[pac.entity_id].set_vx(0.0);
                    world[pac.entity_id].set_vy(0.0);
                    for ghost in ghosts.iter() {
                        world[ghost.entity_id].set_vx(0.0);
                        world[ghost.entity_id].set_vy(0.0);
                    }

                    timing::msleep(400);

                    world[pac.entity_id].set_animation(&ANIM_PAC_DEAD);
                    for _ in 0..33 {
                        world.update(engine.get_frame());
                        engine.render(&mut world);
                    }

                    world[pac.entity_id].active = false;
                    engine.mark_all_dirty();
                    engine.render(&mut world);
                    timing::msleep(500);

                    lives = lives.saturating_sub(1);
                    draw_lives(lives);

                    if lives > 0 {
                        pac.reset_position(&mut world, &level_mgr.config);

                        for ghost in ghosts.iter_mut() {
                            ghost.reset_position(&mut world, &level_mgr.config);
                        }

                        engine.mark_all_dirty();
                        engine.render(&mut world);
                        timing::msleep(800);
                        continue 'gameloop;
                    } else {
                        delete_save("pacman", None);
                        break 'gameloop;
                    }
                }
            }
        }

        // 6. Level Victory Condition
        if !has_remaining_dots(&grid) {
            world[pac.entity_id].set_vx(0.0);
            world[pac.entity_id].set_vy(0.0);
            for g in ghosts.iter() {
                world[g.entity_id].set_vx(0.0);
                world[g.entity_id].set_vy(0.0);
            }
            engine.render(&mut world);
            draw_centered_string(
                "LEVEL COMPLETE!\0",
                CENTER.y - LARGE_CHAR_HEIGHT / 2,
                true,
                &COMPLETE_COLOR_CONFIG,
                false,
            );
            timing::msleep(1500);

            push_rect_uniform(
                numworks_utils::eadk::Rect {
                    x: VIEW_SCREEN_X,
                    y: VIEW_SCREEN_Y,
                    height: VIEW_SCREEN_H,
                    width: VIEW_SCREEN_W,
                },
                Color::BLACK,
            );
            timing::msleep(500);

            level_mgr.advance_level();
            level_mgr.config.pac_speed *= speed_mult;
            level_mgr.config.pac_dot_speed *= speed_mult;
            level_mgr.config.ghost_speed *= speed_mult;
            level_mgr.config.frightened_ghost_speed *= speed_mult;
            level_mgr.config.tunnel_ghost_speed *= speed_mult;
            level_mgr.config.eaten_ghost_speed *= speed_mult;

            write_save(
                "pacman",
                None,
                &PacmanSave {
                    level: level_mgr.current_level,
                    score,
                    lives,
                },
            );

            draw_constant_ui(*high_score);
            draw_score(score);
            draw_level(level_mgr.current_level);
            draw_lives(lives);

            load_maze_into_buffers(MAZE_BYTES, &mut grid);
            world.tilemap_mut(maze_map_id).set_offset([0, 0]);

            pac.reset_position(&mut world, &level_mgr.config);
            pac.dots_eaten = 0;

            for ghost in ghosts.iter_mut() {
                ghost.reset_position(&mut world, &level_mgr.config);
            }

            world[fruit_id].active = false;
            world[fruit_id].set_animation(fruit_anim(level_mgr.config.fruit_id));

            engine.mark_all_dirty();
            engine.render(&mut world);
            draw_centered_string(
                "READY!\0",
                CENTER.y - LARGE_CHAR_HEIGHT / 2,
                true,
                &READY_COLOR_CONFIG,
                false,
            );
            timing::msleep(1500);
            engine.mark_all_dirty();

            continue 'gameloop;
        }

        engine.render(&mut world);
    }

    if score > *high_score {
        *high_score = score;
    }

    draw_centered_string(
        "GAME OVER\0",
        SCREEN_HEIGHT / 2 - LARGE_CHAR_HEIGHT,
        true,
        &COLOR_CONFIG,
        true,
    );

    selection(
        &COLOR_CONFIG,
        &MenuConfig {
            choices: &["Play again\0", "Menu\0", "Exit\0"],
            rect_margins: (20, 12),
            dimensions: (160, LARGE_CHAR_HEIGHT * 6),
            offset: (0, 40),
            back_key_return: 2,
        },
        false,
    )
}

#[inline(always)]
pub fn has_remaining_dots(grid: &Grid) -> bool {
    grid.iter()
        .any(|&space| matches!(space, Space::Point | Space::Superball))
}
