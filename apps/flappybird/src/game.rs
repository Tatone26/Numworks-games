use heapless::Vec;

use num_engine::{
    define_scratch_buffer, define_tilemap_buffer,
    graphics::{
        compositor::InterlaceMode,
        tilemap::{Parallax, WrapMode},
    },
    init_engine_window, inset_hitbox, tile_span,
    world::World,
    world_fill_tilemap, world_spawn, world_tilemap, Engine,
};
use numworks_utils::{
    eadk::{
        display::{SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, Color, Point,
    },
    graphical::{draw_centered_string, fill_screen, ColorConfig},
    menu::{
        selection,
        settings::{write_values_to_file, Setting},
        start_menu, MenuConfig,
    },
    utils::{randint, CENTER, LARGE_CHAR_HEIGHT},
};

use crate::{
    flappy_ui::{
        countdown, draw_constant_ui, draw_ui, menu_vis_addon, ANIM_BIRD_DEAD, ANIM_BIRD_FALL,
        ANIM_BIRD_FLAP_UP, ANIM_CLOUD, BACKGROUND, TILESET, TILESET_TILE_SIZE, UI_BACKGROUND,
    },
    pipes::PipePool,
};

// =============================================================================
// Display & Layout Metrics
// =============================================================================
pub const WINDOW_SIZE: u16 = 20;

pub const VIEW_SCREEN_X: u16 = 20;
pub const VIEW_SCREEN_Y: u16 = 20;
pub const VIEW_SCREEN_W: u16 = 280;
pub const VIEW_SCREEN_H: u16 = 220;

pub const TILE_SIZE: usize = 20;
pub const CELL_AREA: usize = 400; // 20 * 20 px

pub const WINDOW_COLS: usize = 14; // 280 / 20
pub const WINDOW_ROWS: usize = 11; // 220 / 20

const DEBUG_MODE: bool = false;

// 1. Solid Ground Layer: 1 row high (row 10), placed at Y = 200 px
pub const GROUND_COLS: usize = 15;
pub const GROUND_ROWS: usize = 1;
pub const GROUND_ROW_INDEX: usize = WINDOW_ROWS - 1; // row 10
pub const GROUND_Y: i16 = (GROUND_ROW_INDEX * TILE_SIZE) as i16; // 200 px

// 2. Foreground Decor Layer: 3 rows high (rows 7, 8, 9), placed at Y = 140 px (No collision)
pub const DECOR_COLS: usize = 15;
pub const DECOR_ROWS: usize = 3;
pub const DECOR_START_ROW: usize = GROUND_ROW_INDEX - DECOR_ROWS; // row 7
pub const DECOR_Y: i16 = (DECOR_START_ROW * TILE_SIZE) as i16; // 140 px

pub const MAX_PIPES_ON_SCREEN: usize = 6;
const NUM_CLOUDS: usize = 2;

const WORLD_ENT_CAP: usize = 16;
const WORLD_MAP_CAP: usize = 3;

pub type GameEngine<'s> = Engine<'s, TILE_SIZE, CELL_AREA, WINDOW_COLS, WINDOW_ROWS, DEBUG_MODE>;
pub type GameWorld<'a> = World<'a, TILE_SIZE, CELL_AREA, WORLD_ENT_CAP, WORLD_MAP_CAP>;

// Static off-stack memory buffers
define_scratch_buffer!(SCRATCH_BUFFER, CELL_AREA, WINDOW_COLS, WINDOW_ROWS);
define_tilemap_buffer!(BG_DATA, WINDOW_COLS, WINDOW_ROWS);
define_tilemap_buffer!(DECOR_DATA, DECOR_COLS, DECOR_ROWS);
define_tilemap_buffer!(GROUND_DATA, GROUND_COLS, GROUND_ROWS);

const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: BACKGROUND,
    alt: Color::from_rgb888(255, 140, 65),
};

const NICE_COLLISION_MARGIN: u16 = 2;

pub fn start() {
    let mut opt: [&mut Setting; 7] = [
        &mut Setting {
            name: "Starting speed\0",
            choice: 1,
            values: Vec::from_slice(&[0.5_f32.to_bits(), 0.75_f32.to_bits(), 1.0_f32.to_bits()])
                .unwrap(),
            texts: Vec::from_slice(&["Slow\0", "Normal\0", "Fast\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "Pipes density\0",
            choice: 1,
            values: Vec::from_slice(&[1, 2, 3]).unwrap(),
            texts: Vec::from_slice(&["Sparse\0", "Normal\0", "Dense\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "Speed increase\0",
            choice: 2,
            values: Vec::from_slice(&[1000, 10, 5, 1]).unwrap(),
            texts: Vec::from_slice(&[
                "Never\0",
                "Every 10 pts\0",
                "Every 5 pts\0",
                "Every point\0",
            ])
            .unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "Die on floor\0",
            choice: 1,
            values: Vec::from_slice(&[0, 1]).unwrap(),
            texts: Vec::from_slice(&["No\0", "Yes\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "Jump strength\0",
            choice: 1,
            values: Vec::from_slice(&[5.5_f32.to_bits(), 6.5_f32.to_bits(), 7.5_f32.to_bits()])
                .unwrap(),
            texts: Vec::from_slice(&["Weak\0", "Normal\0", "Strong\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "No collisions\0",
            choice: 0,
            values: Vec::from_slice(&[0, 1]).unwrap(),
            texts: Vec::from_slice(&["No\0", "Yes (CHEAT)\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "High Score\0",
            choice: 0,
            values: Vec::from_slice(&[0, 0, u16::MAX as u32]).unwrap(),
            texts: Vec::new(),
            user_modifiable: false,
            fixed_values: false,
        },
    ];

    loop {
        let start = start_menu(
            "FLAPPY BIRD\0",
            &mut opt,
            &COLOR_CONFIG,
            menu_vis_addon,
            include_str!("./data/model_controls.txt"),
            "flappybird",
        );
        if start == 0 {
            loop {
                let mut high_score = opt[6].get_setting_value();
                let action = game(
                    f32::from_bits(opt[0].get_setting_value()),
                    opt[1].get_setting_value() as u16,
                    opt[2].get_setting_value() as u16,
                    opt[3].get_setting_value() != 0,
                    f32::from_bits(opt[4].get_setting_value()),
                    opt[5].get_setting_value() != 0,
                    &mut high_score,
                );

                opt[6].set_value(high_score);
                write_values_to_file(&mut opt, "flappybird");

                if action == 2 {
                    return;
                } else if action == 1 {
                    break;
                }
            }
        } else {
            return;
        }
    }
}

pub fn game(
    starting_speed: f32,
    density: u16,
    speed_increase: u16,
    killer_floor: bool,
    jump_power: f32,
    no_collisions: bool,
    high_score: &mut u32,
) -> u8 {
    fill_screen(BACKGROUND);
    draw_constant_ui(*high_score as u16);
    draw_ui(0);

    // -------------------------------------------------------------------------
    // 1. Engine & Scene Setup
    // -------------------------------------------------------------------------
    let mut engine: GameEngine = init_engine_window!(
        color: BACKGROUND,
        window: [VIEW_SCREEN_X, VIEW_SCREEN_Y, VIEW_SCREEN_W, VIEW_SCREEN_H],
        interlace: Some(InterlaceMode::None),
        scratch: SCRATCH_BUFFER,
    );

    let mut world: GameWorld = GameWorld::new();

    // -------------------------------------------------------------------------
    // 2. Tilemap Registration
    // -------------------------------------------------------------------------
    let _ = world_fill_tilemap!(
        world: world,
        tileset: &TILESET,
        buffer: BG_DATA,
        cols: WINDOW_COLS,
        rows: WINDOW_ROWS,
        z: 0,
        tile: [3u8, 3u8],
        parallax: Parallax::FIXED,
        wrap: WrapMode::None,
    );

    let decor_map_id = world_tilemap!(
        world: world,
        tileset: &TILESET,
        buffer: DECOR_DATA,
        cols: DECOR_COLS,
        rows: DECOR_ROWS,
        offset: [0, DECOR_Y],
        z: 14,
        transparent: true,
        parallax: Parallax::FOREGROUND,
        wrap: WrapMode::Horizontal,
    )
    .unwrap();

    let ground_map_id = world_tilemap!(
        world: world,
        tileset: &TILESET,
        buffer: GROUND_DATA,
        cols: GROUND_COLS,
        rows: GROUND_ROWS,
        offset: [0, GROUND_Y],
        z: 15,
        transparent: true,
        parallax: Parallax::FOREGROUND,
        wrap: WrapMode::Horizontal,
    )
    .unwrap();

    tile_span!(world.tilemap_mut(ground_map_id), row: 0, cols: 0..GROUND_COLS, tile: [0, 4]);

    // Set scroll velocity directly on tilemaps: they will step themselves in world.update()
    world
        .tilemap_mut(decor_map_id)
        .set_horizontal_speed(starting_speed);
    world
        .tilemap_mut(ground_map_id)
        .set_horizontal_speed(starting_speed);

    // -------------------------------------------------------------------------
    // 3. Entity Instantiation
    // -------------------------------------------------------------------------
    let mut cloud_ids: Vec<usize, NUM_CLOUDS> = Vec::new();
    for i in 0..NUM_CLOUDS {
        let cx = (VIEW_SCREEN_W as i16 - 40) - (i as i16 * 130);
        let cy = 5 + (i as i16 * 20);
        let id = world_spawn!(
            world: world,
            anim: &ANIM_CLOUD,
            pos: [cx as f32, cy as f32],
            z: 2,
        )
        .unwrap();
        world[id].set_vx(-0.20);
        let _ = cloud_ids.push(id);
    }

    let mut pipes = PipePool::new(density, starting_speed, &mut world);

    let bird_x: i16 = 60;
    let bird_id = world_spawn!(
        world: world,
        anim: &ANIM_BIRD_FALL,
        pos: [bird_x as f32, 90.0],
        z: 20,
        hitbox: inset_hitbox!(TILESET_TILE_SIZE, TILESET_TILE_SIZE, NICE_COLLISION_MARGIN),
    )
    .unwrap();

    let mut current_speed = starting_speed;
    let mut score: u16 = 0;
    let mut can_increase_speed = true;
    let mut frame_counter: u16 = 0;
    let mut started = false;
    let mut jump_latched = false;

    // -------------------------------------------------------------------------
    // 4. Initial World Presentation & Countdown
    // -------------------------------------------------------------------------
    countdown(
        Point {
            x: CENTER.x - TILESET_TILE_SIZE,
            y: CENTER.y - TILESET_TILE_SIZE * 2,
        },
        &mut engine,
        &mut world,
    );

    draw_constant_ui(*high_score as u16);
    draw_ui(0);

    // =========================================================================
    // Simulation Loop
    // =========================================================================
    'gameloop: loop {
        let scan = keyboard::scan();

        if scan.key_down(key::BACK) {
            let action = flappy_pause(false);
            if action != 0 {
                return action;
            } else {
                fill_screen(BACKGROUND);
                countdown(
                    Point {
                        x: CENTER.x - TILESET_TILE_SIZE,
                        y: CENTER.y - TILESET_TILE_SIZE * 2,
                    },
                    &mut engine,
                    &mut world,
                );
                draw_constant_ui(*high_score as u16);
                draw_ui(score);
                continue;
            }
        }

        let frame = engine.get_frame();

        // 1. Bird Input & Dynamics
        let jump_pressed = scan.key_down(key::OK) || scan.key_down(key::UP);
        if !started && (jump_pressed || frame_counter > 20) {
            started = true;
        }

        let bird = &mut world[bird_id];
        if started {
            if jump_pressed && !jump_latched {
                bird.set_vy(-jump_power);
                jump_latched = true;
            } else if !jump_pressed {
                jump_latched = false;
            }

            bird.set_vy((bird.vy() + 0.7).min(8.0));

            // World-space ceiling collision
            if bird.y() <= 0.0 {
                bird.set_y(0.0);
                bird.set_vy(0.7);
            }

            if bird.vy() < 0.0 {
                bird.set_animation(&ANIM_BIRD_FLAP_UP);
            } else {
                bird.set_animation(&ANIM_BIRD_FALL);
            }
        }

        // 2. Scene Simulation Tick (Steps bodies, sprite offsets, anims, AND tilemap scrolls)
        world.update(frame);

        // 3. Clouds Horizontal Wrap
        for &c_id in cloud_ids.iter() {
            let cloud = &mut world[c_id];
            if cloud.x() < -40.0 {
                cloud.set_pos(VIEW_SCREEN_W as f32, randint(5, 45) as f32);
            }
        }

        // 4. Pipe Queue, Spawning & Scoring
        if pipes.update(&mut world, bird_x) {
            score += 1;
            can_increase_speed = true;
            draw_ui(score);
        }

        // 5. Collision Checks
        if !no_collisions {
            if pipes.collides(&world, bird_id) {
                break 'gameloop;
            }

            if world.collides_tilemap(bird_id, ground_map_id) {
                if killer_floor {
                    break 'gameloop;
                } else {
                    let floor_surface_y = (GROUND_Y - TILESET_TILE_SIZE as i16) as f32;
                    let bird = &mut world[bird_id];
                    bird.set_y(floor_surface_y);
                    bird.set_vy(0.1);
                }
            }
        }

        // 6. Dynamic Speed Scaling
        if can_increase_speed && score != 0 && score.is_multiple_of(speed_increase) {
            current_speed *= 1.15;
            pipes.set_speed(&mut world, current_speed);
            world
                .tilemap_mut(decor_map_id)
                .set_horizontal_speed(current_speed);
            world
                .tilemap_mut(ground_map_id)
                .set_horizontal_speed(current_speed);
            can_increase_speed = false;
        }

        // 7. Hardware Render (Sparse invalidation automatically tracks moving ground & decor tiles)
        engine.render(&mut world);

        // can also use (it is the same):
        //        world.render_to_engine(&mut engine, progressive);

        frame_counter = frame_counter.wrapping_add(1);
    }

    // Death Presentation
    world[bird_id].set_animation(&ANIM_BIRD_DEAD);
    engine.render_progressive(&mut world);

    draw_centered_string("GAME OVER\0", 70, true, &COLOR_CONFIG, true);

    if score > *high_score as u16 {
        draw_centered_string(
            "NEW HIGH SCORE!\0",
            70 + LARGE_CHAR_HEIGHT + 2,
            true,
            &COLOR_CONFIG,
            true,
        );
        *high_score = score as u32;
    }

    flappy_pause(true)
}

fn flappy_pause(death: bool) -> u8 {
    selection(
        &ColorConfig {
            text: Color::WHITE,
            bckgrd: UI_BACKGROUND,
            alt: COLOR_CONFIG.alt,
        },
        &MenuConfig {
            choices: if death {
                &["Play again\0", "Menu\0", "Exit\0"]
            } else {
                &["Resume\0", "Menu\0", "Exit\0"]
            },
            rect_margins: (20, 0),
            dimensions: (SCREEN_WIDTH, LARGE_CHAR_HEIGHT + LARGE_CHAR_HEIGHT / 2),
            offset: (
                0,
                SCREEN_HEIGHT as i16 / 2 - 2 * LARGE_CHAR_HEIGHT as i16 / 3,
            ),
            back_key_return: if death { 2 } else { 1 },
        },
        true,
    )
}
