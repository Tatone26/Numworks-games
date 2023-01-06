#![no_std]
#![no_main]

pub mod eadk;

pub mod menu;

pub mod utils;

mod game;

#[used]
#[link_section = ".rodata.eadk_app_name"]
pub static EADK_APP_NAME: [u8; 6] = *b"TESTS\0";

#[used]
#[link_section = ".rodata.eadk_api_level"]
pub static EADK_APP_API_LEVEL: u32 = 0;

#[used]
#[link_section = ".rodata.eadk_app_icon"]
pub static EADK_APP_ICON: [u8; 2286] = *include_bytes!("../target/icon.nwi");

#[no_mangle]
pub fn main() {
    game::start();
}
