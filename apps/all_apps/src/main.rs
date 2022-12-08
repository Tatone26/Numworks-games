#![no_std]
#![no_main]

use eadk::{Color, timing};
use snake::game_snake;
use solitaire::game_solitaire;
use tetris::game_tetris;
use utils::fill_screen;

pub mod eadk;
pub mod menu;
pub mod utils;

pub mod solitaire;
pub mod snake;
pub mod tetris;

#[used]
#[link_section = ".rodata.eadk_app_name"]
pub static EADK_APP_NAME: [u8; 6] = *b"GAMES\0";

#[used]
#[link_section = ".rodata.eadk_api_level"]
pub static EADK_APP_API_LEVEL: u32 = 0;

#[used]
#[link_section = ".rodata.eadk_app_icon"]
pub static EADK_APP_ICON: [u8; 2286] = *include_bytes!("../target/icon.nwi");

#[no_mangle]
pub fn main() {
    fill_screen(Color::BLACK);
    timing::msleep(500);
    game_solitaire::start();
    game_snake::start();
    game_tetris::start();
}
