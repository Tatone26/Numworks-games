use crate::{
    eadk::{display::push_rect_uniform, key, keyboard, timing, Color, Point, Rect},
    menu::{menu, pause_menu, MyOption},
    tetriminos::{Tetrimino, I_SHAPE, J_SHAPE, L_SHAPE, O_SHAPE, S_SHAPE, T_SHAPE, Z_SHAPE},
    ui::{draw_stable_ui, draw_level},
    utils::{draw_centered_string, randint, ColorConfig, CENTER},
};

/// The number of Boolean Options used. Public so menu() can use it.
pub const BOOL_OPTIONS_NUMBER: usize = 1;

// This dictates the principal colors that will be used
pub const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::from_rgb888(251, 251, 219),
    bckgrd: Color::from_rgb888(10, 10, 10),
    alt: Color::RED,
};

pub const BACKGROUND_GRAY: Color = Color::from_rgb888(100, 100, 100);
pub const BACKGROUND_DARK_GRAY: Color = Color::from_rgb888(70, 70, 70);

static mut EXEMPLE: bool = false;

fn vis_addon() {
    draw_centered_string("VIS ADDON\0", 70, true, &COLOR_CONFIG, true);
}
/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut MyOption<bool, 2>; BOOL_OPTIONS_NUMBER] = [&mut MyOption {
        name: "Option !\0",
        value: 0,
        possible_values: [(true, "True\0"), (false, "False\0")],
    }];
    loop {
        let start = menu("TETRIS\0", &mut opt, &COLOR_CONFIG, vis_addon); // The menu does everything itself !
        if start == 1 {
            unsafe {
                EXEMPLE = opt[0].get_value().0; // You could use mutable statics, but it is not very good
            }
            loop {
                // a loop where the game is played again and again, which means it should be 100% contained after the menu
                let action = game(); // calling the game based on the parameters is better
                if action == 0 {
                    // 0 means quitting
                    return;
                } else if action == 2 {
                    // 2 means back to menu
                    break;
                } // if action == 1 : rejouer
            }
        } else {
            return;
        }
    }
}

pub const HIGH_SCORE: &'static str = "000000\0"; // Need to be 6 char long !
pub const CASE_SIZE: u16 = 10;
pub const PLAYFIELD_HEIGHT: u16 = 20;
pub const PLAYFIELD_WIDTH: u16 = 10;

const ROTATE_SPEED: u64 = 200;

/// The entire game is here.
pub fn game() -> u8 {
    draw_stable_ui();
    let mut actual_tetri: Tetrimino = get_new_tetrimino();
    draw_tetrimino(&actual_tetri, false);
    let mut fall_speed: u64 = 500; // decrease with level increase
    let mut last_fall_time: u64 = timing::millis();
    let mut last_move_time: u64 = timing::millis();
    let mut move_button_down: bool = false;
    let mut last_rotate_time: u64 = timing::millis();
    let mut rotate_button_down: bool = false;
    loop {
        let keyboard_state = keyboard::scan();
        if (!move_button_down | (last_move_time + fall_speed < timing::millis())) // if we touch the button for the first time in this frame, or if we maintained it pressed and some time has passed
            & (keyboard_state.key_down(key::LEFT) | keyboard_state.key_down(key::RIGHT))
        {
            // MOVE
            last_move_time = timing::millis();
            move_button_down = true;
        } else if (!rotate_button_down | (last_rotate_time + ROTATE_SPEED < timing::millis()))
            & keyboard_state.key_down(key::OK)
        {
            // ROTATE
            draw_tetrimino(&actual_tetri, true);
            actual_tetri.rotate_left();
            draw_tetrimino(&actual_tetri, false);
            last_rotate_time = timing::millis();
            rotate_button_down = true;
        }
        if last_fall_time + fall_speed < timing::millis() {
            // FALL
            draw_tetrimino(&actual_tetri, true);
            actual_tetri.pos.y += 1;
            draw_tetrimino(&actual_tetri, false);
            last_fall_time = timing::millis();
        }

        if move_button_down
            & !(keyboard_state.key_down(key::LEFT) | keyboard_state.key_down(key::RIGHT))
        {
            move_button_down = false;
        }

        if rotate_button_down & !keyboard_state.key_down(key::OK) {
            rotate_button_down = false;
        }

        if keyboard_state.key_down(key::BACKSPACE) {
            // PAUSE MENU
            let action = pause_menu(&COLOR_CONFIG, 0);
            if action != 1 {
                return action;
            } else {
                draw_stable_ui();
                // redraws everything that needs it
                return action;
            }
        }
    }
}

/// Draws a given tetrimino.
fn draw_tetrimino(tetri: &Tetrimino, clear: bool) {
    // TODO : Needs to not try to draw when negative position !!!
    for p in tetri.get_blocks() {
        push_rect_uniform(
            Rect {
                x: CENTER.x - (PLAYFIELD_WIDTH / 2) * CASE_SIZE
                    + ((tetri.pos.x + p.0) as u16) * CASE_SIZE,
                y: CASE_SIZE * 2 + ((tetri.pos.y + p.1) as u16) * CASE_SIZE,
                width: CASE_SIZE,
                height: CASE_SIZE,
            },
            if clear {
                COLOR_CONFIG.bckgrd
            } else {
                tetri.color
            },
        )
    }
}

fn get_new_tetrimino() -> Tetrimino {
    match randint(0, 6) {
        0 => return T_SHAPE,
        1 => return J_SHAPE,
        2 => return O_SHAPE,
        3 => return L_SHAPE,
        4 => return I_SHAPE,
        5 => return S_SHAPE,
        _ => return Z_SHAPE,
    }
}
