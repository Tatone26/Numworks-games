use core::str::FromStr;

use heapless::String;
use num_engine::{
    define_scratch_buffer, define_tilemap_buffer,
    engine::InterlaceTuning,
    graphics::{
        compositor::InterlaceMode,
        particles::ParticleKind,
        tilemap::{Parallax, WrapMode},
    },
    init_engine_window, inset_hitbox, tile_span,
    world::World,
    world_ascii_tilemap, world_fill_tilemap, world_particles, world_spawn, world_tilemap, Engine,
};
use numworks_utils::{
    eadk::{
        display::{SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, timing, Color, Point,
    },
    graphical::{draw_centered_string, fill_screen, ColorConfig},
    menu::{selection, MenuConfig},
    utils::{string_from_u16, CENTER, LARGE_CHAR_HEIGHT},
};

use crate::{
    events::{EventConfig, EventManager, GameEvent},
    flappy_ui::{
        countdown, draw_constant_ui, draw_ui, headwind_emitter, high_gravity_emitter,
        low_gravity_emitter, tailwind_emitter, ANIM_BIRD_DEAD, ANIM_BIRD_FALL, ANIM_BIRD_FLAP_UP,
        BACKGROUND, TILESET, TILESET_TILE_SIZE, UI_BACKGROUND,
    },
    pipes::{OscillationMode, PipePool},
};

pub const WINDOW_SIZE: u16 = 20;
pub const VIEW_SCREEN_X: u16 = 20;
pub const VIEW_SCREEN_Y: u16 = 20;
pub const VIEW_SCREEN_W: u16 = 280;
pub const VIEW_SCREEN_H: u16 = 220;

pub const TILE_SIZE: usize = 20;
pub const CELL_AREA: usize = 400;
pub const WINDOW_COLS: usize = 14;
pub const WINDOW_ROWS: usize = 11;

const DEBUG_MODE: bool = true;

pub const GROUND_COLS: usize = 15;
pub const GROUND_ROWS: usize = 1;
pub const GROUND_ROW_INDEX: usize = WINDOW_ROWS - 1;
pub const GROUND_Y: i16 = (GROUND_ROW_INDEX * TILE_SIZE) as i16;

pub const DECOR_COLS: usize = 30;
pub const DECOR_ROWS: usize = 3;
pub const DECOR_START_ROW: usize = GROUND_ROW_INDEX - DECOR_ROWS;
pub const DECOR_Y: i16 = (DECOR_START_ROW * TILE_SIZE) as i16;

pub const CLOUDS_COLS: usize = 25;
pub const CLOUDS_ROWS: usize = 4;
pub const CLOUDS_Y: usize = 0;

pub const MAX_PIPES_ON_SCREEN: usize = 8;

const WORLD_ENT_CAP: usize = 32;
const WORLD_MAP_CAP: usize = 4;
const WORLD_PARTS_PER_ENT: usize = 4;
const WORLD_RENDER_CAP: usize = 128;
const WORLD_PARTICLE_CAP: usize = 48;
const WORLD_PARTICLE_SYS_CAP: usize = 4;

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
define_tilemap_buffer!(BG_DATA, WINDOW_COLS, WINDOW_ROWS);
define_tilemap_buffer!(DECOR_DATA, DECOR_COLS, DECOR_ROWS);
define_tilemap_buffer!(GROUND_DATA, GROUND_COLS, GROUND_ROWS);
define_tilemap_buffer!(CLOUD_DATA, CLOUDS_COLS, CLOUDS_ROWS);

pub const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: BACKGROUND,
    alt: Color::from_rgb888(255, 140, 65),
};

const DEATH_COLOR_CONFIG: ColorConfig = ColorConfig {
    text: COLOR_CONFIG.alt,
    bckgrd: Color::from_rgb888(50, 50, 50),
    alt: Color::RED,
};

const NICE_COLLISION_MARGIN: u16 = 2;

fn sync_particles(world: &mut GameWorld, wind_id: usize, grav_id: usize, event: GameEvent) {
    match event {
        GameEvent::Tailwind => {
            world.particles_mut(wind_id).set_emitter(tailwind_emitter());
        }
        GameEvent::Headwind => {
            world.particles_mut(wind_id).set_emitter(headwind_emitter());
        }
        GameEvent::LowGravity => {
            world
                .particles_mut(grav_id)
                .set_emitter(low_gravity_emitter());
        }
        GameEvent::HighGravity => {
            world
                .particles_mut(grav_id)
                .set_emitter(high_gravity_emitter());
        }
        _ => {
            world.particles_mut(wind_id).enable_emitter(false);
            world.particles_mut(grav_id).enable_emitter(false);
        }
    }
}

fn play_death_drop(bird_id: usize, world: &mut GameWorld, engine: &mut GameEngine) {
    let floor_y = (GROUND_Y - TILESET_TILE_SIZE as i16) as f32;
    let bird = &mut world[bird_id];
    bird.set_animation(&ANIM_BIRD_DEAD);
    if bird.y() < floor_y {
        bird.set_vy(-2.4);
    }

    while world[bird_id].y() < floor_y {
        let bird = &mut world[bird_id];
        let vy = (bird.vy() + 0.75).min(9.0);
        bird.set_vy(vy);
        bird.set_y((bird.y() + vy).min(floor_y));

        world.update(engine.get_frame());
        engine.render(world);
    }

    world[bird_id].set_y(floor_y);
    world[bird_id].set_vy(0.0);
    engine.render_progressive(world);
    timing::msleep(400);
}

pub fn game(
    starting_speed: f32,
    density: u16,
    gap_difficulty: u8,
    osc_mode: OscillationMode,
    event_config: EventConfig,
    speed_increase: u16,
    jump_power: f32,
    no_collisions: bool,
    high_score: &mut u32,
) -> u8 {
    fill_screen(BACKGROUND);
    draw_constant_ui(*high_score as u16);
    draw_ui(0);

    let mut engine: GameEngine = init_engine_window!(
        color: BACKGROUND,
        window: [VIEW_SCREEN_X, VIEW_SCREEN_Y, VIEW_SCREEN_W, VIEW_SCREEN_H],
        interlace: Some(InterlaceMode::Columns),
        scratch: SCRATCH_BUFFER,
    );
    engine.set_interlace_tuning(InterlaceTuning {
        target_budget_us: 22_000,
        hysteresis_us: 2_500,
        tripwire_frames: 1,
        cooldown_frames: 1,
    });

    let mut world: GameWorld = GameWorld::new();
    let mut events = EventManager::new(event_config);

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

    let clouds_map_id = world_ascii_tilemap!(
        world: world,
        tileset: &TILESET,
        buffer: CLOUD_DATA,
        cols: CLOUDS_COLS,
        rows: CLOUDS_ROWS,
        offset: [0, CLOUDS_Y],
        z: 2,
        transparent: false,
        parallax: Parallax::FIXED,
        wrap: WrapMode::Horizontal,
        mapping: {
            b'.' => None,
            b'A' => Some([1, 3]),
            b'B' => Some([2, 3]),
        },
        layout: "
        .........AB.........AB...
        ..AB........AB...........
        ......AB.................
        ....................AB...
    ",
    )
    .unwrap();
    world.tilemap_mut(clouds_map_id).set_horizontal_speed(-0.05);

    let apply_speed = |world: &mut GameWorld, pipes: &mut PipePool, speed: f32| {
        pipes.set_speed(world, speed);
        world.tilemap_mut(decor_map_id).set_horizontal_speed(-speed);
        world
            .tilemap_mut(ground_map_id)
            .set_horizontal_speed(-speed);
    };

    let wind_id = world_particles!(
        world: world,
        z: 25,
        kind: ParticleKind::HorizontalStreak,
    )
    .unwrap();

    let grav_id = world_particles!(
        world: world,
        z: 25,
        kind: ParticleKind::VerticalStreak,
    )
    .unwrap();

    let mut pipes = PipePool::new(
        density,
        starting_speed,
        jump_power,
        gap_difficulty,
        osc_mode,
        &events,
        &mut world,
    );
    apply_speed(&mut world, &mut pipes, starting_speed);

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

    'gameloop: loop {
        let scan = keyboard::scan();

        if scan.key_down(key::BACK) {
            let action = flappy_pause(false);
            if action != 0 {
                return action;
            }
            world.particles_mut(wind_id).clear();
            world.particles_mut(grav_id).clear();
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

        let frame = engine.get_frame();

        // 1. Dynamic Events Transition & Velocity Updates
        let dense_cleared = !pipes.has_dense_ahead(&world, bird_x);
        let steep_leap_ahead = pipes.has_steep_leap_ahead(&world, bird_x);
        if events.update(dense_cleared, steep_leap_ahead) {
            let eff = current_speed * events.speed_multiplier();
            apply_speed(&mut world, &mut pipes, eff);
            sync_particles(&mut world, wind_id, grav_id, events.active_particle_event());
        }

        // 2. Bird Input & Dynamics
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

            let gravity = 0.65 * events.gravity_multiplier();
            bird.set_vy((bird.vy() + gravity).min(8.0));

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

        // 3. Scene Simulation Tick
        world.update(frame);

        // 4. Pipe Queue, Spawning & Scoring
        if pipes.update(&mut world, bird_x, &events) {
            score += 1;
            can_increase_speed = true;
            draw_ui(score);
        }

        // 5. Collisions
        if !no_collisions
            && (pipes.collides(&world, bird_id) || world.collides_tilemap(bird_id, ground_map_id))
        {
            break 'gameloop;
        }

        // 6. Progressive Speed Scaling
        if can_increase_speed && score != 0 && score.is_multiple_of(speed_increase) {
            current_speed *= 1.15;
            let eff = current_speed * events.speed_multiplier();
            apply_speed(&mut world, &mut pipes, eff);
            can_increase_speed = false;
        }

        // 7. Render
        engine.render(&mut world);
        frame_counter = frame_counter.wrapping_add(1);
    }

    // Death & Game Over Sequence
    apply_speed(&mut world, &mut pipes, 0.0);
    world.particles_mut(wind_id).enable_emitter(false);
    world.particles_mut(grav_id).enable_emitter(false);

    play_death_drop(bird_id, &mut world, &mut engine);

    draw_centered_string("GAME OVER\0", 70, true, &DEATH_COLOR_CONFIG, true);
    let mut score_str: String<15> = String::from_str("Score: ").unwrap();
    score_str.push_str(&string_from_u16(score)).unwrap();
    draw_centered_string(
        &score_str,
        70 + 10 + LARGE_CHAR_HEIGHT,
        true,
        &DEATH_COLOR_CONFIG,
        false,
    );

    if score > *high_score as u16 {
        draw_centered_string(
            "NEW HIGH SCORE!\0",
            70 + 20 + LARGE_CHAR_HEIGHT * 2,
            true,
            &DEATH_COLOR_CONFIG,
            false,
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
