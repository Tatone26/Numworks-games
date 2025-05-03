use heapless::String;
use numworks_utils::{
    eadk::{
        display::{push_rect_uniform, wait_for_vblank, SCREEN_HEIGHT, SCREEN_WIDTH},
        Color, Point, Rect,
    },
    graphical::{draw_string_cfg, tiling::Tileset},
    include_bytes_align_as,
    utils::{string_from_u32, CENTER, LARGE_CHAR_HEIGHT},
};

use crate::{
    game_tetris::{
        BACKGROUND_DARK_GRAY, BACKGROUND_GRAY, CASE_SIZE, COLOR_CONFIG, PLAYFIELD_HEIGHT,
        PLAYFIELD_WIDTH,
    },
    tetriminos::Tetrimino,
};

const IMAGE_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/image.nppm");

static TILESET: Tileset = Tileset::new(CASE_SIZE, 10, IMAGE_BYTES);

/// Draws a box of the given size, at the given pos on the grid, with a given title (first-line text), following the ui style
fn draw_ui_base(title: &'static str, pos: Point, w: u16, h: u16) {
    // pos, width and height of the INNER rectangle.
    push_rect_uniform(
        Rect {
            x: CASE_SIZE * (pos.x - 1),
            y: CASE_SIZE * (pos.y - 1),
            width: (w + 2) * CASE_SIZE,
            height: (h + 2) * CASE_SIZE,
        },
        BACKGROUND_DARK_GRAY,
    );
    push_rect_uniform(
        Rect {
            x: CASE_SIZE * pos.x,
            y: CASE_SIZE * pos.y,
            width: w * CASE_SIZE,
            height: h * CASE_SIZE,
        },
        COLOR_CONFIG.bckgrd,
    );
    draw_string_cfg(
        title,
        Point::new(CASE_SIZE * pos.x, CASE_SIZE * pos.y),
        true,
        &COLOR_CONFIG,
        false,
    );
}

/// This draws every UI elements that will never change (rects, titles...)
/// Needs to be upgraded so it doesn't take almost TWO HUNDREDS lines.
/// Upgrade : function that draws one UI rect given position, size and title.
pub fn draw_stable_ui(level: u16, level_lines: u16, score: u32, high_score: u32) {
    let start_x = CENTER.x - (PLAYFIELD_WIDTH / 2) * CASE_SIZE;
    let start_y = CASE_SIZE * 2;
    wait_for_vblank();
    push_rect_uniform(
        Rect {
            x: 0,
            y: 0,
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
        },
        BACKGROUND_GRAY,
    );
    push_rect_uniform(
        Rect {
            x: CENTER.x - (PLAYFIELD_WIDTH / 2 + 1) * CASE_SIZE,
            y: CASE_SIZE,
            width: CASE_SIZE * (PLAYFIELD_WIDTH + 2),
            height: CASE_SIZE * (PLAYFIELD_HEIGHT + 2),
        },
        BACKGROUND_DARK_GRAY,
    );
    TILESET.tiling(
        Point::new(start_x, start_y),
        (PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT),
        Point::new(7, 0),
        false,
        1,
    );

    wait_for_vblank();

    draw_ui_base("  BEST\0", Point::new(24, 2), 6, 8);
    draw_ui_base(" LEVEL\0", Point::new(24, 14), 6, 8);
    draw_ui_base("NEXT\0", Point::new(2, 2), 6, 8);
    draw_ui_base("HOLD\0", Point::new(2, 14), 6, 8);

    wait_for_vblank();
    draw_score(high_score, true);
    draw_string_cfg(
        " SCORE\0",
        Point::new(CASE_SIZE * 24, CASE_SIZE * 3 + LARGE_CHAR_HEIGHT * 2),
        true,
        &COLOR_CONFIG,
        false,
    );
    draw_score(score, false);
    draw_level(level);
    draw_string_cfg(
        "  LINE\0",
        Point::new(
            SCREEN_WIDTH - CASE_SIZE * 8,
            CASE_SIZE * 15 + LARGE_CHAR_HEIGHT * 2,
        ),
        true,
        &COLOR_CONFIG,
        false,
    );
    draw_lines_number(level_lines);
}

pub fn draw_score(score: u32, high_score: bool) {
    let mut score_txt: String<7> = String::<7>::new();
    if score < 999_999 {
        let mut score_str: String<11> = string_from_u32(score);
        score_str.pop();
        for _ in 0..(6 - score_str.chars().count()) {
            score_txt.push('0').unwrap();
        }
        score_txt.push_str(score_str.as_str()).unwrap();
        score_txt.push('\0').unwrap();
    } else {
        score_txt.push_str("999999\0").unwrap();
    }
    if !high_score {
        draw_string_cfg(
            score_txt.as_str(),
            Point::new(CASE_SIZE * 24, CASE_SIZE * 3 + LARGE_CHAR_HEIGHT * 3),
            true,
            &COLOR_CONFIG,
            false,
        );
    } else {
        draw_string_cfg(
            score_txt.as_str(),
            Point::new(CASE_SIZE * 24, CASE_SIZE * 3 + LARGE_CHAR_HEIGHT / 2),
            true,
            &COLOR_CONFIG,
            false,
        );
    }
}

pub fn draw_level(level: u16) {
    let mut level_txt: String<7> = String::<7>::new();
    let level_str: String<11> = string_from_u32(level as u32);
    for _ in 0..(6 - level_str.chars().count()) {
        level_txt.push(' ').unwrap();
    }
    level_txt.push_str(level_str.as_str()).unwrap();
    level_txt.push('\0').unwrap();
    draw_string_cfg(
        level_txt.as_str(),
        Point::new(CASE_SIZE * 24, CASE_SIZE * 14 + LARGE_CHAR_HEIGHT),
        true,
        &COLOR_CONFIG,
        false,
    );
}

pub fn draw_lines_number(line: u16) {
    let mut line_txt: String<7> = String::<7>::new();
    let line_str: String<11> = string_from_u32(line as u32);
    for _ in 0..(6 - line_str.chars().count()) {
        line_txt.push(' ').unwrap();
    }
    line_txt.push_str(line_str.as_str()).unwrap();
    line_txt.push('\0').unwrap();
    draw_string_cfg(
        line_txt.as_str(),
        Point::new(CASE_SIZE * 24, CASE_SIZE * 15 + LARGE_CHAR_HEIGHT * 3),
        true,
        &COLOR_CONFIG,
        false,
    );
}

/// Draws a given tetrimino.
pub fn draw_tetrimino(tetri: &Tetrimino, clear: bool) {
    for pos in tetri.get_blocks_grid_pos() {
        if (pos.x >= 0) & (pos.y >= 0) {
            draw_block_image(
                CENTER.x - (PLAYFIELD_WIDTH / 2) * CASE_SIZE + (pos.x as u16) * CASE_SIZE,
                CASE_SIZE * 2 + (pos.y as u16) * CASE_SIZE,
                if clear { 7 } else { tetri.color as u16 },
            );
        }
    }
}

pub fn draw_next_tetrimino(tetri: &Tetrimino) {
    push_rect_uniform(
        Rect {
            x: CASE_SIZE * 2,
            y: CASE_SIZE * 4,
            width: 6 * CASE_SIZE,
            height: 4 * CASE_SIZE,
        },
        COLOR_CONFIG.bckgrd,
    );
    for (x, y) in tetri.get_blocks() {
        draw_block_image(
            (5 + x) as u16 * CASE_SIZE,
            (6 + y) as u16 * CASE_SIZE,
            tetri.color as u16,
        );
    }
}

pub fn draw_held_tetrimino(tetri: &Tetrimino) {
    push_rect_uniform(
        Rect {
            x: CASE_SIZE * 2,
            y: CASE_SIZE * 16,
            width: 6 * CASE_SIZE,
            height: 6 * CASE_SIZE,
        },
        COLOR_CONFIG.bckgrd,
    );
    for (x, y) in tetri.get_blocks() {
        draw_block_image(
            (5 + x) as u16 * CASE_SIZE,
            (18 + y) as u16 * CASE_SIZE,
            tetri.color as u16,
        );
    }
}

pub fn draw_blank_line(y: u16) {
    let start_x = CENTER.x - (PLAYFIELD_WIDTH / 2) * CASE_SIZE;
    TILESET.tiling(
        Point::new(start_x, y * CASE_SIZE + 2 * CASE_SIZE),
        (PLAYFIELD_WIDTH, 1),
        Point::new(7, 0),
        false,
        1,
    );
}

fn draw_block_image(abs_x: u16, abs_y: u16, x_map: u16) {
    TILESET.draw_tile(Point::new(abs_x, abs_y), Point::new(x_map, 0), 1, false);
}

pub fn draw_block(x: u16, y: u16, color: u16) {
    draw_block_image(
        CENTER.x - (PLAYFIELD_WIDTH / 2) * CASE_SIZE + x * CASE_SIZE,
        CASE_SIZE * 2 + y * CASE_SIZE,
        color,
    );
}

// pub fn debug_check() {
//     push_rect_uniform(
//         Rect {
//             x: 0,
//             y: 0,
//             width: 10,
//             height: 10,
//         },
//         Color::GREEN,
//     );
//     timing::msleep(200);
//     push_rect_uniform(
//         Rect {
//             x: 0,
//             y: 0,
//             width: 10,
//             height: 10,
//         },
//         Color::BLACK,
//     );
// }
