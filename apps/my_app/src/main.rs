#![no_std]
#![no_main]
pub mod eadk;
use eadk::{key, keyboard, Color};

mod menu;
use menu::menu;
use utils::fill_screen;

mod utils;

#[used]
#[link_section = ".rodata.eadk_app_name"]
pub static EADK_APP_NAME: [u8; 10] = *b"HelloRust\0";

#[used]
#[link_section = ".rodata.eadk_api_level"]
pub static EADK_APP_API_LEVEL: u32 = 0;

#[used]
#[link_section = ".rodata.eadk_app_icon"]
pub static EADK_APP_ICON: [u8; 4250] = *include_bytes!("../target/icon.nwi");

#[no_mangle]
pub fn main() {
    menu("CECI EST UN TITRE!", Color::WHITE);
    fill_screen(Color::BLUE);
    loop {
        let keyboard_state = keyboard::scan();
        if keyboard_state.key_down(key::EXE) {
            break;
        };
    }
}
