#![no_std]
#![no_main]

use eadk::Color;
use global_menu::{apps_menu, App};
use snake::game_snake;
use solitaire::game_solitaire;
use tetris::game_tetris;
use utils::ColorConfig;

pub mod eadk;
pub mod menu;
pub mod utils;

mod global_menu;
pub mod snake;
pub mod solitaire;
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
    apps_menu(
        &ColorConfig {
            text: Color::BLACK,
            bckgrd: Color::WHITE,
            alt: Color::GREEN,
        },
        [&App {
            name: "Solitaire\0",
            launching_function: game_solitaire::start,
            rules: "test\0",
        }, 
        &App {
            name: "Snake\0",
            launching_function: game_snake::start,
            rules: "test\0",
        },
        &App {
            name: "Tetris\0",
            launching_function: game_tetris::start,
            rules: "test\0",
        },], 
    );
}
