#![no_std]

mod events;
mod flappy_ui;
mod game;
mod menu;
mod pipes;

use flappy_ui::BACKGROUND;
pub use menu::start;

use heapless::String;
use numworks_utils::{
    eadk::{display::push_rect_uniform, Color, Point, Rect},
    utils::CENTER,
};

use crate::flappy_ui::{
    ANIM_BIRD_FALL, ANIM_CLOUD, ANIM_PIPE_LIP_BOT, ANIM_PIPE_LIP_TOP, TILESET_TILE_SIZE,
};

pub fn get_name() -> String<15> {
    String::try_from("Flappy Bird\0").unwrap()
}

pub fn thumbnail(_: Point) {
    // Outer border frame (cadre)
    push_rect_uniform(
        Rect {
            x: CENTER.x - 75,
            y: 15,
            width: 150,
            height: 100,
        },
        Color::BLACK,
    );
    // Inner background fill
    push_rect_uniform(
        Rect {
            x: CENTER.x - 72,
            y: 18,
            width: 144,
            height: 94,
        },
        BACKGROUND,
    );

    ANIM_BIRD_FALL.draw_at(
        Point {
            x: CENTER.x - 30,
            y: 55,
        },
        0,
    );

    ANIM_PIPE_LIP_BOT.draw_at(
        Point {
            x: CENTER.x,
            y: 18 + 94 - TILESET_TILE_SIZE,
        },
        0,
    );

    ANIM_PIPE_LIP_TOP.draw_at(Point { x: CENTER.x, y: 18 }, 0);

    ANIM_CLOUD.draw_at(
        Point {
            x: CENTER.x - 65,
            y: 45 - TILESET_TILE_SIZE,
        },
        0,
    );
}
