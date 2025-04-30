#![no_std]

mod game_tetris;
mod tetriminos;
mod ui_tetris;

pub use game_tetris::start;
use heapless::String;
use numworks_utils::eadk::Point;
use tetriminos::T_SHAPE;
use ui_tetris::draw_tetrimino;

pub fn thumbnail(_: Point) {
    let mut tetri = T_SHAPE;
    tetri.pos.y = 5;
    draw_tetrimino(&tetri, false);
}

pub fn get_name() -> String<15> {
    String::try_from("Tetris").unwrap()
}
