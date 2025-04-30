#![no_std]

mod bird;
mod flappy_ui;
mod game;
mod pipes;

use flappy_ui::{draw_bird, draw_pipe_entrance, BACKGROUND, PIXELS, TILESET, TILESET_TILE_SIZE};
pub use game::start;

use heapless::String;
use numworks_utils::{
    eadk::{display::push_rect_uniform, Color, Point, Rect},
    utils::CENTER,
};
use pipes::OptiTiles;
pub fn get_name() -> String<15> {
    String::try_from("Flappy Bird").unwrap()
}

pub fn thumbnail(_: Point) {
    let tiles = OptiTiles {
        left_bottom_tile: TILESET.get_tile::<PIXELS>(Point { x: 0, y: 2 }),
        right_bottom_tile: TILESET.get_tile::<PIXELS>(Point { x: 3, y: 2 }),
        left_top_tile: TILESET.get_tile::<PIXELS>(Point { x: 0, y: 1 }),
        right_top_tile: TILESET.get_tile::<PIXELS>(Point { x: 3, y: 1 }),
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
        BACKGROUND,
    );

    draw_pipe_entrance(
        CENTER.x + 10,
        18,
        &tiles.left_top_tile,
        &tiles.right_top_tile,
        true,
    );
    draw_pipe_entrance(
        CENTER.x + 10,
        97 + 15 - TILESET_TILE_SIZE,
        &tiles.left_bottom_tile,
        &tiles.right_bottom_tile,
        false,
    );
    draw_bird(
        Point {
            x: CENTER.x - 35,
            y: 15 + TILESET_TILE_SIZE + TILESET_TILE_SIZE,
        },
        0,
    );
}
