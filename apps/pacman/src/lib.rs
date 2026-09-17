#![no_std]

use heapless::String;
use numworks_utils::{
    eadk::{display::push_rect_uniform, Color, Point, Rect},
    utils::CENTER,
};

/// Function that will be called by the multiple apps packages
pub use game::start;

mod game;
mod ghost;
mod levels;
mod moveable;
mod pac_ui;
mod player;

/// Function that returns a string with the name of the app for the launcher menu
pub fn get_name() -> String<15> {
    String::try_from("Pacman\0").unwrap()
}

pub fn thumbnail(_: Point) {
    push_rect_uniform(
        Rect {
            x: CENTER.x - 75,
            y: 15,
            width: 150,
            height: 100,
        },
        Color::BLUE,
    );

    push_rect_uniform(
        Rect {
            x: CENTER.x - 72,
            y: 18,
            width: 144,
            height: 94,
        },
        Color::BLACK,
    );
}
