use heapless::Vec;

use num_engine::{
    compositor::{InterlaceMode, Renderable},
    engine::Engine,
    spawn_sprite,
    sprite::Sprite,
    tilemap::{Parallax, Tilemap},
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
    pipes::PipePair,
};

pub const WINDOW_SIZE: u16 = 20;

// Sub-screen Viewport Window on physical display
pub const VIEW_SCREEN_X: u16 = 20;
pub const VIEW_SCREEN_Y: u16 = 20;
pub const VIEW_SCREEN_W: u16 = 280;
pub const VIEW_SCREEN_H: u16 = 220;

// -----------------------------------------------------------------------------
// Engine Generics
// -----------------------------------------------------------------------------
pub const TILE_SIZE: usize = 20;
pub const CELL_AREA: usize = 400; // 20 * 20
pub const SCREEN_COLS: usize = 14; // 280 / 20
pub const SCREEN_ROWS: usize = 11; // 220 / 20
const DEBUG_MODE: bool = true;

const WINDOW_TILES: usize = SCREEN_COLS * SCREEN_ROWS; // 154

// Ground tilemap: 15 cols x 11 rows (Row 10 is the ground line [0, 4])
const GROUND_COLS: usize = 15;
const GROUND_ROWS: usize = SCREEN_ROWS; // 11
const GROUND_TILES: usize = GROUND_COLS * GROUND_ROWS;

const MAX_PIPES_ON_SCREEN: usize = 6;
const NUM_CLOUDS: usize = 2;

pub type GameEngine = Engine<TILE_SIZE, CELL_AREA, SCREEN_COLS, SCREEN_ROWS, DEBUG_MODE>;
pub type GameTilemap<'a> = Tilemap<'a, TILE_SIZE, CELL_AREA>;
pub type GameSprite<'a> = Sprite<'a, TILE_SIZE, CELL_AREA>;
pub type GameRenderable<'r, 'a> = Renderable<'r, 'a, TILE_SIZE, CELL_AREA>;

const MAX_CHUNKS: usize = if SCREEN_COLS > SCREEN_ROWS {
    SCREEN_COLS
} else {
    SCREEN_ROWS
};
const SCRATCH_PIXELS: usize = CELL_AREA + (MAX_CHUNKS * CELL_AREA);
static mut SCRATCH_BUFFER: [Color; SCRATCH_PIXELS] = [Color::BLACK; SCRATCH_PIXELS];

const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: BACKGROUND,
    alt: Color::from_rgb888(255, 140, 65),
};

const NICE_COLLISION_MARGIN: i16 = 2;

// -----------------------------------------------------------------------------
// Menu & Options Entry Point
// -----------------------------------------------------------------------------
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

// -----------------------------------------------------------------------------
// Main Game Engine Loop
// -----------------------------------------------------------------------------
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

    let mut engine: GameEngine = GameEngine::new_with_window(
        BACKGROUND,
        VIEW_SCREEN_X,
        VIEW_SCREEN_Y,
        VIEW_SCREEN_W,
        VIEW_SCREEN_H,
        Some(InterlaceMode::Columns),
    );

    // 1. Sky Background (Z = 0)
    let mut bg_data: [Option<[u8; 2]>; WINDOW_TILES] = [Some([3u8, 3u8]); WINDOW_TILES];
    let bg_map: GameTilemap = GameTilemap::new(
        &TILESET,
        &mut bg_data,
        SCREEN_COLS,
        SCREEN_ROWS,
        &[],
        0,
        false,
        Parallax::FIXED,
        false,
    );

    // 2. Ground Tilemap (Z = 15, row 10 is [0, 4])
    let mut ground_data: [Option<[u8; 2]>; GROUND_TILES] = [None; GROUND_TILES];
    let ground_row = SCREEN_ROWS - 1;
    for col in 0..GROUND_COLS {
        ground_data[ground_row * GROUND_COLS + col] = Some([0, 4]);
    }
    let mut ground_map: GameTilemap = GameTilemap::new(
        &TILESET,
        &mut ground_data,
        GROUND_COLS,
        GROUND_ROWS,
        &[],
        15,
        true,
        Parallax::FOREGROUND,
        true,
    );

    // 3. Cloud Sprites (Z = 2) - in world space [0..280]
    let mut clouds: Vec<GameSprite, NUM_CLOUDS> = Vec::new();
    for i in 0..NUM_CLOUDS {
        let cx = (VIEW_SCREEN_W as i16 - 40) - (i as i16 * 130);
        let cy = 5 + (i as i16 * 20);
        let mut c: GameSprite = spawn_sprite!(&ANIM_CLOUD, [cx, cy], 2);
        c.set_speed([-0.20, 0.0]);
        let _ = clouds.push(c);
    }

    // 4. Pipe Spacing & Pool
    let pipe_spacing: i16 = match density {
        3 => 75,
        2 => 105,
        _ => 145,
    };

    let mut pipes: Vec<PipePair, MAX_PIPES_ON_SCREEN> = Vec::new();
    for _ in 0..MAX_PIPES_ON_SCREEN {
        let _ = pipes.push(PipePair::new(75, starting_speed));
    }
    pipes[0].active = true;
    pipes[0].warp_to(VIEW_SCREEN_W as i16);

    // 5. Bird Sprite (Z = 20) - in world space
    let bird_x: i16 = 60;
    let mut bird_sprite: GameSprite = spawn_sprite!(&ANIM_BIRD_FALL, [bird_x, 90], 20);

    let mut current_speed = starting_speed;
    let mut score: u16 = 0;
    let mut can_increase_speed = true;
    let mut frame_counter: u16 = 0;
    let mut ground_scroll: f32 = 0.0;
    let mut started = false;
    let mut jump_latched = false;

    countdown(Point {
        x: CENTER.x - TILESET_TILE_SIZE,
        y: CENTER.y - TILESET_TILE_SIZE * 2,
    });

    draw_constant_ui(*high_score as u16);
    draw_ui(0);

    // -------------------------------------------------------------------------
    // Frame Simulation Loop
    // -------------------------------------------------------------------------
    'gameloop: loop {
        let scan = keyboard::scan();

        if scan.key_down(key::BACK) {
            let action = flappy_pause(false);
            if action != 0 {
                return action;
            } else {
                fill_screen(BACKGROUND);
                draw_constant_ui(*high_score as u16);
                draw_ui(score);
                engine.mark_all_dirty();
                countdown(Point {
                    x: CENTER.x - TILESET_TILE_SIZE,
                    y: CENTER.y - TILESET_TILE_SIZE * 3,
                });
                draw_constant_ui(*high_score as u16);
                draw_ui(score);
                continue;
            }
        }

        let frame = engine.get_frame();

        // 1. Bird Physics
        let jump_pressed = scan.key_down(key::OK) || scan.key_down(key::UP);
        if !started && (jump_pressed || frame_counter > 20) {
            started = true;
        }

        if started {
            let [_, mut vy] = bird_sprite.get_speed();

            if jump_pressed && !jump_latched {
                vy = -jump_power;
                jump_latched = true;
            } else if !jump_pressed {
                jump_latched = false;
            }

            vy = (vy + 0.7).min(8.0);
            bird_sprite.set_speed([0.0, vy]);
            bird_sprite.update(frame);

            // Ceiling clamp (world y = 0)
            if bird_sprite.position[1] <= 0 {
                bird_sprite.set_position([bird_x, 0], None);
                bird_sprite.set_speed([0.0, 0.7]);
            }

            // Floor collision check (top of row 10 is world y = 200)
            let floor_y = 200 - TILESET_TILE_SIZE as i16;
            if bird_sprite.position[1] >= floor_y {
                bird_sprite.set_position([bird_x, floor_y], None);
                bird_sprite.set_speed([0.0, 0.1]);
                if killer_floor && !no_collisions {
                    break 'gameloop;
                }
            }

            if bird_sprite.get_speed()[1] < 0.0 {
                bird_sprite.set_animation(&ANIM_BIRD_FLAP_UP);
            } else {
                bird_sprite.set_animation(&ANIM_BIRD_FALL);
            }
        } else {
            bird_sprite.update(frame);
        }

        // 2. Pipes Movement & Reliable Queue Spawning
        for p in pipes.iter_mut().filter(|p| p.active) {
            p.update(frame);
        }

        let mut rightmost_x = VIEW_SCREEN_W as i16;
        for p in pipes.iter().filter(|p| p.active) {
            if p.x() > rightmost_x {
                rightmost_x = p.x();
            }
        }

        if rightmost_x <= VIEW_SCREEN_W as i16 {
            if let Some(inactive) = pipes.iter_mut().find(|p| !p.active) {
                inactive.active = true;
                let target_x = rightmost_x + pipe_spacing;
                inactive.warp_to(target_x);
                rightmost_x = target_x;
            }
        }

        for p in pipes.iter_mut().filter(|p| p.active) {
            let px = p.x();

            if !p.scored && (px + (TILESET_TILE_SIZE as i16 * 2)) < bird_x {
                p.scored = true;
                score += 1;
                can_increase_speed = true;
                draw_ui(score);
            }

            // Left exit in world space: -80px is offscreen left
            if px < -80 {
                let target_x = rightmost_x.max(VIEW_SCREEN_W as i16) + pipe_spacing;
                p.warp_to(target_x);
                rightmost_x = target_x;
            }
        }

        // 3. Exact Collision Checks
        if !no_collisions {
            let bx = bird_sprite.position[0];
            let by = bird_sprite.position[1];
            let bw = TILESET_TILE_SIZE as i16;
            let bh = TILESET_TILE_SIZE as i16;

            for p in pipes.iter().filter(|p| p.active) {
                let px = p.x();
                let pw = (TILESET_TILE_SIZE * 2) as i16;

                let h_overlap = (bx + bw - NICE_COLLISION_MARGIN) > px
                    && (bx + NICE_COLLISION_MARGIN) < (px + pw);

                if h_overlap {
                    let in_gap = (by + NICE_COLLISION_MARGIN) >= p.gap_y
                        && (by + bh - NICE_COLLISION_MARGIN) <= (p.gap_y + p.gap_size);

                    if !in_gap {
                        break 'gameloop;
                    }
                }
            }
        }

        // 4. Ground Scrolling
        ground_scroll -= current_speed;
        ground_map.set_origin(ground_scroll as i32, 0);
        engine.mark_row_dirty(ground_row);

        // 5. Cloud Drifting
        for c in clouds.iter_mut() {
            c.update(frame);
            if c.position[0] < -40 {
                let rx = VIEW_SCREEN_W as i16;
                let ry = randint(5, 45) as i16;
                c.set_position([rx, ry], None);
            }
        }

        // 6. Dynamic Speed Progression
        if can_increase_speed && score != 0 && score.is_multiple_of(speed_increase) {
            current_speed *= 1.15;
            for p in pipes.iter_mut() {
                p.set_speed(current_speed);
            }
            can_increase_speed = false;
        }

        // 7. Assemble Engine Render List (using Renderable::Sprite for world space entities!)
        {
            let mut render_list: Vec<GameRenderable, 36> = Vec::new();

            let _ = render_list.push(Renderable::Tilemap(&bg_map));

            for c in clouds.iter() {
                let _ = render_list.push(Renderable::Sprite(c));
            }

            for p in pipes.iter().filter(|p| p.active) {
                let _ = render_list.push(Renderable::Sprite(&p.spr_top_shaft));
                let _ = render_list.push(Renderable::Sprite(&p.spr_top_lip));
                let _ = render_list.push(Renderable::Sprite(&p.spr_bot_lip));
                let _ = render_list.push(Renderable::Sprite(&p.spr_bot_shaft));
            }

            let _ = render_list.push(Renderable::Tilemap(&ground_map));
            let _ = render_list.push(Renderable::Sprite(&bird_sprite));

            let scratch = unsafe { &mut *(&raw mut SCRATCH_BUFFER) };
            engine.render_frame(&mut render_list, scratch);
        }

        // 8. Commit Frames
        bird_sprite.commit_frame();
        for c in clouds.iter_mut() {
            c.commit_frame();
        }
        for p in pipes.iter_mut().filter(|p| p.active) {
            p.commit_frame();
        }

        frame_counter = frame_counter.wrapping_add(1);
    }

    // Switch the player sprite to the dead bird animation and render one final frame
    bird_sprite.set_animation(&ANIM_BIRD_DEAD);

    {
        let mut render_list: Vec<GameRenderable, 36> = Vec::new();

        let _ = render_list.push(Renderable::Tilemap(&bg_map));

        for c in clouds.iter() {
            let _ = render_list.push(Renderable::Sprite(c));
        }

        for p in pipes.iter().filter(|p| p.active) {
            let _ = render_list.push(Renderable::Sprite(&p.spr_top_shaft));
            let _ = render_list.push(Renderable::Sprite(&p.spr_top_lip));
            let _ = render_list.push(Renderable::Sprite(&p.spr_bot_lip));
            let _ = render_list.push(Renderable::Sprite(&p.spr_bot_shaft));
        }

        let _ = render_list.push(Renderable::Tilemap(&ground_map));
        let _ = render_list.push(Renderable::Sprite(&bird_sprite));

        let scratch = unsafe { &mut *(&raw mut SCRATCH_BUFFER) };
        engine.render_frame(&mut render_list, scratch);
    }

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
