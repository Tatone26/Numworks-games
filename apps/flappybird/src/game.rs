use heapless::Vec;

use num_engine::{
    define_scratch_buffer, define_tilemap_buffer,
    graphics::{
        compositor::InterlaceMode,
        particles::ParticleKind,
        tilemap::{Parallax, WrapMode},
    },
    init_engine_window, inset_hitbox, tile_span,
    world::World,
    world_fill_tilemap, world_particles, world_spawn, world_tilemap, Engine,
};
use numworks_utils::{
    eadk::{
        display::{SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, timing, Color, Point,
    },
    graphical::{draw_centered_string, fill_screen, ColorConfig},
    menu::{selection, MenuConfig},
    utils::{randint, CENTER, LARGE_CHAR_HEIGHT},
};

use crate::{
    events::{EventConfig, EventManager, GameEvent},
    flappy_ui::{
        countdown, draw_constant_ui, draw_ui, headwind_emitter, high_gravity_emitter,
        low_gravity_emitter, tailwind_emitter, ANIM_BIRD_DEAD, ANIM_BIRD_FALL, ANIM_BIRD_FLAP_UP,
        ANIM_CLOUD, BACKGROUND, TILESET, TILESET_TILE_SIZE, UI_BACKGROUND,
    },
    pipes::{OscillationMode, PipePool},
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
pub const CELL_AREA: usize = 400;

pub const WINDOW_COLS: usize = 14;
pub const WINDOW_ROWS: usize = 11;

const DEBUG_MODE: bool = true;

pub const GROUND_COLS: usize = 15;
pub const GROUND_ROWS: usize = 1;
pub const GROUND_ROW_INDEX: usize = WINDOW_ROWS - 1;
pub const GROUND_Y: i16 = (GROUND_ROW_INDEX * TILE_SIZE) as i16;

pub const DECOR_COLS: usize = 15;
pub const DECOR_ROWS: usize = 3;
pub const DECOR_START_ROW: usize = GROUND_ROW_INDEX - DECOR_ROWS;
pub const DECOR_Y: i16 = (DECOR_START_ROW * TILE_SIZE) as i16;

pub const MAX_PIPES_ON_SCREEN: usize = 8;
const NUM_CLOUDS: usize = 2;

const WORLD_ENT_CAP: usize = 32;
const WORLD_MAP_CAP: usize = 3;
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

pub const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: BACKGROUND,
    alt: Color::from_rgb888(255, 140, 65),
};

const NICE_COLLISION_MARGIN: u16 = 2;

pub fn game(
    starting_speed: f32,
    density: u16,
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

    world
        .tilemap_mut(decor_map_id)
        .set_horizontal_speed(starting_speed);
    world
        .tilemap_mut(ground_map_id)
        .set_horizontal_speed(starting_speed);

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

    let wind_particles_id = world_particles!(
        world: world,
        z: 25,
        kind: ParticleKind::HorizontalStreak,
    )
    .unwrap();

    let grav_particles_id = world_particles!(
        world: world,
        z: 25,
        kind: ParticleKind::VerticalStreak,
    )
    .unwrap();

    let mut pipes = PipePool::new(density, starting_speed, osc_mode, &events, &mut world);

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
            } else {
                world.particles_mut(wind_particles_id).clear();
                world.particles_mut(grav_particles_id).clear();
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

        // 1. Events State Transitions & Synchronized Updates
        let event_transition = events.update();
        if event_transition {
            let effective_speed = current_speed * events.speed_multiplier();
            pipes.set_speed(&mut world, effective_speed);
            world
                .tilemap_mut(decor_map_id)
                .set_horizontal_speed(effective_speed);
            world
                .tilemap_mut(ground_map_id)
                .set_horizontal_speed(effective_speed);

            match events.active {
                GameEvent::MovingPipesSurge => {
                    pipes.apply_moving_surge(&mut world);
                }
                GameEvent::Tailwind => {
                    world
                        .particles_mut(wind_particles_id)
                        .set_emitter(tailwind_emitter());
                }
                GameEvent::Headwind => {
                    world
                        .particles_mut(wind_particles_id)
                        .set_emitter(headwind_emitter());
                }
                GameEvent::LowGravity => {
                    world
                        .particles_mut(grav_particles_id)
                        .set_emitter(low_gravity_emitter());
                }
                GameEvent::HighGravity => {
                    world
                        .particles_mut(grav_particles_id)
                        .set_emitter(high_gravity_emitter());
                }
                GameEvent::None => {
                    world.particles_mut(wind_particles_id).enable_emitter(false);
                    world.particles_mut(grav_particles_id).enable_emitter(false);
                    pipes.stop_surge(&mut world);
                }
                _ => {}
            }
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

        // 4. Clouds Horizontal Wrap
        for &c_id in cloud_ids.iter() {
            let cloud = &mut world[c_id];
            if cloud.x() < -40.0 {
                cloud.set_pos(VIEW_SCREEN_W as f32, randint(5, 45) as f32);
            }
        }

        // 5. Pipe Queue, Spawning, Morphing & Scoring
        if pipes.update(&mut world, bird_x, &events) {
            score += 1;
            can_increase_speed = true;
            draw_ui(score);
        }

        // 6. Collision Checks
        if !no_collisions {
            if pipes.collides(&world, bird_id) {
                break 'gameloop;
            }

            if world.collides_tilemap(bird_id, ground_map_id) {
                break 'gameloop;
            }
        }

        // 7. Dynamic Speed Scaling
        if can_increase_speed && score != 0 && score.is_multiple_of(speed_increase) {
            current_speed *= 1.15;
            let eff = current_speed * events.speed_multiplier();
            pipes.set_speed(&mut world, eff);
            world.tilemap_mut(decor_map_id).set_horizontal_speed(eff);
            world.tilemap_mut(ground_map_id).set_horizontal_speed(eff);
            can_increase_speed = false;
        }

        // 8. Hardware Render
        engine.render(&mut world);

        frame_counter = frame_counter.wrapping_add(1);
    }

    // =========================================================================
    // Arcade Death Drop Sequence
    // =========================================================================
    // 1. Freeze horizontal world scrolling and disable active particle emitters
    pipes.set_speed(&mut world, 0.0);
    world.tilemap_mut(decor_map_id).set_horizontal_speed(0.0);
    world.tilemap_mut(ground_map_id).set_horizontal_speed(0.0);
    world.particles_mut(wind_particles_id).enable_emitter(false);
    world.particles_mut(grav_particles_id).enable_emitter(false);

    // 2. Switch bird sprite to dead pose and give it an initial upward impact bounce
    let floor_y = (GROUND_Y - TILESET_TILE_SIZE as i16) as f32;
    let bird = &mut world[bird_id];
    bird.set_animation(&ANIM_BIRD_DEAD);
    if bird.y() < floor_y {
        bird.set_vy(-2.4); // Subtle upward bump before plummeting
    }

    // 3. Fall straight down to the grass
    while world[bird_id].y() < floor_y {
        let bird = &mut world[bird_id];
        let vy = (bird.vy() + 0.75).min(9.0);
        bird.set_vy(vy);
        bird.set_y((bird.y() + vy).min(floor_y));

        let frame = engine.get_frame();
        world.update(frame);
        engine.render(&mut world);
    }

    // Rest dead bird firmly on the ground surface
    world[bird_id].set_y(floor_y);
    world[bird_id].set_vy(0.0);
    engine.render_progressive(&mut world);
    timing::msleep(400);

    // =========================================================================
    // Game Over Presentation
    // =========================================================================
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
