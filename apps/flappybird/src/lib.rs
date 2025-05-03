#![no_std]

mod bird;
mod flappy_ui;
mod game;
mod pipes;

use flappy_ui::{draw_bird, draw_pipe_entrance, BACKGROUND, TILESET_TILE_SIZE};
pub use game::start;

use heapless::String;
use numworks_utils::{
    eadk::{display::push_rect_uniform, Color, Point, Rect},
    utils::CENTER,
};

pub fn get_name() -> String<15> {
    String::try_from("Flappy Bird\0").unwrap()
}

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
        BACKGROUND,
    );
    draw_pipe_entrance(CENTER.x + 10, 18, true);
    draw_pipe_entrance(CENTER.x + 10, 97 + 15 - TILESET_TILE_SIZE, false);
    draw_bird(
        Point {
            x: CENTER.x - 35,
            y: 15 + TILESET_TILE_SIZE + TILESET_TILE_SIZE,
        },
        0,
    );
}
