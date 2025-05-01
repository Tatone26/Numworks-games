#![no_std]

mod game_tetris;
mod tetriminos;
mod ui_tetris;

use game_tetris::BACKGROUND_GRAY;
use heapless::String;
use numworks_utils::{
    eadk::{display::push_rect_uniform, Color, Point, Rect},
    utils::CENTER,
};
use tetriminos::{I_SHAPE, J_SHAPE, L_SHAPE, O_SHAPE, S_SHAPE, T_SHAPE, Z_SHAPE};
use ui_tetris::draw_tetrimino;

pub use game_tetris::start;

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
        BACKGROUND_GRAY,
    );
    let mut tetri = T_SHAPE;
    tetri.pos.y = 8;
    tetri.pos.x = 6;
    draw_tetrimino(&tetri, false);
    let mut tetri2 = S_SHAPE;
    tetri2.pos.y = 8;
    tetri2.pos.x = 4;
    draw_tetrimino(&tetri2, false);
    let mut tetri3 = O_SHAPE;
    tetri3.pos.y = 7;
    tetri3.pos.x = 2;
    draw_tetrimino(&tetri3, false);
    let mut tetri4 = L_SHAPE;
    tetri4.pos.y = 6;
    tetri4.pos.x = 3;
    draw_tetrimino(&tetri4, false);
    let mut tetri5 = I_SHAPE;
    tetri5.pos.y = 3;
    tetri5.pos.x = 8;
    tetri5.rotation = 1;
    draw_tetrimino(&tetri5, false);
    let mut tetri6 = Z_SHAPE;
    tetri6.pos.y = 5;
    tetri6.pos.x = 5;
    tetri6.rotation = 1;
    draw_tetrimino(&tetri6, false);
    let mut tetri7 = J_SHAPE;
    tetri7.pos.y = 4;
    tetri7.pos.x = 3;
    tetri7.rotation = 3;
    draw_tetrimino(&tetri7, false);
}

pub fn get_name() -> String<15> {
    String::try_from("Tetris\0").unwrap()
}
