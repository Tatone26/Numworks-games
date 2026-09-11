#![no_std]
#![no_main]

mod events;
mod flappy_ui;
mod game;
mod menu;
mod pipes;

#[used]
#[link_section = ".rodata.eadk_app_name"]
pub static EADK_APP_NAME: [u8; 7] = *b"FLAPPY\0";

#[used]
#[link_section = ".rodata.eadk_api_level"]
pub static EADK_APP_API_LEVEL: u32 = 0;

#[used]
#[link_section = ".rodata.eadk_app_icon"]
pub static EADK_APP_ICON: [u8; 3305] = *include_bytes!("../target/icon.nwi");

#[no_mangle]
pub fn main() {
    menu::start();
}
