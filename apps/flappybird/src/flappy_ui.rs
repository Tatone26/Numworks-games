use num_engine::{define_anim, define_repeating_anim, emitter, graphics::particles::Emitter};
use numworks_utils::{
    eadk::{
        display::{draw_string, push_rect_uniform, wait_for_vblank, SCREEN_HEIGHT, SCREEN_WIDTH},
        timing, Color, Point, Rect,
    },
    graphical::tiling::Tileset,
    include_bytes_align_as,
    utils::{get_string_pixel_size, string_from_u16, CENTER},
};

use crate::game::{GameEngine, GameWorld, TILE_SIZE, VIEW_SCREEN_H, VIEW_SCREEN_W, WINDOW_SIZE};

pub const TILESET_TILE_SIZE: u16 = TILE_SIZE as u16;

const IMAGE_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/image.nppm");
pub static TILESET: Tileset = Tileset::new(TILESET_TILE_SIZE, 4, IMAGE_BYTES);

pub const BACKGROUND: Color = Color::from_rgb888(128, 212, 255);
pub const UI_BACKGROUND: Color = Color::from_rgb888(50, 50, 50);

// Wind streak palettes
pub const COLOR_TAILWIND: Color = Color::from_rgb888(235, 245, 255);
pub const COLOR_HEADWIND: Color = COLOR_TAILWIND;
pub const COLOR_LOW_GRAV: Color = Color::from_rgb888(235, 245, 205);

#[inline(always)]
pub const fn tailwind_emitter() -> Emitter {
    emitter!(
        area: [-20, 50, 10, VIEW_SCREEN_H - 50],
        rate: 1,
        interval: 3,
        vx: [6.5, 9.5],
        vy: [0.0, 0.0],
        width: [8, 16],
        height: [1, 1],
        color: COLOR_TAILWIND,
        life: [25, 38],
    )
}

#[inline(always)]
pub const fn headwind_emitter() -> Emitter {
    emitter!(
        area: [VIEW_SCREEN_W as i16 + 50, 50, 10, VIEW_SCREEN_H - 50],
        rate: 1,
        interval: 3,
        vx: [-9.5, -6.5],
        vy: [0.0, 0.0],
        width: [10, 18],
        height: [1, 1],
        color: COLOR_HEADWIND,
        life: [25, 38],
    )
}

#[inline(always)]
pub const fn low_gravity_emitter() -> Emitter {
    emitter!(
        area: [30, VIEW_SCREEN_H as i16 - 5, VIEW_SCREEN_W as i16 - 70, 10],
        rate: 1,
        interval: 3,
        vx: [0.0, 0.0],
        vy: [-9.5, -6.5],
        width: [1, 1],
        height: [10, 18],
        color: COLOR_LOW_GRAV,
        life: [25, 38],
    )
}

#[inline(always)]
pub const fn high_gravity_emitter() -> Emitter {
    emitter!(
        area: [30, -5, VIEW_SCREEN_W as i16 - 70, 10],
        rate: 1,
        interval: 3,
        vx: [0.0, 0.0],
        vy: [9.5, 6.5],
        width: [1, 1],
        height: [10, 18],
        color: COLOR_LOW_GRAV,
        life: [25, 38],
    )
}

define_anim!(pub ANIM_BIRD_FLAP_UP, &TILESET, 1, 1, true, 1, [(3, 0)]);
define_anim!(pub ANIM_BIRD_FALL, &TILESET, 1, 1, true, 1, [(0, 0)]);
define_anim!(pub ANIM_BIRD_DEAD, &TILESET, 1, 1, true, 1, [(0, 3)]);

define_anim!(pub ANIM_CLOUD, &TILESET, 2, 1, true, 1, [(1, 3)]);

define_anim!(pub ANIM_PIPE_LIP_TOP, &TILESET, 4, 1, true, 1, [(0, 1)]);
define_anim!(pub ANIM_PIPE_LIP_BOT, &TILESET, 4, 1, true, 1, [(0, 2)]);

define_repeating_anim!(pub ANIM_PIPE_SHAFT, &TILESET, 2, 11, 2, 1, false, 1, [(1, 0)]);
define_anim!(pub ANIM_PIPE_SHAFT_LITTLE, &TILESET, 2, 1, false, 1, [(1, 0)]);

pub fn draw_constant_ui(high_score: u16) {
    push_rect_uniform(
        Rect {
            x: 0,
            y: 0,
            width: SCREEN_WIDTH,
            height: WINDOW_SIZE,
        },
        UI_BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: 0,
            y: SCREEN_HEIGHT - TILESET_TILE_SIZE / 2,
            width: SCREEN_WIDTH - 2 * WINDOW_SIZE,
            height: TILESET_TILE_SIZE / 2,
        },
        UI_BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: SCREEN_WIDTH - WINDOW_SIZE,
            y: WINDOW_SIZE,
            width: TILESET_TILE_SIZE,
            height: SCREEN_HEIGHT,
        },
        UI_BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: 0,
            y: WINDOW_SIZE,
            width: TILESET_TILE_SIZE,
            height: SCREEN_HEIGHT,
        },
        UI_BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: 0,
            y: SCREEN_HEIGHT - WINDOW_SIZE + TILESET_TILE_SIZE / 2,
            width: SCREEN_WIDTH - WINDOW_SIZE,
            height: TILESET_TILE_SIZE / 2,
        },
        UI_BACKGROUND,
    );
    draw_string(
        "Score: \0",
        Point { x: 5, y: 1 },
        true,
        Color::WHITE,
        UI_BACKGROUND,
    );
    let s = string_from_u16(high_score);
    draw_string(
        "Best: \0",
        Point {
            x: SCREEN_WIDTH
                - 5
                - get_string_pixel_size("Best: \0", true)
                - get_string_pixel_size(&s, true),
            y: 1,
        },
        true,
        Color::WHITE,
        UI_BACKGROUND,
    );
    draw_string(
        &string_from_u16(high_score),
        Point {
            x: SCREEN_WIDTH - 5 - get_string_pixel_size(&s, true),
            y: 1,
        },
        true,
        Color::WHITE,
        UI_BACKGROUND,
    );
}

pub fn draw_ui(score: u16) {
    draw_string(
        &string_from_u16(score),
        Point {
            x: 5 + get_string_pixel_size("Score: \0", true),
            y: 1,
        },
        true,
        Color::WHITE,
        UI_BACKGROUND,
    );
}

pub fn countdown(pos: Point, engine: &mut GameEngine, world: &mut GameWorld) {
    for n in (1..=3).rev() {
        engine.mark_all_dirty();
        engine.render_progressive(world);

        wait_for_vblank();
        TILESET.draw_tile(pos, Point { x: n, y: 4 }, 2, true);

        timing::msleep(800);
    }

    engine.mark_all_dirty();
    engine.render_progressive(world);
}

pub fn menu_vis_addon() {
    ANIM_PIPE_LIP_BOT.draw_at(
        Point {
            x: CENTER.x + 10,
            y: 97 - TILESET_TILE_SIZE,
        },
        0,
    );

    ANIM_PIPE_SHAFT_LITTLE.draw_at(
        Point {
            x: CENTER.x + 10 + TILESET_TILE_SIZE,
            y: 97,
        },
        0,
    );

    ANIM_CLOUD.draw_at(
        Point {
            x: CENTER.x - 95,
            y: 25,
        },
        0,
    );

    ANIM_BIRD_FALL.draw_at(
        Point {
            x: CENTER.x - 35,
            y: 15 + TILESET_TILE_SIZE + TILESET_TILE_SIZE,
        },
        0,
    );
}
