use heapless::{String, Vec};
use num_engine::{
    ascii_tilemap,
    compositor::{InterlaceMode, Renderable},
    define_anim,
    engine::Engine,
    spawn_sprite,
    sprite::Sprite,
    tile_span,
    tilemap::{Parallax, Tilemap},
};
use numworks_utils::{
    eadk::{
        display::{push_rect, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, Color, Rect,
    },
    graphical::{draw_centered_string, fading, tiling::Tileset, ColorConfig},
    include_bytes_align_as,
    menu::{
        selection,
        settings::{write_values_to_file, Setting},
        start_menu, MenuConfig,
    },
    utils::{randint, string_from_u16, LARGE_CHAR_HEIGHT},
};

// -----------------------------------------------------------------------------
// Engine Dimension & World Constants
// -----------------------------------------------------------------------------
const TILE_SIZE: usize = 20;
const CELL_AREA: usize = TILE_SIZE * TILE_SIZE; // 400

const VIEW_SCREEN_X: u16 = 0;
const VIEW_SCREEN_Y: u16 = 40;
const VIEW_SCREEN_W: u16 = SCREEN_WIDTH;
const VIEW_SCREEN_H: u16 = SCREEN_HEIGHT - VIEW_SCREEN_Y; // 200

const SCREEN_COLS: usize = VIEW_SCREEN_W as usize / TILE_SIZE; // 16
const SCREEN_ROWS: usize = VIEW_SCREEN_H as usize / TILE_SIZE; // 10

const WORLD_COLS: usize = 32;
const WORLD_ROWS: usize = 24;
const WORLD_TILES: usize = WORLD_COLS * WORLD_ROWS;
const WORLD_WIDTH_PX: i32 = (WORLD_COLS * TILE_SIZE) as i32; // 640
const WORLD_HEIGHT_PX: i32 = (WORLD_ROWS * TILE_SIZE) as i32; // 480

// Midground hills layer (32x6 tiles)
const HILLS_COLS: usize = 32;
const HILLS_ROWS: usize = 6;
const HILLS_TILES: usize = HILLS_COLS * HILLS_ROWS;

// Deadzone bounds inside the 320x200 viewport window
const DEADZONE_LEFT: i32 = 70;
const DEADZONE_RIGHT: i32 = (VIEW_SCREEN_W as i32) - 100 - (TILE_SIZE as i32);
const DEADZONE_TOP: i32 = 40;
const DEADZONE_BOTTOM: i32 = (VIEW_SCREEN_H as i32) - 40 - (TILE_SIZE as i32);

const DEBUG_MODE: bool = true;
const NUM_SWARM: usize = 16;
const NUM_CLOUDS: usize = 4;

const IMAGE_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/image.nppm");
static TILESET: Tileset = Tileset::new(TILE_SIZE as u16, 4, IMAGE_BYTES);

type GameEngine = Engine<TILE_SIZE, CELL_AREA, SCREEN_COLS, SCREEN_ROWS, DEBUG_MODE>;
type GameTilemap<'a> = Tilemap<'a, TILE_SIZE, CELL_AREA>;
type GameSprite<'a> = Sprite<'a, TILE_SIZE, CELL_AREA>;
type GameRenderable<'r, 'a> = Renderable<'r, 'a, TILE_SIZE, CELL_AREA>;

const MAX_CHUNKS: usize = if SCREEN_COLS > SCREEN_ROWS {
    SCREEN_COLS
} else {
    SCREEN_ROWS
};
const SCRATCH_PIXELS: usize = CELL_AREA + (MAX_CHUNKS * CELL_AREA);
static mut SCRATCH_BUFFER: [Color; SCRATCH_PIXELS] = [Color::BLACK; SCRATCH_PIXELS];

const HUD_BG_COLOR: Color = Color::from_rgb888(220, 220, 220);

const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: Color::from_rgb888(200, 200, 200),
    alt: Color::from_rgb888(0, 140, 60),
};

fn vis_addon() {}

// -----------------------------------------------------------------------------
// Declarative Animations Stored in Flash (ROM)
// -----------------------------------------------------------------------------

// Player flying: 1x1 tile (20x20 px), transparent, 6 ticks/frame
define_anim!(ANIM_PLAYER_FLY, &TILESET, 1, 1, true, 6, [(0, 0), (3, 0)]);

// Player dead: 1x1 tile, transparent, static frame
define_anim!(ANIM_PLAYER_DEAD, &TILESET, 1, 1, true, 1, [(0, 3)]);

// Cloud sprite: 2x1 tiles (40x20 px), transparent, static frame
define_anim!(ANIM_CLOUD, &TILESET, 2, 1, true, 1, [(1, 3)]);

// UI Reticle: 1x1 tile, transparent, static frame
define_anim!(ANIM_RETICLE, &TILESET, 1, 1, true, 1, [(1, 4)]);

// Swarm enemy: 1x1 tile, transparent, static frame
define_anim!(ANIM_SWARM, &TILESET, 1, 1, true, 1, [(0, 3)]);

// -----------------------------------------------------------------------------
// Visual ASCII Tilemap Layouts
// -----------------------------------------------------------------------------

// 32 cols x 6 rows: Midground rolling hills
const HILLS_LAYOUT: &str = "\
................................
................................
HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH
GGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG
................................
................................";

// 32 cols x 24 rows: Foreground obstacles at columns 5, 10, 15, 20, 25, 30
const PIPE_LAYOUT: &str = "\
.....AB...AB...AB...AB...AB...AB
.....AB...AB...AB...AB...AB...AB
.....AB...AB...AB...AB...AB...AB
.....AB...AB...AB...AB...AB...AB
.....CD...CD...CD...CD...CD...CD
................................
................................
................................
................................
................................
................................
................................
................................
................................
................................
................................
.....EF...EF...EF...EF...EF...EF
.....AB...AB...AB...AB...AB...AB
.....AB...AB...AB...AB...AB...AB
.....AB...AB...AB...AB...AB...AB
.....AB...AB...AB...AB...AB...AB
.....AB...AB...AB...AB...AB...AB
.....AB...AB...AB...AB...AB...AB";

/// Clears the dedicated top HUD rectangle outside the game viewport
fn clear_hud_bar() {
    let fill_row = [HUD_BG_COLOR; SCREEN_WIDTH as usize];
    for y in 0..VIEW_SCREEN_Y {
        push_rect(
            Rect {
                x: 0,
                y,
                width: SCREEN_WIDTH,
                height: 1,
            },
            &fill_row,
        );
    }
}

// -----------------------------------------------------------------------------
// Menu & Options Entry Point
// -----------------------------------------------------------------------------
pub fn start() {
    let mut opt: [&mut Setting; 3] = [
        &mut Setting {
            name: "Speed\0",
            choice: 1,
            values: Vec::from_slice(&[3, 2, 1]).unwrap(),
            texts: Vec::from_slice(&["Fast\0", "Normal\0", "Slow\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "Hazards\0",
            choice: 0,
            values: Vec::from_slice(&[1, 0]).unwrap(),
            texts: Vec::from_slice(&["Yes\0", "No\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "High Score\0",
            choice: 0,
            values: Vec::from_slice(&[0, 0, u32::MAX]).unwrap(),
            texts: Vec::new(),
            user_modifiable: false,
            fixed_values: false,
        },
    ];

    loop {
        let start_choice = start_menu(
            "NUM-ENGINE DEMO\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("data/model_controls.txt"),
            "num_engine_demo",
        );

        if start_choice == 0 {
            loop {
                let speed = opt[0].get_setting_value() as f32;
                let has_hazards = opt[1].get_setting_value() != 0;
                let mut high_score = opt[2].get_setting_value();

                let action = game(speed, has_hazards, &mut high_score);

                opt[2].set_value(high_score);
                write_values_to_file(&mut opt, "num_engine_demo");

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
pub fn game(speed_factor: f32, has_hazards: bool, high_score: &mut u32) -> u8 {
    clear_hud_bar();

    let mut engine = GameEngine::new_with_window(
        Color::from_rgb888(135, 206, 235),
        VIEW_SCREEN_X,
        VIEW_SCREEN_Y,
        VIEW_SCREEN_W,
        VIEW_SCREEN_H,
        Some(InterlaceMode::Rows),
    );

    // Tile storage buffers owned on stack
    let mut bg_data = [Some([3u8, 3u8]); WORLD_TILES];
    let mut hills_data = [None::<[u8; 2]>; HILLS_TILES];
    let mut pipe_data = [None::<[u8; 2]>; WORLD_TILES];

    // 1. Base Layer (Opaque: bypasses buffer.fill(clear_color))
    let ground_row = WORLD_ROWS - 1;
    let mut bg_map = GameTilemap::new(
        &TILESET,
        &mut bg_data,
        WORLD_COLS,
        WORLD_ROWS,
        &[],
        0,
        false,
        Parallax::FOREGROUND,
        false,
    );
    tile_span!(bg_map, row: ground_row, cols: 0..WORLD_COLS, tile: Some([0, 4]));

    // 2. Parallax Hills Layer from ASCII layout (0.5x speed, repeats infinitely)
    let mut hills_map = ascii_tilemap!(
        tileset: &TILESET,
        buffer: &mut hills_data,
        cols: HILLS_COLS,
        rows: HILLS_ROWS,
        z: 2,
        transparent: true,
        parallax: Parallax::from_distance(2),
        wrap: true,
        mapping: {
            b'.' => None,
            b'H' => Some([1, 2]),
            b'G' => Some([1, 0]),
        },
        layout: HILLS_LAYOUT,
    );
    hills_map.set_origin(0, (WORLD_HEIGHT_PX - (HILLS_ROWS * TILE_SIZE) as i32) - 20);

    // 3. Foreground Obstacles from ASCII layout
    let pipe_map = ascii_tilemap!(
        tileset: &TILESET,
        buffer: &mut pipe_data,
        cols: WORLD_COLS,
        rows: WORLD_ROWS,
        z: 10,
        transparent: true,
        parallax: Parallax::FOREGROUND,
        wrap: false,
        mapping: {
            b'.' => None,
            b'A' => Some([1, 0]),
            b'B' => Some([2, 0]),
            b'C' => Some([1, 1]),
            b'D' => Some([2, 1]),
            b'E' => Some([1, 2]),
            b'F' => Some([2, 2]),
        },
        layout: PIPE_LAYOUT,
    );

    // 4. Sprites (Spawned instantly from ROM animations)
    let mut player = spawn_sprite!(&ANIM_PLAYER_FLY, [100, 140], 20);
    let mut ui_marker = spawn_sprite!(&ANIM_RETICLE, [8, 8], 25);

    // Multi-tile drifting clouds
    let mut clouds: Vec<GameSprite, NUM_CLOUDS> = Vec::new();
    let mut cloud_speeds = [0.0f32; NUM_CLOUDS];

    for i in 0..NUM_CLOUDS {
        let cx = (i as i16 * 160) + randint(10, 40) as i16;
        let cy = randint(15, 90) as i16;
        let mut c = spawn_sprite!(&ANIM_CLOUD, [cx, cy], 3);
        let drift_speed = -((randint(20, 50) as f32) / 100.0);
        c.set_speed([drift_speed, 0.0]);
        cloud_speeds[i] = drift_speed;
        let _ = clouds.push(c);
    }

    // Swarm Entities
    let active_swarm_count = if has_hazards { NUM_SWARM } else { 0 };
    let mut swarm_vel = [[0.0f32; 2]; NUM_SWARM];
    let mut swarm: Vec<GameSprite, NUM_SWARM> = Vec::new();

    for i in 0..active_swarm_count {
        let spawn_x = randint(60, (WORLD_WIDTH_PX - 60) as u32) as i16;
        let spawn_y = randint(40, (WORLD_HEIGHT_PX - 80) as u32) as i16;

        let raw_vx = (randint(0, 300) as i32 - 150) as f32 / 100.0 * speed_factor;
        let raw_vy = (randint(0, 300) as i32 - 150) as f32 / 100.0 * speed_factor;

        swarm_vel[i] = [
            if raw_vx == 0.0 { 1.0 } else { raw_vx },
            if raw_vy == 0.0 { -1.0 } else { raw_vy },
        ];

        let s = spawn_sprite!(&ANIM_SWARM, [spawn_x, spawn_y], 15);
        let _ = swarm.push(s);
    }

    engine.mark_all_dirty();

    let mut score: u16 = 0;
    let mut prev_score: u16 = u16::MAX;
    let mut is_dead = false;

    // -------------------------------------------------------------------------
    // Main Simulation Loop
    // -------------------------------------------------------------------------
    loop {
        let keyboard_state = keyboard::scan();

        if keyboard_state.key_down(key::BACK) {
            let action = pause_menu(score, false, high_score);
            if action != 0 {
                return action;
            }
            clear_hud_bar();
            engine.mark_all_dirty();
            prev_score = u16::MAX;
        }

        let frame = engine.get_frame();

        // 1. Cloud Drifting & Wrapping
        for i in 0..NUM_CLOUDS {
            clouds[i].set_speed([cloud_speeds[i], 0.0]);
            clouds[i].update(frame);

            if clouds[i].position[0] < -40 {
                let new_y = randint(15, 90) as i16;
                clouds[i].set_position([WORLD_WIDTH_PX as i16, new_y], None);
            }
        }

        // 2. 4-Directional Player Movement
        let move_speed = 2.0 * speed_factor;
        let mut p_vx = 0.0f32;
        let mut p_vy = 0.0f32;

        if keyboard_state.key_down(key::LEFT) {
            p_vx -= move_speed;
        }
        if keyboard_state.key_down(key::RIGHT) {
            p_vx += move_speed;
        }
        if keyboard_state.key_down(key::UP) {
            p_vy -= move_speed;
        }
        if keyboard_state.key_down(key::DOWN) {
            p_vy += move_speed;
        }

        player.set_speed([p_vx, p_vy]);
        player.update(frame);

        // Clamping to World horizontal and top boundaries
        let clamped_px = player.position[0].clamp(0, (WORLD_WIDTH_PX - TILE_SIZE as i32) as i16);
        let mut clamped_py = player.position[1];
        if clamped_py < 0 {
            clamped_py = 0;
        }

        if clamped_px != player.position[0] || clamped_py != player.position[1] {
            player.set_position([clamped_px, clamped_py], None);
        }

        // 3. Ground Collision (Exclusive Death Condition)
        let ground_pixel_y = (ground_row * TILE_SIZE) as i32;
        if (player.position[1] as i32) + (TILE_SIZE as i32) >= ground_pixel_y {
            is_dead = true;
        }

        if frame % 30 == 0 {
            score = score.saturating_add(1);
        }

        // 4. Viewport Tracking (Deadzone)
        {
            let viewport = engine.get_mut_viewport();
            let screen_px = (player.position[0] as i32) - viewport.x;
            let screen_py = (player.position[1] as i32) - viewport.y;

            let mut cam_dx = 0;
            let mut cam_dy = 0;

            if screen_px < DEADZONE_LEFT {
                cam_dx = screen_px - DEADZONE_LEFT;
            } else if screen_px > DEADZONE_RIGHT {
                cam_dx = screen_px - DEADZONE_RIGHT;
            }

            if screen_py < DEADZONE_TOP {
                cam_dy = screen_py - DEADZONE_TOP;
            } else if screen_py > DEADZONE_BOTTOM {
                cam_dy = screen_py - DEADZONE_BOTTOM;
            }

            if cam_dx != 0 || cam_dy != 0 {
                viewport.move_by(cam_dx, cam_dy);
                viewport.clamp(0, WORLD_WIDTH_PX, 0, WORLD_HEIGHT_PX);
            }
        }

        // 5. UI Reticle Update
        ui_marker.set_speed([0.0, 0.0]);
        ui_marker.update(frame);

        // 6. Swarm Movement & Boundaries (Demo entities bounce inside play bounds)
        for i in 0..active_swarm_count {
            swarm[i].set_speed(swarm_vel[i]);
            swarm[i].update(frame);

            let mut cur_x = swarm[i].position[0];
            let mut cur_y = swarm[i].position[1];
            let mut bounced = false;

            if cur_x <= 0 {
                cur_x = 1;
                swarm_vel[i][0] = -swarm_vel[i][0];
                bounced = true;
            } else if cur_x >= (WORLD_WIDTH_PX - TILE_SIZE as i32) as i16 {
                cur_x = (WORLD_WIDTH_PX - TILE_SIZE as i32 - 1) as i16;
                swarm_vel[i][0] = -swarm_vel[i][0];
                bounced = true;
            }

            if cur_y <= 0 {
                cur_y = 1;
                swarm_vel[i][1] = -swarm_vel[i][1];
                bounced = true;
            } else if cur_y >= (ground_pixel_y - TILE_SIZE as i32) as i16 {
                cur_y = (ground_pixel_y - TILE_SIZE as i32 - 1) as i16;
                swarm_vel[i][1] = -swarm_vel[i][1];
                bounced = true;
            }

            if bounced {
                swarm[i].set_position([cur_x, cur_y], None);
            }
        }

        // 7. Standalone HUD Refresh
        if score != prev_score {
            let mut hud_text: String<32> = String::new();
            hud_text.push_str("SCORE: ").unwrap();
            hud_text
                .push_str(
                    string_from_u16(score)
                        .as_str()
                        .split_terminator('\0')
                        .next()
                        .unwrap(),
                )
                .unwrap();
            hud_text.push_str(" | NUMWORKS 2D\0").unwrap();

            draw_centered_string(
                &hud_text,
                12,
                false,
                &ColorConfig {
                    text: Color::BLACK,
                    bckgrd: HUD_BG_COLOR,
                    alt: Color::from_rgb888(50, 150, 50),
                },
                true,
            );
            prev_score = score;
        }

        // 8. Assemble Render List
        {
            let mut render_list: Vec<GameRenderable, { NUM_SWARM + NUM_CLOUDS + 5 }> = Vec::new();

            let _ = render_list.push(Renderable::Tilemap(&bg_map));
            let _ = render_list.push(Renderable::Tilemap(&hills_map));

            for c in clouds.iter() {
                let _ = render_list.push(Renderable::Sprite(c));
            }

            let _ = render_list.push(Renderable::Tilemap(&pipe_map));

            let _ = render_list.push(Renderable::Sprite(&player));
            for s in swarm.iter() {
                let _ = render_list.push(Renderable::Sprite(s));
            }
            let _ = render_list.push(Renderable::UiSprite(&ui_marker));

            let scratch = unsafe { &mut *(&raw mut SCRATCH_BUFFER) };
            engine.render_frame(&mut render_list, scratch);
        }

        // 9. Frame Commits
        player.commit_frame();
        ui_marker.commit_frame();
        for c in clouds.iter_mut() {
            c.commit_frame();
        }
        for s in swarm.iter_mut() {
            s.commit_frame();
        }

        // Handle Death Sequence
        if is_dead {
            player.set_animation(&ANIM_PLAYER_DEAD);
            player.update(frame + 1);

            let mut death_list: Vec<GameRenderable, 2> = Vec::new();
            let _ = death_list.push(Renderable::Tilemap(&pipe_map));
            let _ = death_list.push(Renderable::Sprite(&player));
            let scratch = unsafe { &mut *(&raw mut SCRATCH_BUFFER) };
            engine.render_frame(&mut death_list, scratch);

            fading(300);
            break;
        }
    }

    // -------------------------------------------------------------------------
    // Game Over Screen & Score Processing
    // -------------------------------------------------------------------------
    draw_centered_string(
        " GAME OVER! \0",
        SCREEN_HEIGHT / 3 - LARGE_CHAR_HEIGHT,
        true,
        &ColorConfig {
            text: Color::BLACK,
            bckgrd: COLOR_CONFIG.bckgrd,
            alt: Color::RED,
        },
        true,
    );

    if score > *high_score as u16 {
        draw_centered_string(
            "NEW HIGH SCORE!\0",
            SCREEN_HEIGHT / 3 + 2,
            true,
            &ColorConfig {
                text: Color::BLACK,
                bckgrd: COLOR_CONFIG.bckgrd,
                alt: Color::RED,
            },
            true,
        );
        *high_score = score as u32;
    }

    pause_menu(score, true, high_score)
}

fn pause_menu(points: u16, death: bool, high_score: &u32) -> u8 {
    let mut string_points: String<16> = String::new();
    string_points.push_str(" Score : ").unwrap();
    string_points
        .push_str(
            string_from_u16(points)
                .as_str()
                .split_terminator('\0')
                .next()
                .unwrap(),
        )
        .unwrap();
    string_points.push_str(" \0").unwrap();
    draw_centered_string(&string_points, 5, true, &COLOR_CONFIG, false);

    let mut string_high_score: String<22> = String::new();
    string_high_score.push_str(" High Score : ").unwrap();
    string_high_score
        .push_str(
            string_from_u16(*high_score as u16)
                .as_str()
                .split_terminator('\0')
                .next()
                .unwrap(),
        )
        .unwrap();
    string_high_score.push_str(" \0").unwrap();
    draw_centered_string(
        &string_high_score,
        5 + LARGE_CHAR_HEIGHT + 2,
        true,
        &COLOR_CONFIG,
        false,
    );

    let action = selection(
        &COLOR_CONFIG,
        &MenuConfig {
            choices: if death {
                &["Play again\0", "Menu\0", "Exit\0"]
            } else {
                &["Resume\0", "Menu\0", "Exit\0"]
            },
            rect_margins: (20, 12),
            dimensions: (SCREEN_WIDTH * 7 / 15, LARGE_CHAR_HEIGHT * 7),
            offset: (0, if death { 50 } else { 0 }),
            back_key_return: if death { 2 } else { 0 },
        },
        false,
    );

    if action != 0 {
        fading(500);
    }
    action
}
