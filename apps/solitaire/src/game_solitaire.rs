use heapless::Vec;
use numworks_utils::utils::draw_tile;



use crate::{
    eadk::{
        display::{self, push_rect_uniform},
        key, keyboard, Color, Point, Rect,
    },
    menu::{menu, MyOption, OptionType},
    utils::{
        draw_string_cfg, fill_screen, wait_for_no_keydown, ColorConfig, Tileset,
    }, ui_solitaire::ui_test,
};

// This dictates the principal colors that will be used
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: Color::WHITE,
    alt: Color::RED,
};

static mut EXEMPLE: bool = false;

fn vis_addon() {
    push_rect_uniform(
        Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        },
        Color::BLACK,
    );
}
/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut MyOption; 1] = [&mut MyOption {
        name: "Option !\0",
        value: 0,
        possible_values: {
            let mut v = Vec::new();
            unsafe { v.push_unchecked((OptionType::Bool(true), "True\0")) };
            unsafe { v.push_unchecked((OptionType::Bool(false), "False\0")) };
            v
        },
    }];
    loop {
        let start = menu(
            "SNAKE 2.0\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./model_controls.txt"),
        );
        // The menu does everything itself !
        if start == 0 {
            unsafe {
                EXEMPLE = opt[0].get_param_value(); // You could use mutable statics, but it is not very good
            }
            loop {
                // a loop where the game is played again and again, which means it should be 100% contained after the menu
                let action = game(opt[0].get_param_value()); // calling the game based on the parameters is better
                if action == 2 {
                    // 2 means quitting
                    return;
                } else if action == 1 {
                    // 1 means back to menu
                    break;
                } // if action == 0 : rejouer
            }
        } else {
            return;
        }
    }
}



/// The entire game is here.
pub fn game(_exemple: bool) -> u8 {
    {
        fill_screen(Color::WHITE);
        draw_string_cfg(
            "Push <OK> for some magic ! <Back> to quit back to menu.\0",
            Point::new(0, 0),
            false,
            &COLOR_CONFIG,
            true,
        );
        loop {
            let keyboard_state = keyboard::scan();
            if keyboard_state.key_down(key::OK) {
                ui_test();
            } else if keyboard_state.key_down(key::BACK) {
                break;
            }
        }
    }
    fill_screen(Color::GREEN);
    return 1;
}
