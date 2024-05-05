use heapless::Vec;
use numworks_utils::eadk::{
    display::{self, SCREEN_HEIGHT, SCREEN_WIDTH},
    key, keyboard, Point,
};

use crate::{
    eadk::Color,
    flappy_ui::{clear_tile, draw_bird, BACKGROUND, TILESET},
    menu::{menu, MyOption, OptionType},
    utils::{fill_screen, ColorConfig},
};

// This dictates the principal colors that will be used
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: Color::WHITE,
    alt: Color::RED,
};

fn vis_addon() {
    // TODO
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
            "FLAPPY BIRD\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./model_controls.txt"),
        );
        // The menu does everything itself !
        if start == 0 {
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

const X_BIRD_POS: u16 = SCREEN_WIDTH / 3;

const GRAVITY: f32 = 0.7;
const JUMP_POWER: f32 = 10.0;

/// The entire game is here.
pub fn game(_exemple: bool) -> u8 {
    fill_screen(BACKGROUND);
    let mut pos = Point {
        x: X_BIRD_POS,
        y: SCREEN_HEIGHT / 2,
    };

    let mut pressed: bool = false;

    let mut y_speed: f32 = 0.0;
    loop {
        y_speed += GRAVITY;

        display::wait_for_vblank();
        clear_tile(pos);
        let new_pos = pos.y as i16 + y_speed as i16;
        if new_pos <= 0 {
            y_speed += GRAVITY; // head bump
            pos.y = 0;
        } else if new_pos >= (SCREEN_HEIGHT - TILESET.tile_size) as i16 {
            y_speed = 0.1; // foot bump
            pos.y = SCREEN_HEIGHT - TILESET.tile_size;
        } else {
            pos.y = new_pos as u16;
        }
        draw_bird(pos);

        let scan = keyboard::scan();
        if !pressed && scan.key_down(key::OK) {
            pressed = true;
            y_speed = -JUMP_POWER;
        } else if scan.key_down(key::UP) {
            break;
        } else if !scan.key_down(key::OK) {
            pressed = false;
        }
    }

    1
}
