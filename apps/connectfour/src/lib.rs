#![no_std]

mod game_p4;
mod ia_p4;
mod ui_p4;

use heapless::String;
use numworks_utils::{
    eadk::{display::push_rect_uniform, Color, Point, Rect},
    graphical::ColorConfig,
    utils::CENTER,
};

pub use game_p4::start;
use ui_p4::{draw_selection_coin, COIN_SIZE};

pub fn thumbnail(_: Point) {
    const COLOR_CONFIG: ColorConfig = ColorConfig {
        text: Color::BLACK,
        bckgrd: Color::WHITE,
        alt: Color::from_rgb888(90, 90, 255),
    };
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
        Color::WHITE,
    );
    draw_selection_coin(2, 0, &COLOR_CONFIG, 2 * COIN_SIZE as i16);
    draw_selection_coin(3, 0, &COLOR_CONFIG, 2 * COIN_SIZE as i16);
    draw_selection_coin(4, 1, &COLOR_CONFIG, 2 * COIN_SIZE as i16);
    draw_selection_coin(2, 1, &COLOR_CONFIG, COIN_SIZE as i16 - 5);
    draw_selection_coin(3, 0, &COLOR_CONFIG, COIN_SIZE as i16 - 5);
    draw_selection_coin(3, 1, &COLOR_CONFIG, -10);
}

pub fn get_name() -> String<15> {
    String::try_from("Connect 4\0").unwrap()
}
