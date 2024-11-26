use heapless::Vec;
use numworks_utils::{
    eadk::{
        display::{self, draw_string, push_rect_uniform, SCREEN_WIDTH},
        Color, Point, Rect,
    },
    graphical::{fill_screen, tiling::Tileset},
    include_bytes_align_as,
    utils::{randint, LARGE_CHAR_HEIGHT, LARGE_CHAR_WIDTH},
};

use crate::{
    cards::{Card, Suit, NAMES_LIST},
    game_solitaire::CursorPos,
    table::{Stack, Table},
};

/// Images work really well with square tiles. You can still draw other images, but it is less good.
static TILESET: Tileset = Tileset {
    tile_size: 35,
    width: 35 * 5,
    image: include_bytes_align_as!(Color, "./data/image.nppm"),
};
const PIXELS: usize = { 35 * 35 } as usize;

pub const BACKGROUND_COLOR: Color = Color::from_rgb888(1, 115, 55);
const CARD_HEIGHT: u16 = 56;
const CARD_WIDTH: u16 = 35;
const HIDDEN_CARD_TILE: u16 = 4;

/// Draws a given [Card] at a given [Point], point being in absolute pixel value
fn draw_card(card: &Card, pos: Point) {
    if card.visible {
        TILESET.draw_tile::<PIXELS>(pos, Point::new(card.suit as u16, 0), 1, false);
        TILESET.draw_tile::<PIXELS>(
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
        TILESET.draw_tile::<PIXELS>(pos, Point::new(HIDDEN_CARD_TILE, 0), 1, false);
        TILESET.draw_tile::<PIXELS>(
            Point::new(pos.x, pos.y + TILESET.tile_size),
            Point::new(HIDDEN_CARD_TILE, 1),
            1,
            true,
        );
    }
}

/// Returns the card absolute position from a given [CursorPos]
fn get_pos_from_cursor_pos(cursor_pos: &CursorPos, table: &Table) -> Point {
    match cursor_pos {
        CursorPos::Tableau(i) => {
            let number_of_cards = table.tableau[*i as usize].len() as u16;
            Point::new(
                20 + (*i as u16) * (CARD_WIDTH + 6),
                CARD_HEIGHT
                    + 25
                    + (if number_of_cards > 0 {
                        number_of_cards - 1
                    } else {
                        0
                    }) * {
                        if table.tableau[*i as usize].len() <= 7 {
                            LARGE_CHAR_HEIGHT
                        } else {
                            LARGE_CHAR_HEIGHT / 2
                        }
                    },
            )
        }
        CursorPos::Fondations(i) => Point::new(20 + (*i as u16) * (CARD_WIDTH + 6), 10),
        CursorPos::Stock(i) => {
            if *i == 1 {
                Point::new(SCREEN_WIDTH - CARD_WIDTH - 19, 10)
            } else {
                let mut o = 0;
                if table.stock_iter == 2 {
                    o = 1;
                } else if table.stock_iter >= 2 {
                    o = 2;
                }
                Point::new(SCREEN_WIDTH - 3 * CARD_WIDTH - 23 + o * CARD_WIDTH / 2, 10)
            }
        }
    }
}

/// Draw stuff given by the parameters : [CursorPos] for the position, clear to know if it needs to remove everything and selected to know if it is selected.
pub fn draw_selection(
    cursor_pos: &CursorPos,
    clear: bool,
    selected: bool,
    table: &Table,
    selection_size: u8,
) {
    let pos = get_pos_from_cursor_pos(cursor_pos, table);
    display::wait_for_vblank();
    let empty: bool = {
        match cursor_pos {
            CursorPos::Tableau(b) => table.tableau[*b as usize].is_empty(),
            CursorPos::Fondations(b) => table.fondations[*b as usize].is_empty(),
            CursorPos::Stock(b) => {
                (*b == 0 && table.stock_iter == 0)
                    || (*b == 1 && table.stock_iter >= table.stock.len())
            }
        }
    };
    if clear && empty {
        draw_empty_place(pos);
    } else {
        let tile: u16 = {
            if selected {
                4
            } else if clear {
                2
            } else {
                3
            }
        };
        TILESET.draw_tile::<PIXELS>(
            Point::new(pos.x, pos.y + TILESET.tile_size),
            Point::new(tile, 3),
            1,
            true,
        );
        match cursor_pos {
            CursorPos::Tableau(b) => {
                let stack = &table.tableau[*b as usize];
                for i in 0..selection_size {
                    TILESET.draw_tile::<PIXELS>(
                        Point::new(
                            pos.x,
                            pos.y
                                - (i as u16) * {
                                    if stack.len() <= 7 {
                                        LARGE_CHAR_HEIGHT
                                    } else {
                                        LARGE_CHAR_HEIGHT / 2
                                    }
                                },
                        ),
                        Point::new(tile, 2),
                        1,
                        true,
                    );
                }
            }
            _ => TILESET.draw_tile::<PIXELS>(pos, Point::new(tile, 2), 1, true),
        }
    }
}

/// Draws an empty place, which is just an outline.
fn draw_empty_place(pos: Point) {
    TILESET.draw_tile::<PIXELS>(pos, Point::new(0, 2), 1, true);
    TILESET.draw_tile::<PIXELS>(
        Point::new(pos.x, pos.y + TILESET.tile_size),
        Point::new(0, 3),
        1,
        true,
    );
}

/// Draws a given stack on the "tableau", at the given position.
pub fn draw_tableau_stack(stack: &Stack, stack_number: u16) {
    display::wait_for_vblank();
    push_rect_uniform(
        Rect {
            x: 20 + stack_number * (CARD_WIDTH + 6),
            y: CARD_HEIGHT + 25,
            height: 200,
            width: CARD_WIDTH,
        },
        BACKGROUND_COLOR,
    );
    if !stack.is_empty() {
        for (i, c) in stack.iter().enumerate() {
            draw_card(
                c,
                Point::new(
                    20 + stack_number * (CARD_WIDTH + 6),
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
        draw_empty_place(Point::new(
            20 + stack_number * (CARD_WIDTH + 6),
            CARD_HEIGHT + 25,
        ));
    }
}

/// Draws the stack at the top (the foundations), given a stack and its position.
pub fn draw_foundations_stack(stack: &Vec<Card, 13>, number: u16) {
    if !stack.is_empty() {
        draw_card(
            stack.last().unwrap(),
            Point::new(20 + number * (CARD_WIDTH + 6), 10),
        );
    } else {
        draw_empty_place(Point::new(20 + number * (CARD_WIDTH + 6), 10));
    }
}

/// Draws the stock, which is a stack with some cards shown and some not shown.
pub fn draw_stock(stack: &Stack, stock_iter: usize) {
    display::wait_for_vblank();
    if !stack.is_empty() {
        if stock_iter >= stack.len() {
            draw_empty_place(Point::new(SCREEN_WIDTH - CARD_WIDTH - 19, 10));
        } else {
            draw_card(
                stack.get(stock_iter).unwrap(),
                Point::new(SCREEN_WIDTH - CARD_WIDTH - 19, 10),
            );
        }
        push_rect_uniform(
            Rect {
                x: SCREEN_WIDTH - 3 * CARD_WIDTH - 23,
                y: 10,
                height: CARD_HEIGHT,
                width: CARD_WIDTH * 2,
            },
            BACKGROUND_COLOR,
        );
        if stock_iter > 0 {
            let start: usize = { stock_iter.saturating_sub(3) };
            for (o, i) in (0..).zip(start..stock_iter) {
                let mut card: Card = *stack.get(i).unwrap();
                card.visible = true;
                display::wait_for_vblank();
                draw_card(
                    &card,
                    Point::new(SCREEN_WIDTH - 3 * CARD_WIDTH - 23 + o * CARD_WIDTH / 2, 10),
                );
            }
        } else {
            draw_empty_place(Point::new(SCREEN_WIDTH - 3 * CARD_WIDTH - 23, 10));
        }
    } else {
        draw_empty_place(Point::new(SCREEN_WIDTH - CARD_WIDTH - 19, 10));
        draw_empty_place(Point::new(SCREEN_WIDTH - 3 * CARD_WIDTH - 23, 10))
    }
}

/// Draws the entire table.
pub fn draw_table(table: &Table) {
    fill_screen(BACKGROUND_COLOR);
    for i in 0..7 {
        draw_tableau_stack(&table.tableau[i], i as u16);
    }
    for i in 0..4 {
        draw_foundations_stack(&table.fondations[i], i as u16);
    }
    draw_stock(&table.stock, table.stock_iter);
}

/// The visual addon for the menu.
pub fn menu_vis_addon() {
    let x_start = SCREEN_WIDTH / 2 - 2 * CARD_WIDTH - 7;
    draw_card(
        &Card {
            suit: Suit::Heart,
            number: randint(0, 13) as u8,
            visible: true,
        },
        Point::new(x_start, 50),
    );
    draw_card(
        &Card {
            suit: Suit::Spade,
            number: randint(0, 13) as u8,
            visible: true,
        },
        Point::new(x_start + CARD_WIDTH + 5, 50),
    );
    draw_card(
        &Card {
            suit: Suit::Diamond,
            number: randint(0, 13) as u8,
            visible: true,
        },
        Point::new(x_start + (CARD_WIDTH + 5) * 2, 50),
    );
    draw_card(
        &Card {
            suit: Suit::Club,
            number: randint(0, 13) as u8,
            visible: true,
        },
        Point::new(x_start + (CARD_WIDTH + 5) * 3, 50),
    );
}
