use heapless::Vec;
use numworks_utils::eadk::{
    display::{self, SCREEN_WIDTH},
    key, keyboard, timing, Point,
};

use crate::{
    bird::Player,
    eadk::Color,
    flappy_ui::{
        clear_bottom_pipes, clear_top_pipe, draw_bird, draw_bottom_pipes, draw_constant_ui,
        draw_top_pipe, draw_ui, BACKGROUND, TILESET,
    },
    menu::{menu, MyOption, OptionType},
    utils::{fill_screen, ColorConfig},
};

// This dictates the principal colors that will be used
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: BACKGROUND,
    alt: Color::from_rgb888(255, 166, 65),
};

fn vis_addon() {
    draw_bird(
        Point {
            x: SCREEN_WIDTH / 2 - TILESET.tile_size / 2,
            y: 70,
        },
        0,
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

/// visual, but dictates the speed too.
const PIPES_REFRESH_SPEED: u64 = 75;

/// The entire game is here.
pub fn game(_exemple: bool) -> u8 {
    fill_screen(BACKGROUND);
    draw_constant_ui();

    let pipes_interval = (50, 150);

    // !! more to the left = bad wrapping.
    let mut pipes_x_pos = SCREEN_WIDTH - 3 * TILESET.tile_size;
    let mut last_pipes_pos: u16 = pipes_x_pos;
    let mut pipes_speed = 5;
    let mut bottom_need_to_move: bool = true;
    let mut last_move: u64 = timing::millis();

    let mut bird = Player::new();
    bird.draw_self();

    let mut score: u16 = 0;

    loop {
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
                score += 1;
            } else {
                pipes_x_pos -= pipes_speed;
            }
            draw_top_pipe(pipes_x_pos, pipes_interval);
            // draw_ui();
            last_move = timing::millis();
            bottom_need_to_move = true;
        }
        let scan = keyboard::scan();
        bird.action_function(scan);
        if scan.key_down(key::EXE) {
            break;
        }
        draw_ui(score);
    }

    1
}
