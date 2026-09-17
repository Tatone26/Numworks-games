use num_engine::{define_anim, graphics::texture::Animation};
use numworks_utils::{
    eadk::{
        display::{draw_string, push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        Color, Point, Rect,
    },
    graphical::tiling::Tileset,
    include_bytes_align_as,
    utils::{get_string_pixel_size, string_from_u16, string_from_u32, CENTER, SMALL_CHAR_HEIGHT},
};

use crate::game::{TILE_SIZE, VIEW_SCREEN_X};

pub const WALL_IMAGES_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/walls.nppm");
pub const SPRITES_IMAGES_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/sprites.nppm");

pub static TILESET_WALLS: Tileset = Tileset::new(TILE_SIZE as u16, 16, WALL_IMAGES_BYTES);
pub static TILESET_SPRITES: Tileset = Tileset::new(TILE_SIZE as u16, 16, SPRITES_IMAGES_BYTES);

// =============================================================================
// PAC-MAN ANIMATIONS (2x2 tiles of 8x8 = 16x16 px)
// =============================================================================
define_anim!(pub ANIM_PAC_RIGHT, &TILESET_SPRITES, 2, 2, true, 2, [(4, 0), (2, 0), (0, 0), (2, 0)]);
define_anim!(pub ANIM_PAC_LEFT,  &TILESET_SPRITES, 2, 2, true, 2, [(4, 0), (2, 2), (0, 2), (2, 2)]);
define_anim!(pub ANIM_PAC_UP,    &TILESET_SPRITES, 2, 2, true, 2, [(4, 0), (2, 4), (0, 4), (2, 4)]);
define_anim!(pub ANIM_PAC_DOWN,  &TILESET_SPRITES, 2, 2, true, 2, [(4, 0), (2, 6), (0, 6), (2, 6)]);
define_anim!(pub ANIM_PAC_IDLE,  &TILESET_SPRITES, 2, 2, true, 1, [(4, 0)]);

define_anim!(
    pub ANIM_PAC_DEAD,
    &TILESET_SPRITES, 2, 2, true, 3,
    [
        (4, 0), (6, 0), (8, 0), (10, 0), (12, 0), (14, 0),
        (4, 2), (6, 2), (8, 2), (10, 2), (12, 2), (14, 2)
    ]
);

define_anim!(pub ANIM_LIFE_ICON, &TILESET_SPRITES, 2, 2, true, 1, [(2, 2)]);

// =============================================================================
// GHOST ANIMATIONS (2x2 tiles of 8x8 = 16x16 px)
// =============================================================================
define_anim!(pub ANIM_BLINKY_RIGHT, &TILESET_SPRITES, 2, 2, true, 8, [(0, 8), (2, 8)]);
define_anim!(pub ANIM_BLINKY_LEFT,  &TILESET_SPRITES, 2, 2, true, 8, [(4, 8), (6, 8)]);
define_anim!(pub ANIM_BLINKY_UP,    &TILESET_SPRITES, 2, 2, true, 8, [(8, 8), (10, 8)]);
define_anim!(pub ANIM_BLINKY_DOWN,  &TILESET_SPRITES, 2, 2, true, 8, [(12, 8), (14, 8)]);

define_anim!(pub ANIM_PINKY_RIGHT, &TILESET_SPRITES, 2, 2, true, 8, [(0, 10), (2, 10)]);
define_anim!(pub ANIM_PINKY_LEFT,  &TILESET_SPRITES, 2, 2, true, 8, [(4, 10), (6, 10)]);
define_anim!(pub ANIM_PINKY_UP,    &TILESET_SPRITES, 2, 2, true, 8, [(8, 10), (10, 10)]);
define_anim!(pub ANIM_PINKY_DOWN,  &TILESET_SPRITES, 2, 2, true, 8, [(12, 10), (14, 10)]);

define_anim!(pub ANIM_INKY_RIGHT, &TILESET_SPRITES, 2, 2, true, 8, [(0, 12), (2, 12)]);
define_anim!(pub ANIM_INKY_LEFT,  &TILESET_SPRITES, 2, 2, true, 8, [(4, 12), (6, 12)]);
define_anim!(pub ANIM_INKY_UP,    &TILESET_SPRITES, 2, 2, true, 8, [(8, 12), (10, 12)]);
define_anim!(pub ANIM_INKY_DOWN,  &TILESET_SPRITES, 2, 2, true, 8, [(12, 12), (14, 12)]);

define_anim!(pub ANIM_CLYDE_RIGHT, &TILESET_SPRITES, 2, 2, true, 8, [(0, 14), (2, 14)]);
define_anim!(pub ANIM_CLYDE_LEFT,  &TILESET_SPRITES, 2, 2, true, 8, [(4, 14), (6, 14)]);
define_anim!(pub ANIM_CLYDE_UP,    &TILESET_SPRITES, 2, 2, true, 8, [(8, 14), (10, 14)]);
define_anim!(pub ANIM_CLYDE_DOWN,  &TILESET_SPRITES, 2, 2, true, 8, [(12, 14), (14, 14)]);

define_anim!(pub ANIM_GHOST_FRIGHTENED, &TILESET_SPRITES, 2, 2, true, 8, [(4, 4), (6, 4)]);
define_anim!(pub ANIM_GHOST_FLASHING,   &TILESET_SPRITES, 2, 2, true, 4, [(4, 4), (6, 4), (8, 4), (10, 4)]);

define_anim!(pub ANIM_EYES_RIGHT, &TILESET_SPRITES, 2, 2, true, 1, [(4, 6)]);
define_anim!(pub ANIM_EYES_LEFT,  &TILESET_SPRITES, 2, 2, true, 1, [(6, 6)]);
define_anim!(pub ANIM_EYES_UP,    &TILESET_SPRITES, 2, 2, true, 1, [(8, 6)]);
define_anim!(pub ANIM_EYES_DOWN,  &TILESET_SPRITES, 2, 2, true, 1, [(10, 6)]);

define_anim!(pub ANIM_FRUIT_CHERRY,     &TILESET_SPRITES, 2, 2, true, 1, [(0, 16)]);
define_anim!(pub ANIM_FRUIT_STRAWBERRY, &TILESET_SPRITES, 2, 2, true, 1, [(2, 16)]);
define_anim!(pub ANIM_FRUIT_PEACH,      &TILESET_SPRITES, 2, 2, true, 1, [(4, 16)]);
define_anim!(pub ANIM_FRUIT_APPLE,      &TILESET_SPRITES, 2, 2, true, 1, [(6, 16)]);
define_anim!(pub ANIM_FRUIT_GRAPES,     &TILESET_SPRITES, 2, 2, true, 1, [(8, 16)]);
define_anim!(pub ANIM_FRUIT_GALAXIAN,   &TILESET_SPRITES, 2, 2, true, 1, [(10, 16)]);
define_anim!(pub ANIM_FRUIT_BELL,       &TILESET_SPRITES, 2, 2, true, 1, [(12, 16)]);
define_anim!(pub ANIM_FRUIT_KEY,        &TILESET_SPRITES, 2, 2, true, 1, [(14, 16)]);

pub fn fruit_anim(fruit_id: u8) -> &'static Animation<'static> {
    match fruit_id {
        0 => &ANIM_FRUIT_CHERRY,
        1 => &ANIM_FRUIT_STRAWBERRY,
        2 => &ANIM_FRUIT_PEACH,
        3 => &ANIM_FRUIT_APPLE,
        4 => &ANIM_FRUIT_GRAPES,
        5 => &ANIM_FRUIT_GALAXIAN,
        6 => &ANIM_FRUIT_BELL,
        _ => &ANIM_FRUIT_KEY,
    }
}

// =============================================================================
// HUD RENDERING (Margins outside the 224 px viewport)
// =============================================================================
const UI_X: u16 = 2;

pub fn draw_constant_ui(high_score: u32) {
    push_rect_uniform(
        Rect {
            x: 0,
            y: 0,
            width: VIEW_SCREEN_X,
            height: SCREEN_HEIGHT,
        },
        Color::BLACK,
    );
    push_rect_uniform(
        Rect {
            x: VIEW_SCREEN_X + 224,
            y: 0,
            width: VIEW_SCREEN_X,
            height: SCREEN_HEIGHT,
        },
        Color::BLACK,
    );

    draw_string(
        "SCORE\0",
        Point { x: UI_X, y: 8 },
        false,
        Color::WHITE,
        Color::BLACK,
    );

    // Written flush right (2px margin)
    let high_label_w = get_string_pixel_size("HIGH\0", false);
    draw_string(
        "HIGH\0",
        Point {
            x: SCREEN_WIDTH - UI_X - high_label_w,
            y: 8,
        },
        false,
        Color::WHITE,
        Color::BLACK,
    );

    let high_str = string_from_u32(high_score);
    draw_string(
        &high_str,
        Point {
            x: SCREEN_WIDTH - UI_X - get_string_pixel_size(&high_str, false),
            y: 24,
        },
        false,
        Color::WHITE,
        Color::BLACK,
    );

    draw_string(
        "LEVEL\0",
        Point { x: UI_X, y: 64 },
        false,
        Color::WHITE,
        Color::BLACK,
    );
    draw_string(
        "LIVES\0",
        Point {
            x: UI_X,
            y: SCREEN_HEIGHT - 44,
        },
        false,
        Color::WHITE,
        Color::BLACK,
    );
}

pub fn draw_score(score: u32) {
    push_rect_uniform(
        Rect {
            x: UI_X,
            y: 24,
            width: 44,
            height: SMALL_CHAR_HEIGHT,
        },
        Color::BLACK,
    );
    let s = string_from_u32(score);
    draw_string(
        &s,
        Point { x: UI_X, y: 24 },
        false,
        Color::WHITE,
        Color::BLACK,
    );
}

pub fn draw_level(level: u16) {
    push_rect_uniform(
        Rect {
            x: UI_X,
            y: 80,
            width: 44,
            height: SMALL_CHAR_HEIGHT,
        },
        Color::BLACK,
    );
    let l = string_from_u16(level);
    draw_string(
        &l,
        Point { x: UI_X, y: 80 },
        false,
        Color::WHITE,
        Color::BLACK,
    );
}

pub fn draw_lives(lives: u8) {
    let y = SCREEN_HEIGHT - 24;
    push_rect_uniform(
        Rect {
            x: 0,
            y,
            width: VIEW_SCREEN_X,
            height: 18,
        },
        Color::BLACK,
    );
    let mut x = UI_X;
    for _ in 0..lives.min(3) {
        ANIM_LIFE_ICON.draw_at(Point { x, y }, 0);
        x += 14;
    }
}

pub fn menu_vis_addon() {
    ANIM_PAC_RIGHT.draw_at(
        Point {
            x: CENTER.x - TILE_SIZE as u16,
            y: 65,
        },
        1,
    );
}
