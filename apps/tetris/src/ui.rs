use heapless::String;

use crate::{
    eadk::{
        display::{push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        Point, Rect,
    },
    game::{
        BACKGROUND_DARK_GRAY, BACKGROUND_GRAY, CASE_SIZE, COLOR_CONFIG, HIGH_SCORE,
        PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH,
    },
    utils::{draw_string_cfg, CENTER, LARGE_CHAR_HEIGHT}, tetriminos::Tetrimino,
};

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
pub fn draw_stable_ui() {
    push_rect_uniform(
        Rect {
            x: 0,
            y: 0,
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
        },
        BACKGROUND_GRAY,
    );
    draw_ui_base("  BEST\0", Point::new(24, 2), 6, 8);
    draw_string_cfg(
        HIGH_SCORE,
        Point::new(CASE_SIZE * 24, CASE_SIZE * 2 + LARGE_CHAR_HEIGHT),
        true,
        &COLOR_CONFIG,
        false,
    );
    draw_string_cfg(
        " SCORE\0",
        Point::new(CASE_SIZE * 24, CASE_SIZE * 3 + LARGE_CHAR_HEIGHT * 2),
        true,
        &COLOR_CONFIG,
        false,
    );
    draw_score(0);
    push_rect_uniform(
        Rect {
            x: CENTER.x - (PLAYFIELD_WIDTH / 2 + 1) * CASE_SIZE,
            y: CASE_SIZE * 1,
            width: CASE_SIZE * (PLAYFIELD_WIDTH + 2),
            height: CASE_SIZE * (PLAYFIELD_HEIGHT + 2),
        },
        BACKGROUND_DARK_GRAY,
    );
    push_rect_uniform(
        Rect {
            x: CENTER.x - (PLAYFIELD_WIDTH / 2) * CASE_SIZE,
            y: CASE_SIZE * 2,
            width: CASE_SIZE * PLAYFIELD_WIDTH,
            height: CASE_SIZE * PLAYFIELD_HEIGHT,
        },
        COLOR_CONFIG.bckgrd,
    );
    draw_ui_base(" LEVEL\0", Point::new(24, 14), 6, 8);
    draw_level(1);
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
    draw_line(0);
    draw_ui_base("NEXT\0", Point::new(2, 2), 6, 8);
    draw_ui_base("HOLD\0", Point::new(2, 14), 6, 8);
}

pub fn draw_score(score: u32) {
    let mut score_txt: String<7> = String::<7>::new();
    if score < 999_999 {
        let score_str: String<6> = String::<6>::from(score);
        for _ in 0..(6 - score_str.chars().count()) {
            score_txt.push('0').unwrap();
        }
        score_txt.push_str(score_str.as_str()).unwrap();
        score_txt.push('\0').unwrap();
    } else {
        score_txt.push_str("999999\0").unwrap();
    }
    draw_string_cfg(
        score_txt.as_str(),
        Point::new(CASE_SIZE * 24, CASE_SIZE * 3 + LARGE_CHAR_HEIGHT * 3),
        true,
        &COLOR_CONFIG,
        false,
    );
}

pub fn draw_level(level: u16) {
    let mut level_txt: String<7> = String::<7>::new();
    let level_str: String<6> = String::<6>::from(level);
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

pub fn draw_line(line: u16) {
    let mut line_txt: String<7> = String::<7>::new();
    let line_str: String<6> = String::<6>::from(line);
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
    // TODO : Needs to not try to draw when negative position !!!
    for p in tetri.get_blocks() {
        if ((tetri.pos.x + p.0) >= 0) & ((tetri.pos.y + p.1) >= 0) {
            push_rect_uniform(
                Rect {
                    x: CENTER.x - (PLAYFIELD_WIDTH / 2) * CASE_SIZE
                        + ((tetri.pos.x + p.0) as u16) * CASE_SIZE,
                    y: CASE_SIZE * 2 + ((tetri.pos.y + p.1) as u16) * CASE_SIZE,
                    width: CASE_SIZE,
                    height: CASE_SIZE,
                },
                if clear {
                    COLOR_CONFIG.bckgrd
                } else {
                    tetri.color
                },
            )
        }
    }
}
