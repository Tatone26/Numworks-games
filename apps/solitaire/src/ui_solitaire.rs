use heapless::Vec;
use numworks_utils::{
    eadk::{
        display::{self, draw_string, push_rect_uniform, SCREEN_WIDTH},
        timing::msleep,
        Color, Point, Rect,
    },
    utils::{
        draw_tile, fill_screen, wait_for_no_keydown, Tileset, LARGE_CHAR_HEIGHT, LARGE_CHAR_WIDTH,
    },
};

use crate::game_solitaire::{Card, CursorPos, Table};

/// Images work really well with square tiles. You can still draw other images, but it is less good.
static TILESET: Tileset = Tileset {
    tile_size: 35,
    image: include_bytes!("./data/cartes.ppm"),
};
const PIXELS: usize = { 35 * 35 } as usize;

const NAMES_LIST: [&'static str; 13] = [
    "A\0", "2\0", "3\0", "4\0", "5\0", "6\0", "7\0", "8\0", "9\0", "10\0", "J\0", "Q\0", "K\0",
];

const BACKGROUND_COLOR: Color = Color::from_rgb888(1, 115, 55);
const CARD_HEIGHT: u16 = 56;
const CARD_WIDTH: u16 = 35;
const HIDDEN_CARD_TILE: u16 = 4;

fn draw_card(card: &Card, pos: Point) {
    if card.visible {
        draw_tile::<PIXELS>(&TILESET, pos, Point::new(card.suit as u16, 0), 1, false);
        draw_tile::<PIXELS>(
            &TILESET,
            Point::new(pos.x, pos.y + TILESET.tile_size),
            Point::new(card.suit as u16, 1),
            1,
            true,
        );
        draw_string(
            NAMES_LIST[card.number as usize],
            Point::new(pos.x + 3, pos.y + 2),
            true,
            Color::BLACK,
            Color::WHITE,
        );
        draw_string(
            NAMES_LIST[card.number as usize],
            Point::new(
                pos.x + CARD_WIDTH - 4 - {
                    if card.number == 9 {
                        2 * LARGE_CHAR_WIDTH
                    } else {
                        LARGE_CHAR_WIDTH
                    }
                },
                pos.y + CARD_HEIGHT - 4 - LARGE_CHAR_HEIGHT,
            ),
            true,
            Color::BLACK,
            Color::WHITE,
        );
    } else {
        draw_tile::<PIXELS>(&TILESET, pos, Point::new(HIDDEN_CARD_TILE, 0), 1, false);
        draw_tile::<PIXELS>(
            &TILESET,
            Point::new(pos.x, pos.y + TILESET.tile_size),
            Point::new(HIDDEN_CARD_TILE, 1),
            1,
            true,
        );
    }
}

fn get_pos_from_cursor_pos(cursor_pos: CursorPos, table: &Table) -> Point {
    match cursor_pos {
        CursorPos::Tableau(i) => {
            return Point::new(20 + (i as u16) * (CARD_WIDTH + 6), CARD_HEIGHT
            + 25
            + (i as u16) * {
                if table.tableau[i as usize].len() <= 7 {
                    LARGE_CHAR_HEIGHT
                } else {
                    LARGE_CHAR_HEIGHT / 2
                }
            })
        }
        CursorPos::Fondations(i) => return Point::new(20 + (i as u16) * (CARD_WIDTH + 6), 10),
        CursorPos::Stock(i) => {
            if i == 1 {
                return Point::new(SCREEN_WIDTH - CARD_WIDTH - 21, 10);
            } else {
                let mut o = 0;
                if table.stock_iter == 2 {
                    o = 1;
                } else if table.stock_iter >= 2 {
                    o = 2;
                }
                return Point::new(SCREEN_WIDTH - 3 * CARD_WIDTH - 21 + o * CARD_WIDTH / 2, 10);
            }
        }
    };
}

pub fn draw_selection(cursor_pos: CursorPos, selected: bool, table: &Table) {
    let pos = get_pos_from_cursor_pos(cursor_pos, table);
    display::wait_for_vblank();
    let empty: bool = {
        match cursor_pos {
            CursorPos::Tableau(b) => table.tableau[b as usize].is_empty(),
            CursorPos::Fondations(b) => table.fondations[b as usize].is_empty(),
            CursorPos::Stock(b) => {
                (b == 0 && table.stock_iter <= 0)
                    || (b == 1 && table.stock_iter >= table.stock.len())
            }
        }
    };
    if !selected && empty {
        draw_empty_place(pos);
    } else {
        let tile: u16 = {
            if selected {
                3
            } else {
                2
            }
        };
        draw_tile::<PIXELS>(&TILESET, pos, Point::new(tile, 2), 1, true);
        draw_tile::<PIXELS>(
            &TILESET,
            Point::new(pos.x, pos.y + TILESET.tile_size),
            Point::new(tile, 3),
            1,
            true,
        );
    }
}

fn draw_empty_place(pos: Point) {
    draw_tile::<PIXELS>(&TILESET, pos, Point::new(0, 2), 1, true);
    draw_tile::<PIXELS>(
        &TILESET,
        Point::new(pos.x, pos.y + TILESET.tile_size),
        Point::new(0, 3),
        1,
        true,
    );
}

fn draw_empty_table() {
    fill_screen(BACKGROUND_COLOR);
    let table = Table::empty();
    for i in 0..4 {
        draw_selection(CursorPos::Fondations(i), false, &table);
    }
    for i in 0..2 {
        draw_selection(CursorPos::Stock(i), false, &table);
    }
    for i in 0..7 {
        draw_selection(CursorPos::Tableau(i), false, &table);
    }
}

fn draw_tableau_stack(stack: &Vec<Card, 52>, number: u16) {
    if !stack.is_empty() {
        for (i, c) in stack.iter().enumerate() {
            draw_card(
                c,
                Point::new(
                    20 + number * (CARD_WIDTH + 6),
                    CARD_HEIGHT
                        + 25
                        + (i as u16) * {
                            if stack.len() <= 7 {
                                LARGE_CHAR_HEIGHT
                            } else {
                                LARGE_CHAR_HEIGHT / 2
                            }
                        },
                ),
            );
        }
    } else {
        draw_empty_place(Point::new(20 + number * (CARD_WIDTH + 6), CARD_HEIGHT + 25));
    }
}

fn draw_fondations_stack(stack: &Vec<Card, 13>, number: u16) {
    if !stack.is_empty() {
        draw_card(
            stack.last().unwrap(),
            Point::new(20 + number * (CARD_WIDTH + 6), 10),
        );
    } else {
        draw_empty_place(Point::new(20 + number * (CARD_WIDTH + 6), 10));
    }
}

pub fn draw_stock(stack: &Vec<Card, 52>, stock_iter: usize) {
    display::wait_for_vblank();
    if !stack.is_empty() {
        if stock_iter >= stack.len() {
            draw_empty_place(Point::new(SCREEN_WIDTH - CARD_WIDTH - 21, 10));
        } else {
            draw_card(
                stack.get(stock_iter).unwrap(),
                Point::new(SCREEN_WIDTH - CARD_WIDTH - 21, 10),
            );
        }
        if stock_iter > 0 {
            let start: usize = {
                if (stock_iter as i16 - 3) > 0 {
                    stock_iter - 3
                } else {
                    0
                }
            };
            let mut o: u16 = 0;
            display::wait_for_vblank();
            for i in start..stock_iter {
                let mut card: Card = stack.get(i).unwrap().clone();
                card.visible = true;
                draw_card(
                    &card,
                    Point::new(SCREEN_WIDTH - 3 * CARD_WIDTH - 21 + o * CARD_WIDTH / 2, 10),
                );
                o += 1;
            }
        } else {
            draw_empty_place(Point::new(SCREEN_WIDTH - 3 * CARD_WIDTH - 21, 10));
        }
    } else {
        draw_empty_place(Point::new(SCREEN_WIDTH - CARD_WIDTH - 21, 10));
        draw_empty_place(Point::new(SCREEN_WIDTH - 3 * CARD_WIDTH - 21, 10))
    }
}

fn draw_table(table: &Table) {
    for i in 0..7 {
        draw_tableau_stack(&table.tableau[i], i as u16);
    }
    for i in 0..4 {
        draw_fondations_stack(&table.fondations[i], i as u16);
    }
    draw_stock(&table.stock, table.stock_iter);
}

pub fn ui_test(table: &Table) {
    wait_for_no_keydown();
    draw_empty_table();
    draw_table(table);
}
