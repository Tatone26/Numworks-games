use crate::{
    eadk::{
        display::{push_rect_uniform, wait_for_vblank},
        Color, Point, Rect,
    },
    game_p4::{MAX_HEIGHT_SIZE, MAX_PLAYERS, MAX_WIDTH_SIZE, PLAYERS_COLORS},
    utils::{draw_centered_string, draw_tile, fill_screen, ColorConfig, Tileset, CENTER},
};

const COIN_SIZE: u16 = 21;
const PIXELS: usize = { COIN_SIZE * COIN_SIZE } as usize;
const LEFT_POS: u16 = CENTER.x - (COIN_SIZE + 4) / 2 - (COIN_SIZE + 4) * 3;
const UP_POS: u16 = 60;

static TILESET: Tileset = Tileset {
    tile_size: COIN_SIZE,
    image: include_bytes!("./data/coins.ppm"),
};

pub fn draw_coin(x: u16, y: u16, color: u16, _c: &ColorConfig, vic: bool) {
    wait_for_vblank();
    draw_tile::<PIXELS>(
        &TILESET,
        Point::new(
            LEFT_POS + (COIN_SIZE + 4) * x,
            UP_POS + (COIN_SIZE + 2) * (5 - y),
        ),
        Point::new(if vic { 1 } else { 0 }, color),
        1,
        true,
    );
}

pub fn draw_grid(players: u8, c: &ColorConfig) {
    wait_for_vblank();
    fill_screen(c.bckgrd);
    push_rect_uniform(
        Rect {
            x: LEFT_POS - 4,
            y: UP_POS - 4,
            width: (COIN_SIZE + 4) * { MAX_WIDTH_SIZE as u16 - (MAX_PLAYERS - players) as u16 } + 4,
            height: (COIN_SIZE + 2) * 6 + 6,
        },
        c.text,
    );
    for x in 0..(MAX_WIDTH_SIZE as u16 - (MAX_PLAYERS - players) as u16) {
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
pub fn draw_selection_coin(x: u16, color: u16, _c: &ColorConfig, y_offset: i16) {
    wait_for_vblank();
    draw_tile::<PIXELS>(
        &TILESET,
        Point::new(
            LEFT_POS + (COIN_SIZE + 4) * x,
            ((UP_POS - COIN_SIZE - 4) as i16 + y_offset) as u16,
        ),
        Point::new(0, color),
        1,
        true,
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
    let x_range = pos1.0..pos2.0 + 1;
    let y_range = if pos1.1 <= pos2.1 {
        pos1.1..pos2.1 + 1
    } else {
        pos2.1..pos1.1 + 1
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
