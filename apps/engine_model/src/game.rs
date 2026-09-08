use heapless::{String, Vec};
use num_engine::{
    compositor::Renderable,
    engine::Engine,
    sprite::Sprite,
    texture::{Animation, FrameCoord, TextureDescriptor, MAX_LENGTH_ANIMATION},
    tilemap::Tilemap,
};
use numworks_utils::{
    eadk::{
        display::{SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, Color, Point,
    },
    graphical::{draw_centered_string, fading, tiling::Tileset, ColorConfig},
    include_bytes_align_as,
    menu::{
        selection,
        settings::{write_values_to_file, Setting},
        start_menu, MenuConfig,
    },
    utils::{string_from_u16, LARGE_CHAR_HEIGHT},
};

// -----------------------------------------------------------------------------
// Engine Dimension Config
// -----------------------------------------------------------------------------
const TILE_SIZE: usize = 20;
const CELL_AREA: usize = TILE_SIZE * TILE_SIZE; // 400
const SCREEN_COLS: usize = SCREEN_WIDTH as usize / TILE_SIZE; // 16
const SCREEN_ROWS: usize = SCREEN_HEIGHT as usize / TILE_SIZE; // 12

const IMAGE_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/image.nppm");
static TILESET: Tileset = Tileset::new(TILE_SIZE as u16, 4, IMAGE_BYTES);

type GameEngine<'a> = Engine<TILE_SIZE, CELL_AREA, SCREEN_COLS, SCREEN_ROWS>;
type GameTilemap<'a> = Tilemap<'a, TILE_SIZE, CELL_AREA, SCREEN_COLS, SCREEN_ROWS>;
type GameSprite<'a> = Sprite<'a, TILE_SIZE, CELL_AREA>;
type GameRenderable<'r, 'a> = Renderable<'r, 'a, TILE_SIZE, CELL_AREA, SCREEN_COLS, SCREEN_ROWS>;

const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: Color::from_rgb888(200, 200, 200),
    alt: Color::from_rgb888(0, 140, 60),
};

fn vis_addon() {}
// -----------------------------------------------------------------------------
// Menu & Options Entry Point
// -----------------------------------------------------------------------------
#[no_mangle]
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
            "FLAPPY BIRD\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon, // Optional preview addon
            include_str!("data/model_controls.txt"),
            "flappy",
        );

        if start_choice == 0 {
            loop {
                let speed = opt[0].get_setting_value() as f32;
                let has_hazards = opt[1].get_setting_value() != 0;
                let mut high_score = opt[2].get_setting_value();

                let action = game(speed, has_hazards, &mut high_score);

                opt[2].set_value(high_score);
                write_values_to_file(&mut opt, "flappy");

                if action == 2 {
                    return; // Exit game
                } else if action == 1 {
                    break; // Return to start_menu
                }
                // action == 0: loops and restarts game immediately
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
    // 1. Background Tilemap: Sky + Floor (Z = 0, Opaque)
    let mut bg_data: [[Option<[u8; 2]>; SCREEN_ROWS]; SCREEN_COLS] =
        [[Some([3, 3]); SCREEN_ROWS]; SCREEN_COLS];
    for x in 0..SCREEN_COLS {
        bg_data[x][11] = Some([0, 4]);
    }
    let bg_tilemap = GameTilemap::new(&TILESET, bg_data, 0, false);

    // 2. Foreground Tilemap: Transparent Pipes (Z = 8, Transparent None cells)
    let mut pipe_data: [[Option<[u8; 2]>; SCREEN_ROWS]; SCREEN_COLS] =
        [[None; SCREEN_ROWS]; SCREEN_COLS];
    // Hanging ceiling pipe at column 12
    pipe_data[12][0] = Some([1, 0]);
    pipe_data[12][1] = Some([1, 0]);
    pipe_data[12][2] = Some([1, 1]);
    // Floor pipe at column 5
    pipe_data[5][8] = Some([1, 2]);
    pipe_data[5][9] = Some([1, 0]);
    pipe_data[5][10] = Some([1, 0]);
    let pipe_tilemap = GameTilemap::new(&TILESET, pipe_data, 8, true);

    let mut engine = GameEngine::new(Color::from_rgb888(135, 206, 235));

    // 3. Sprites Setup
    // Background cloud (Z = 2, behind pipes)
    let cloud_desc = TextureDescriptor::new(&TILESET, 2, 1, 1, true);
    let mut cloud_frames = Vec::<FrameCoord, MAX_LENGTH_ANIMATION>::new();
    let _ = cloud_frames.push(FrameCoord { tx: 1, ty: 3 });
    let mut cloud_anim = Animation::new(cloud_desc, cloud_frames, 1);
    let mut cloud = GameSprite::new(&mut cloud_anim, Point::new(180, 30), 2);
    cloud.speed = [-0.4 * speed_factor, 0.0];

    // Player (Z = 10, flies in front of pipes)
    let player_desc = TextureDescriptor::new(&TILESET, 1, 1, 1, true);
    let mut player_frames = Vec::<FrameCoord, MAX_LENGTH_ANIMATION>::new();
    let _ = player_frames.push(FrameCoord { tx: 0, ty: 0 });
    let _ = player_frames.push(FrameCoord { tx: 3, ty: 0 });
    let mut player_anim = Animation::new(player_desc, player_frames, 8);
    let mut player = GameSprite::new(&mut player_anim, Point::new(40, 80), 10);

    // Hazard (Z = 6, flies behind pipes)
    let dead_desc = TextureDescriptor::new(&TILESET, 1, 1, 1, true);
    let mut dead_frames = Vec::<FrameCoord, MAX_LENGTH_ANIMATION>::new();
    let _ = dead_frames.push(FrameCoord { tx: 0, ty: 2 });
    let mut dead_anim = Animation::new(dead_desc, dead_frames, 1);
    let mut dead_bird = GameSprite::new(&mut dead_anim, Point::new(160, 40), 6);
    dead_bird.speed = [0.8 * speed_factor, 1.2 * speed_factor];

    // Mark initial regions dirty to stamp entities on frame 0
    engine
        .compositor
        .grid
        .mark_rect(cloud.position, cloud.pixel_width(), cloud.pixel_height());
    engine
        .compositor
        .grid
        .mark_rect(player.position, player.pixel_width(), player.pixel_height());
    if has_hazards {
        engine.compositor.grid.mark_rect(
            dead_bird.position,
            dead_bird.pixel_width(),
            dead_bird.pixel_height(),
        );
    }

    let mut score: u16 = 0;

    loop {
        let keyboard_state = keyboard::scan();

        if keyboard_state.key_down(key::BACK) {
            let action = pause_menu(score, false, high_score);
            if action != 0 {
                return action;
            }
            // If resume, redraw all layers
            engine.mark_all_dirty();
        }

        // --- Controls ---
        let mut mx = 0.0;
        let mut my = 0.0;
        if keyboard_state.key_down(key::LEFT) {
            mx -= 1.5 * speed_factor;
        }
        if keyboard_state.key_down(key::RIGHT) {
            mx += 1.5 * speed_factor;
        }
        if keyboard_state.key_down(key::UP) {
            my -= 1.5 * speed_factor;
        }
        if keyboard_state.key_down(key::DOWN) {
            my += 1.5 * speed_factor;
        }
        player.speed = [mx, my];

        // --- Entity Logic ---
        if cloud.position.x <= 20 {
            cloud.set_position(
                Point::new((SCREEN_WIDTH - 30) as u16, cloud.position.y),
                None,
            );
        }

        if has_hazards {
            if dead_bird.position.x <= 10 || dead_bird.position.x >= (SCREEN_WIDTH - 30) as u16 {
                dead_bird.speed[0] = -dead_bird.speed[0];
            }
            if dead_bird.position.y <= 10 || dead_bird.position.y >= (SCREEN_HEIGHT - 50) as u16 {
                dead_bird.speed[1] = -dead_bird.speed[1];
            }
            dead_bird.update(engine.frame);
        }

        player.update(engine.frame);
        cloud.update(engine.frame);

        // Survival score increment
        if engine.frame % 30 == 0 {
            score = score.saturating_add(1);
        }

        // --- Collision Check (Game Over Condition) ---
        let px = player.position.x + 4;
        let py = player.position.y + 4;
        let p_cx = (px as usize) / TILE_SIZE;
        let p_cy = (py as usize) / TILE_SIZE;

        let hit_pipe = pipe_tilemap.get_tile(p_cx, p_cy).is_some();
        let hit_floor = player.position.y >= (11 * TILE_SIZE - 16) as u16;

        if hit_pipe || hit_floor {
            break;
        }

        // --- Render Pass ---
        if has_hazards {
            let mut render_list: [GameRenderable; 5] = [
                Renderable::Tilemap(&bg_tilemap),
                Renderable::Tilemap(&pipe_tilemap),
                Renderable::Sprite(&cloud),
                Renderable::Sprite(&dead_bird),
                Renderable::Sprite(&player),
            ];
            engine.render_frame(&mut render_list);
        } else {
            let mut render_list: [GameRenderable; 4] = [
                Renderable::Tilemap(&bg_tilemap),
                Renderable::Tilemap(&pipe_tilemap),
                Renderable::Sprite(&cloud),
                Renderable::Sprite(&player),
            ];
            engine.render_frame(&mut render_list);
        }

        // --- Advance State ---
        player.commit_frame();
        cloud.commit_frame();
        if has_hazards {
            dead_bird.commit_frame();
        }
    }

    // --- Game Over Screen ---
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

// -----------------------------------------------------------------------------
// In-Engine Pause and Game-Over Popup
// -----------------------------------------------------------------------------
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
