#![no_std]
#![no_main]

use numworks_utils::eadk;

mod game_solitaire;
pub mod ui_solitaire;

#[used]
#[link_section = ".rodata.eadk_app_name"]
pub static EADK_APP_NAME: [u8; 10] = *b"SOLITAIRE\0";

#[used]
#[link_section = ".rodata.eadk_api_level"]
pub static EADK_APP_API_LEVEL: u32 = 0;

#[used]
#[link_section = ".rodata.eadk_app_icon"]
pub static EADK_APP_ICON: [u8; 3299] = *include_bytes!("../target/icon.nwi");

#[no_mangle]
pub fn main() {
    game_solitaire::start();
}
