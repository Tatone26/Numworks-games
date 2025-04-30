#![no_std]

mod game_snake;
mod snake_ui;

use heapless::String;
use numworks_utils::eadk::Point;

pub use game_snake::start;
use snake_ui::menu_vis_addon;

pub fn thumbnail(_: Point) {
    menu_vis_addon();
}

pub fn get_name() -> String<15> {
    String::try_from("Snake").unwrap()
}
