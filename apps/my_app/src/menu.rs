use crate::eadk::{display, key, keyboard, timing, Color};

use crate::utils::{draw_centered_string, fill_screen};

pub fn menu(title: &str, bgd_color: Color) {
    fill_screen(bgd_color);
    draw_centered_string(title, 10u16, false, Color::RED, bgd_color);
    loop {
        let keyboard_state = keyboard::scan();
        if keyboard_state.key_down(key::HOME) {
            break;
        }
    }
    timing::msleep(200);
    display::wait_for_vblank();
}
