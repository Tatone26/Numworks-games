#![no_std]
#![no_main]

use numworks_utils::eadk;
use numworks_utils::menu;
use numworks_utils::utils;

mod flappy_ui;
mod game;

#[used]
#[link_section = ".rodata.eadk_app_name"]
pub static EADK_APP_NAME: [u8; 12] = *b"Flappy Bird\0";

#[used]
#[link_section = ".rodata.eadk_api_level"]
pub static EADK_APP_API_LEVEL: u32 = 0;

#[used]
#[link_section = ".rodata.eadk_app_icon"]
pub static EADK_APP_ICON: [u8; 3214] = *include_bytes!("../target/icon.nwi");

#[no_mangle]
pub fn main() {
    game::start();
}
