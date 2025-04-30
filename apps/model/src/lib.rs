#![no_std]
mod game;
use heapless::String;
use numworks_utils::eadk::{display::push_rect_uniform, Color, Point, Rect};

/// Function that will be called by the multiple apps packages
pub use game::start;

/// Function that draws something to represent the game in the multiple apps packages
pub fn thumbnail(pos: Point) {
    push_rect_uniform(
        Rect {
            x: pos.x,
            y: pos.y,
            width: 50,
            height: 50,
        },
        Color::GREEN,
    );
}

/// Function that returns a string with the name of the app (the text to write) for the multiple apps packages
pub fn get_name() -> String<15> {
    String::try_from("TEST").unwrap()
}
