#![no_std]
#![no_main]
pub mod eadk;

pub mod menu;

pub mod utils;

mod game;

#[used]
#[link_section = ".rodata.eadk_app_name"]
pub static EADK_APP_NAME: [u8; 7] = *b"MyName\0"; // You can change array size to correspond

#[used]
#[link_section = ".rodata.eadk_api_level"]
pub static EADK_APP_API_LEVEL: u32 = 0;

#[used]
#[link_section = ".rodata.eadk_app_icon"]
pub static EADK_APP_ICON: [u8; 2286] = *include_bytes!("../target/icon.nwi"); // You need to create your icon by doing : nwlink png-nwi *path to png* target/icon.nwi
// You can change array size too
#[no_mangle]
pub fn main() {
    game::start();
}
