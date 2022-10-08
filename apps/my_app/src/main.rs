#![no_std]
#![no_main]

use eadk::{Point, display, Rect, Color, keyboard, key, timing};

pub mod eadk;

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
    let mut point : Point = Point::new(0, 0);
    loop{
        let keyboard_state = keyboard::scan();
        if keyboard_state.key_down(key::ONE){
            point.x += 5;
            point.y += 5;
        };
        display::push_rect_uniform(Rect{x : point.x, y : point.y, width : 10, height : 10}, Color::RED);
        timing::msleep(20);
    };
}