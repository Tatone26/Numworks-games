use heapless::Vec;
use numworks_utils::eadk::{
    display::{self, SCREEN_HEIGHT, SCREEN_WIDTH},
    key, keyboard, timing, Point,
};

use crate::{
    eadk::Color,
    flappy_ui::{
        clear_bottom_pipes, clear_tile, clear_top_pipe, draw_bird, draw_bottom_pipes,
        draw_constant_ui, draw_top_pipe, draw_ui, BACKGROUND, TILESET,
    },
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

/// number of pixels of the window border.
pub const WINDOW_SIZE: u16 = 20;

const X_BIRD_POS: u16 = SCREEN_WIDTH / 3;

const GRAVITY: f32 = 0.7;
const JUMP_POWER: f32 = 7.0;

/// The entire game is here.
pub fn game(_exemple: bool) -> u8 {
    fill_screen(BACKGROUND);
    draw_constant_ui();
    let mut pos = Point {
        x: X_BIRD_POS,
        y: SCREEN_HEIGHT / 2,
    };

    let pipes_interval = (50, 150);

    // !! more to the left = bad wrapping.
    let mut pipes_x_pos = SCREEN_WIDTH - 3 * TILESET.tile_size;
    let mut last_pipes_pos: u16 = pipes_x_pos;
    let mut pressed: bool = false;

    const PIPES_REFRESH_SPEED: u64 = 75;
    let mut pipes_speed = 5;
    let mut bottom_need_to_move: bool = true;
    let mut last_move: u64 = timing::millis();
    let mut anim: u8 = 0;
    let mut y_speed: f32 = 0.0;
    loop {
        y_speed += GRAVITY;
        display::wait_for_vblank();
        if bottom_need_to_move {
            clear_bottom_pipes(last_pipes_pos, pipes_interval);
            draw_bottom_pipes(pipes_x_pos, pipes_interval);
            bottom_need_to_move = false;
        }
        if timing::millis() - last_move >= PIPES_REFRESH_SPEED {
            clear_top_pipe(pipes_x_pos, pipes_interval);
            last_pipes_pos = pipes_x_pos;
            if pipes_x_pos <= pipes_speed {
                pipes_x_pos = SCREEN_WIDTH - 3 * TILESET.tile_size;
                pipes_speed += 1;
            } else {
                pipes_x_pos -= pipes_speed;
            }
            draw_top_pipe(pipes_x_pos, pipes_interval);
            // draw_ui();
            last_move = timing::millis();
            bottom_need_to_move = true;
        }

        clear_tile(pos);
        let new_pos = pos.y as i16 + y_speed as i16;
        if new_pos <= WINDOW_SIZE as i16 {
            y_speed += GRAVITY; // head bump
            pos.y = WINDOW_SIZE;
        } else if new_pos >= (SCREEN_HEIGHT - WINDOW_SIZE - TILESET.tile_size) as i16 {
            y_speed = 0.1; // foot bump
            pos.y = SCREEN_HEIGHT - WINDOW_SIZE - TILESET.tile_size;
        } else {
            pos.y = new_pos as u16;
        }
        draw_bird(pos, anim % 2);
        anim = anim.wrapping_add(1);

        let scan = keyboard::scan();
        if !pressed && scan.key_down(key::OK) {
            pressed = true;
            y_speed = -JUMP_POWER;
        } else if scan.key_down(key::UP) {
            break;
        } else if !scan.key_down(key::OK) {
            pressed = false;
        }
        draw_ui();
    }

    1
}
