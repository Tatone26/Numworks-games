#![no_std]

mod game_p4;
mod ia_p4;
mod ui_p4;

use heapless::String;
use numworks_utils::{
    eadk::{Color, Point},
    graphical::ColorConfig,
};

pub use game_p4::start;
use ui_p4::draw_selection_coin;

pub fn thumbnail(_: Point) {
    const COLOR_CONFIG: ColorConfig = ColorConfig {
        text: Color::BLACK,
        bckgrd: Color::WHITE,
        alt: Color::from_rgb888(90, 90, 255),
    };
    draw_selection_coin(2, 0, &COLOR_CONFIG, 25);
    draw_selection_coin(3, 1, &COLOR_CONFIG, 25);
    draw_selection_coin(4, 0, &COLOR_CONFIG, 25);
}

pub fn get_name() -> String<15> {
    String::try_from("Connect 4").unwrap()
}
