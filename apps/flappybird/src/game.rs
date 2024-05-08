use heapless::Vec;
use numworks_utils::{
    eadk::{
        display::{self, SCREEN_WIDTH},
        key, keyboard, Point,
    },
    utils::draw_centered_string,
};

use crate::{
    bird::Player,
    eadk::Color,
    flappy_ui::{draw_bird, draw_constant_ui, draw_ui, BACKGROUND, TILESET},
    menu::{menu, MyOption, OptionType},
    pipes::Pipes,
    utils::{fill_screen, ColorConfig},
};

// This dictates the principal colors that will be used for menu etc
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

/// The entire game is here.
pub fn game(_exemple: bool) -> u8 {
    fill_screen(BACKGROUND);
    draw_constant_ui();

    let mut pipes = Pipes::new(3, 75);
    pipes.draw_self();

    let mut pipes2 = Pipes::new(3, 75);

    let mut bird = Player::new();
    bird.draw_self();

    let mut score: u16 = 0;
    let mut can_increase_speed: bool = true;

    let mut delay: u8 = 0;
    let mut first_pipes: bool = false;

    loop {
        display::wait_for_vblank();

        let scan = keyboard::scan();
        bird.action_function(scan);
        if !first_pipes {
            let add = pipes.action_function();
            if add != 0 {
                can_increase_speed = true;
            }
            score += add;
            first_pipes = true;
        } else {
            if delay > 65 {
                let add = pipes2.action_function();
                score += add;
                if add != 0 {
                    can_increase_speed = true;
                }
            }
            first_pipes = false;
        }
        if can_increase_speed && score % 5 == 0 {
            pipes.increase_speed();
            pipes2.increase_speed();
            can_increase_speed = false;
        }
        if bird_collide_with(&bird, &pipes) || bird_collide_with(&bird, &pipes2) {
            draw_centered_string("GAME OVER\0", 100, true, &COLOR_CONFIG, true);
            loop {
                let scan = keyboard::scan();
                if scan.key_down(key::EXE) {
                    break;
                }
            }
            break;
        }
        if scan.key_down(key::EXE) {
            break;
        }
        draw_ui(score);
        delay = delay.saturating_add(1);
    }

    1
}

const NICE_COLLISION_MARGIN: u16 = 2;

fn bird_collide_with(bird: &Player, pipes: &Pipes) -> bool {
    if bird.x_pos + TILESET.tile_size < pipes.x_pos.saturating_sub(5)
        || bird.x_pos > pipes.x_pos + TILESET.tile_size * 2
    {
        return false;
    } else if bird.y_pos < pipes.interval.0.saturating_sub(NICE_COLLISION_MARGIN)
        || bird.y_pos + TILESET.tile_size > pipes.interval.1 + NICE_COLLISION_MARGIN
    {
        return true;
    } else {
        return false;
    }
}
