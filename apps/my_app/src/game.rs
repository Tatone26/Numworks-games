use crate::{
    eadk::{
        self,
        backlight::brightness,
        display::{self, draw_string},
        key, keyboard, Point,
    },
    menu::pause_menu,
    utils::{draw_centered_string, fill_screen},
};
use eadk::Color;
use heapless::String;

pub fn game() {
    fill_screen(Color::BLUE);
    let bs: String<5> = String::from(brightness());
    draw_string(&bs, Point::ZERO, true, Color::WHITE, Color::BLUE);
    loop {
        let keyboard_state = keyboard::scan();
        if keyboard_state.key_down(key::BACKSPACE) {
            let action = pause_menu(Color::RED, Color::WHITE, Color::BLUE);
            if action == 0 {
                fill_screen(Color::WHITE);
                return;
            } else if action == 1 {
                fill_screen(Color::BLACK);
                draw_centered_string("on continue !\0", 50, true, Color::WHITE, Color::BLACK);
            }
        }
        display::wait_for_vblank();
    }
}
