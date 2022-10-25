#![no_std]
#![no_main]
pub mod eadk;

use eadk::{display, Color};

mod menu;
use menu::{menu, MyOption};

mod utils;

mod game;

#[used]
#[link_section = ".rodata.eadk_app_name"]
pub static EADK_APP_NAME: [u8; 10] = *b"Snake 2.0\0";

#[used]
#[link_section = ".rodata.eadk_api_level"]
pub static EADK_APP_API_LEVEL: u32 = 0;

#[used]
#[link_section = ".rodata.eadk_app_icon"]
pub static EADK_APP_ICON: [u8; 4250] = *include_bytes!("../target/icon.nwi");

// This constants have to change from one project to another.
const BOOL_OPTIONS_NUMBER: usize = 2;

#[no_mangle]
pub fn main() {
    let mut opt = [
        &mut MyOption {
            name: "TestOption\0",
            value: (true, "Vrai\0"),
            possible_values: [true, false],
            possible_values_str: ["Vrai\0", "Faux\0"],
        },
        &mut MyOption {
            name: "TestOptionYoupi\0",
            value: (false, "Faux\0"),
            possible_values: [true, false],
            possible_values_str: ["Vrai\0", "Faux\0"],
        },
    ];
    let start = menu(
        "CECI EST UN TITRE!\0",
        &mut opt,
        Color::BLACK,
        Color::WHITE,
        Color::GREEN,
    );
    display::wait_for_vblank();
    if start == 1 {
        game::game();
    } else {
        return;
    }
}
