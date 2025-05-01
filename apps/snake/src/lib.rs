#![no_std]

mod game_snake;
mod snake_ui;

use heapless::String;
use numworks_utils::{
    eadk::{display::push_rect_uniform, Color, Point, Rect},
    utils::CENTER,
};

pub use game_snake::start;
use snake_ui::{BCKD_GRAY, CASE_SIZE, DARK_GREEN, PIXELS, TILEMAP};

pub fn thumbnail(_: Point) {
    push_rect_uniform(
        Rect {
            x: CENTER.x - 75,
            y: 15,
            width: 150,
            height: 100,
        },
        Color::BLACK,
    );
    push_rect_uniform(
        Rect {
            x: CENTER.x - 72,
            y: 18,
            width: 144,
            height: 94,
        },
        BCKD_GRAY,
    );
    TILEMAP.draw_tile::<PIXELS>(
        Point::new(CENTER.x - CASE_SIZE / 2, 15 + 50 - CASE_SIZE),
        Point::new(3, 0),
        2,
        false,
    );
    push_rect_uniform(
        Rect {
            x: CENTER.x - CASE_SIZE * 6 - CASE_SIZE / 2,
            y: 15 + 50 - CASE_SIZE,
            width: CASE_SIZE * 6,
            height: CASE_SIZE * 2,
        },
        DARK_GREEN,
    );
    TILEMAP.draw_tile::<PIXELS>(
        Point::new(
            CENTER.x + CASE_SIZE * 3 + CASE_SIZE / 2,
            15 + 50 - CASE_SIZE,
        ),
        Point::new(0, 0),
        2,
        true,
    );
}

pub fn get_name() -> String<15> {
    String::try_from("Snake\0").unwrap()
}
