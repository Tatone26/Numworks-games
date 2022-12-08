use crate::{
    eadk::{
        display::{self, SCREEN_HEIGHT, SCREEN_WIDTH, push_rect_uniform},
        key, keyboard, timing, Rect,
    },
    menu::MenuConfig,
    utils::{
        draw_centered_string, fill_screen, wait_for_no_keydown, ColorConfig, CENTER,
        LARGE_CHAR_HEIGHT, get_centered_text_x_coordo, fading,
    },
};

/// In milliseconds, the time between each action if we keep a key pushed.
const REPETITION_SPEED: u16 = 200;

pub struct App {
    pub name: &'static str,
    pub launching_function: fn() -> (),
    pub rules: &'static str,
}

/// The working part of any menu.
/// It currently works for only three choices (Start, Option and Exit for exemple)
/// Returns 1 for first_choice, 2 for second_choice and 0 for null_choice
pub fn apps_menu<const N: usize>(color: &ColorConfig, apps: [&App; N]) -> u8 {
    let mut cursor_pos: u8 = 0;
    fill_screen(color.bckgrd);
    draw_selection(&apps, cursor_pos, color);
    display::wait_for_vblank();
    wait_for_no_keydown();
    let mut last_action: u64 = timing::millis();
    let mut last_action_key: u32 = key::ALPHA;
    loop {
        let keyboard_state = keyboard::scan();
        if (keyboard_state.key_down(key::DOWN) | keyboard_state.key_down(key::UP))
            & (timing::millis() >= (last_action + REPETITION_SPEED as u64))
        {
            if keyboard_state.key_down(key::DOWN) {
                if cursor_pos == N as u8 - 1 {
                    cursor_pos = 0
                } else {
                    cursor_pos += 1
                }
                last_action_key = key::DOWN;
            } else if keyboard_state.key_down(key::UP) {
                if cursor_pos == 0 {
                    cursor_pos = N as u8 - 1
                } else {
                    cursor_pos -= 1
                }
                last_action_key = key::UP;
            }
            draw_selection(&apps, cursor_pos, color);
            display::wait_for_vblank();
            last_action = timing::millis();
        } else if keyboard_state.key_down(key::OK) {
            fading(150);
            (apps[cursor_pos as usize].launching_function)();
            fill_screen(color.bckgrd);
            draw_selection(&apps, cursor_pos, color);
            display::wait_for_vblank();
        } else if keyboard_state.key_down(key::BACK) {
            return 0;
        } else if !keyboard_state.key_down(last_action_key) {
            // if we let go of the key
            last_action = timing::millis() - REPETITION_SPEED as u64;
        }
    }
}

fn draw_selection<const N: usize>(apps: &[&App; N], pos: u8, color: &ColorConfig) {
    fill_screen(color.bckgrd);
    let mini_afficher: u8;
    if pos <= 5 {
        mini_afficher = 0
    } else {
        mini_afficher = pos - 5
    }
    for i in mini_afficher..N as u8 {
        let line = apps[i as usize];
        draw_centered_string(
            line.name,
            15 + (LARGE_CHAR_HEIGHT + 15) * (i - mini_afficher) as u16,
            true,
            color,
            i == pos,
        );
        if i == pos {
            push_rect_uniform(
                Rect {
                    x: get_centered_text_x_coordo(apps[pos as usize].name, true) - 15,
                    y: 15 + (LARGE_CHAR_HEIGHT + 15) * (i - mini_afficher) as u16 + LARGE_CHAR_HEIGHT / 2,
                    width: 10,
                    height: 2,
                },
                color.alt,
            );
        }
    }
    
}
