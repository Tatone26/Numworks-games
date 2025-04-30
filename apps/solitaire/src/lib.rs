#![no_std]

mod cards;
mod game_solitaire;
mod table;
mod ui_solitaire;

use cards::{Card, Suit};
use heapless::String;
use numworks_utils::{
    eadk::{display::push_rect_uniform, Color, Point, Rect},
    utils::{randint, CENTER},
};

pub use game_solitaire::start;
use ui_solitaire::{draw_card, BACKGROUND_COLOR, CARD_WIDTH};

pub fn thumbnail(_: Point) {
    push_rect_uniform(
        Rect {
            x: CENTER.x - 85,
            y: 15,
            width: 170,
            height: 100,
        },
        Color::BLACK,
    );
    push_rect_uniform(
        Rect {
            x: CENTER.x - 82,
            y: 18,
            width: 164,
            height: 94,
        },
        BACKGROUND_COLOR,
    );
    let x_start = CENTER.x - 2 * CARD_WIDTH - 7;
    draw_card(
        &Card {
            suit: Suit::Heart,
            number: randint(0, 13) as u8,
            visible: true,
        },
        Point::new(x_start, 35),
    );
    draw_card(
        &Card {
            suit: Suit::Spade,
            number: randint(0, 13) as u8,
            visible: true,
        },
        Point::new(x_start + CARD_WIDTH + 5, 35),
    );
    draw_card(
        &Card {
            suit: Suit::Diamond,
            number: randint(0, 13) as u8,
            visible: true,
        },
        Point::new(x_start + (CARD_WIDTH + 5) * 2, 35),
    );
    draw_card(
        &Card {
            suit: Suit::Club,
            number: randint(0, 13) as u8,
            visible: true,
        },
        Point::new(x_start + (CARD_WIDTH + 5) * 3, 35),
    );
}

pub fn get_name() -> String<15> {
    String::try_from("Solitaire").unwrap()
}
