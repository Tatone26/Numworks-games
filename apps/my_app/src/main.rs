#![no_std]
#![no_main]
pub mod eadk;
use eadk::{Color, display};

mod menu;
use menu::menu;

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
    let start = menu("CECI EST UN TITRE!", Color::BLACK, Color::WHITE, Color::GREEN);
    display::wait_for_vblank();
    if start == 1 {
        return;
    } else {
        return;
    }
}
