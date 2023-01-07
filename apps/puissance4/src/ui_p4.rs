use crate::{
    eadk::{display::push_rect_uniform, Color, Rect},
    game_p4::{MAX_HEIGHT_SIZE, MAX_WIDTH_SIZE, PLAYERS_COLORS},
    utils::{draw_centered_string, draw_image_from_tilemap, fill_screen, ColorConfig, CENTER},
};

const COIN_SIZE: u16 = 21;
const LEFT_POS: u16 = CENTER.x - (COIN_SIZE+4) / 2 - (COIN_SIZE + 4)*3;
const UP_POS: u16 = 60;

static COINS_TILE_MAP: [u8; 19859] = *include_bytes!("./coins.ppm");

pub fn draw_coin(x: u16, y: u16, color: u16, c: &ColorConfig, vic: bool) {
    draw_image_from_tilemap(
        &COINS_TILE_MAP,
        LEFT_POS + (COIN_SIZE + 4) * x,
        UP_POS + (COIN_SIZE + 2) * (5 - y),
        21,
        21,
        1,
        if vic {
            COIN_SIZE * 4
        } else if c.bckgrd.rgb565 > 0x7BEF {
            COIN_SIZE * 3
        } else {
            COIN_SIZE * 2
        },
        color * COIN_SIZE,
    );
}

pub fn draw_grid(three_players: bool, c: &ColorConfig) {
    fill_screen(c.bckgrd);
    push_rect_uniform(
        Rect {
            x: LEFT_POS - 4,
            y: UP_POS - 4,
            width: (COIN_SIZE + 4) * {
                if !three_players {
                    7
                } else {
                    MAX_WIDTH_SIZE as u16
                }
            } + 4,
            height: (COIN_SIZE + 2) * 6 + 6,
        },
        c.text,
    );
    for x in 0..(if !three_players {
        7
    } else {
        MAX_WIDTH_SIZE as u16
    }) {
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
    }
}

// Make sure yourself that y_offset isn't going into negative values.
pub fn draw_selection_coin(x: u16, color: u16, c: &ColorConfig, y_offset: i16) {
    draw_image_from_tilemap(
        &COINS_TILE_MAP,
        LEFT_POS + (COIN_SIZE + 4) * x,
        ((UP_POS - COIN_SIZE - 4) as i16 + y_offset) as u16,
        21,
        21,
        1,
        if c.bckgrd.rgb565 > 0x7BEF {
            0
        } else {
            COIN_SIZE
        },
        color * COIN_SIZE,
    );
}

pub fn clear_selection_coin(x: u16, c: &ColorConfig) {
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

pub fn victory(check: Option<(u8, (u16, u16), (u16, u16))>, c: &ColorConfig) {
    let victor;
    let pos1; // TODO
    let pos2;
    let color: Color;
    (victor, pos1, pos2) = check.unwrap();
    color = PLAYERS_COLORS[(victor - 1) as usize];
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
    let x_range = pos1.0..pos2.0 + 1;
    let y_range = if pos1.1 <= pos2.1 {
        pos1.1..pos2.1 + 1
    } else {
        pos2.1..pos1.1 + 1
    };
    if x_range.len() == 1 {
        for y in y_range {
            draw_coin(pos1.0, y, victor as u16 - 1, c, true);
        }
    } else if y_range.len() == 1 {
        for x in x_range {
            draw_coin(x, pos1.1, victor as u16 - 1, c, true);
        }
    } else {
        if pos1.1 <= pos2.1 {
            for (x, y) in x_range.zip(y_range) {
                draw_coin(x, y, victor as u16 - 1, c, true);
            }
        } else {
            for (x, y) in x_range.zip(y_range.rev()) {
                draw_coin(x, y, victor as u16 - 1, c, true);
            }
        }
    }
}
