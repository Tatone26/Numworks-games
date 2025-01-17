use core::str::FromStr;

use heapless::String;
use numworks_utils::{
    graphical::{draw_centered_string, draw_string_cfg, fill_screen, tiling::Tileset, ColorConfig},
    utils::{get_string_pixel_size, LARGE_CHAR_HEIGHT},
};

use crate::{
    eadk::{
        display::{push_rect_uniform, wait_for_vblank},
        Color, Point, Rect,
    },
    game_p4::{Alignment, MAX_HEIGHT_SIZE, MAX_PLAYERS, MAX_WIDTH_SIZE, PLAYERS_COLORS},
    utils::CENTER,
};

const COIN_SIZE: u16 = 21;
const PIXELS: usize = { COIN_SIZE * COIN_SIZE } as usize;
const LEFT_POS: u16 = CENTER.x - (COIN_SIZE + 4) / 2 - (COIN_SIZE + 4) * 3;
const UP_POS: u16 = 60;

/// This tileset contains 3 colors of coins, each with there victoy version.
static TILESET: Tileset = Tileset {
    tile_size: COIN_SIZE,
    width: 2 * COIN_SIZE,
    image: include_bytes!("./data/image.nppm"),
};

/// Draws a coin at the (x, y) GRID coordinates
pub fn draw_coin(x: u16, y: u16, color: u16, _c: &ColorConfig, vic: bool) {
    wait_for_vblank();
    TILESET.draw_tile::<PIXELS>(
        Point::new(
            LEFT_POS + (COIN_SIZE + 4) * x,
            UP_POS + (COIN_SIZE + 2) * (5 - y),
        ),
        Point::new(if vic { 1 } else { 0 }, color),
        1,
        true,
    );
}

pub fn clear_coin(x: u16, y: u16, c: &ColorConfig) {
    //wait_for_vblank();
    push_rect_uniform(
        Rect {
            x: LEFT_POS + (COIN_SIZE + 4) * x,
            y: UP_POS + (COIN_SIZE + 2) * (5 - y),
            width: COIN_SIZE,
            height: COIN_SIZE,
        },
        if c.bckgrd.rgb565 < 0x7BEF {
            Color::from_rgb888(50, 50, 50)
        } else {
            Color::from_rgb888(200, 200, 200)
        },
    )
}

/// Draws a full grid, taking into account the number of players.
pub fn draw_grid(players: u8, c: &ColorConfig) {
    wait_for_vblank();
    fill_screen(c.bckgrd);
    wait_for_vblank();
    push_rect_uniform(
        Rect {
            x: LEFT_POS - 4,
            y: UP_POS - 4,
            width: (COIN_SIZE + 4) * {
                MAX_WIDTH_SIZE as u16 - (MAX_PLAYERS as u16) + (players as u16)
            } + 4,
            height: (COIN_SIZE + 2) * 6 + 6,
        },
        c.text,
    );
    for x in 0..(MAX_WIDTH_SIZE as u16 - (MAX_PLAYERS as u16) + (players as u16)) {
        push_rect_uniform(
            Rect {
                x: LEFT_POS + (COIN_SIZE + 4) * x - 1,
                y: UP_POS - 4,
                width: COIN_SIZE + 2,
                height: (COIN_SIZE + 2) * 6 + 3,
            },
            c.bckgrd,
        );
        for y in 0..MAX_HEIGHT_SIZE as u16 {
            clear_coin(x, y, c);
        }
    }
}

/// Draws a coin above the grid (y_offset makes it lower, could technically be fixed in code for optimisation)
/// Make sure yourself that y_offset isn't going into negative values.
pub fn draw_selection_coin(x: u16, color: u16, _c: &ColorConfig, y_offset: i16) {
    wait_for_vblank();
    TILESET.draw_tile::<PIXELS>(
        Point::new(
            LEFT_POS + (COIN_SIZE + 4) * x,
            ((UP_POS - COIN_SIZE - 4) as i16 + y_offset) as u16,
        ),
        Point::new(0, color),
        1,
        true,
    );
}

/// Removes eveything above the grid, does NOT take into account y_offset for now -> BAD
pub fn clear_selection_coin(x: u16, c: &ColorConfig) {
    wait_for_vblank();
    push_rect_uniform(
        Rect {
            x: LEFT_POS + (COIN_SIZE + 4) * x,
            y: UP_POS - COIN_SIZE - 4,
            width: COIN_SIZE,
            height: COIN_SIZE,
        },
        c.bckgrd,
    )
}

/// Show the victory coins, and a victory text. Is ONLY visual.
pub fn victory(check: Alignment, c: &ColorConfig) {
    let color: Color = PLAYERS_COLORS[check.0 as usize];
    let x_range = check.1 .0..check.2 .0 + 1;
    let y_range = if check.1 .1 <= check.2 .1 {
        check.1 .1..check.2 .1 + 1
    } else {
        check.2 .1..check.1 .1 + 1
    };
    wait_for_vblank();
    draw_centered_string(
        "PUISSANCE 4 !\0",
        10,
        true,
        &ColorConfig {
            text: c.text,
            bckgrd: c.bckgrd,
            alt: color,
        },
        true,
    );
    if x_range.len() == 1 {
        for y in y_range {
            draw_coin(check.1 .0, y, check.0 as u16, c, true);
        }
    } else if y_range.len() == 1 {
        for x in x_range {
            draw_coin(x, check.1 .1, check.0 as u16, c, true);
        }
    } else if check.1 .1 <= check.2 .1 {
        for (x, y) in x_range.zip(y_range) {
            draw_coin(x, y, check.0 as u16, c, true);
        }
    } else {
        for (x, y) in x_range.zip(y_range.rev()) {
            draw_coin(x, y, check.0 as u16, c, true);
        }
    }
}

pub fn draw_thinking_ai(frame: u16, c: &ColorConfig) {
    wait_for_vblank();
    let mut s: String<13> = String::from_str("Thinking").unwrap();
    for _i in 0..(frame % 4) {
        s.push('.').unwrap();
    }
    for _i in (frame % 4)..3 {
        s.push(' ').unwrap();
    }
    s.push('\0').unwrap();
    draw_string_cfg(&s, Point { x: 3, y: 3 }, true, c, false);
}

pub fn clear_thinking_ai(c: &ColorConfig) {
    wait_for_vblank();
    push_rect_uniform(
        Rect {
            x: 3,
            y: 3,
            width: get_string_pixel_size("Thinking...\0", true),
            height: LARGE_CHAR_HEIGHT,
        },
        c.bckgrd,
    );
}
