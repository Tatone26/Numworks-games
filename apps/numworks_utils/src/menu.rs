pub mod settings;

use settings::{set_values_from_file, Setting};

use crate::eadk::display::{push_rect_uniform, wait_for_vblank, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::eadk::{display, key, keyboard, timing, Point, Rect};
use crate::graphical::{draw_centered_string, draw_string_cfg, fading, fill_screen, ColorConfig};
use crate::storage::open_file;
use crate::utils::{
    get_centered_text_x_coordo, get_string_pixel_size, wait_for_no_keydown, CENTER,
    LARGE_CHAR_HEIGHT, SMALL_CHAR_HEIGHT,
};

/// Duration of fadings, in milliseconds
const FADING_TIME: u32 = 500;

/// In milliseconds, the time between each action if we keep a key pushed.
const REPETITION_SPEED: u16 = 200;

/// For the [options] menu, the space between each line.
const SPACE_BETWEEN_LINES: u16 = LARGE_CHAR_HEIGHT;

/// Used to describe the menu texts, size, pos...*
/// This is THE thing to use to construct a beautiful menu !
/// It may not be really intuitive, but it is efficient.
pub struct MenuConfig {
    /// List of all the choices of the menu
    pub choices: &'static [&'static str],
    /// How much between text and sides, corresponding to the "border" of the menu.
    pub rect_margins: (u16, u16),
    /// Width and Height of the entire menu (not too small or text will glitch out !)
    pub dimensions: (u16, u16),
    /// Offset of the entire menu, from the center point
    pub offset: (i16, i16),
    /// The return value bound to the BACK key
    pub back_key_return: u8,
}

/// Creates a fully fonctional start menu, with [settings::settings] as second choice, "How to Play" as third choice
///
/// [vis_addon] is a function that does not take any argument and will be run when the menu is displayed (on top of the background, below the text)
///
/// [controls_text] is the string to display in the "How to play" page
pub fn start_menu(
    title: &str,
    opt: &mut [&mut Setting],
    cfg: &ColorConfig,
    vis_addon: fn(),
    controls_text: &str,
    filename: &str,
) -> u8 {
    open_file(filename);
    set_values_from_file(opt, filename);

    // the loop is to be able to go back to the main menu after doing settings or controls
    loop {
        // visual stuff (background, vis_addon, title)
        wait_for_vblank();
        fill_screen(cfg.bckgrd);
        vis_addon();
        draw_centered_string(title, 20, true, cfg, false);
        // main stuff : selection !
        let action = selection(
            cfg,
            &MenuConfig {
                choices: &["Play\0", "Settings\0", "How to Play\0", "Exit\0"],
                rect_margins: (10, 10),
                dimensions: (SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
                offset: (0, (SCREEN_HEIGHT / 5) as i16),
                back_key_return: 3,
            },
            false,
        );
        if action == 1 {
            settings::settings(opt, cfg, filename);
        } else if action == 2 {
            controls(controls_text, cfg);
        } else {
            fading(FADING_TIME);
            return action;
        }
    }
}

/// Creates a fully fonctional pause menu, mostly an exemple
pub fn pause_menu(cfg: &ColorConfig, y_offset: i16) -> u8 {
    selection(
        cfg,
        &MenuConfig {
            choices: &["Resume\0", "Menu\0", "Exit\0"],
            rect_margins: (20, 10),
            dimensions: (
                SCREEN_WIDTH * 2 / 5,
                LARGE_CHAR_HEIGHT * 3 + SPACE_BETWEEN_LINES * 4,
            ),
            offset: (0, y_offset),
            back_key_return: 0,
        },
        false,
    )
}

/// The working part of any menu.
/// Can work for any number of choices !
/// Returns the position of the choice in the array. (0 for the first choice, 1 for the second...)
pub fn selection(color: &ColorConfig, config: &MenuConfig, horizontal: bool) -> u8 {
    // visual stuff :
    display::wait_for_vblank();
    display::push_rect_uniform(
        Rect {
            x: (((SCREEN_WIDTH - config.dimensions.0) / 2) as i16 + config.offset.0) as u16,
            y: (((SCREEN_HEIGHT - config.dimensions.1) / 2) as i16 + config.offset.1) as u16,
            width: config.dimensions.0,
            height: config.dimensions.1,
        },
        color.bckgrd,
    );
    for (i, _) in config.choices.iter().enumerate() {
        draw_selection_string(i as u8, color, config, i == 0, horizontal); // pretty important function
    }
    wait_for_no_keydown(); // does not start moving anything unless no keys are pressed.

    let mut cursor_pos: u8 = 0;
    let mut last_action: u64 = timing::millis(); // when did the last action occur (an action being pressing a key), used for auto repetition
    let mut last_action_key: u32 = key::ALPHA; // what key was pressed then
    loop {
        let keyboard_state = keyboard::scan();
        if (keyboard_state.key_down(key::DOWN)
            || keyboard_state.key_down(key::UP)
            || keyboard_state.key_down(key::LEFT)
            || keyboard_state.key_down(key::RIGHT))
            && (timing::millis() >= (last_action + REPETITION_SPEED as u64))
        // this last line allows keeping a button pressed. After [REPETITION_SPEED], we consider the key to be pressed again.
        {
            display::wait_for_vblank();
            draw_selection_string(cursor_pos, color, config, false, horizontal);
            // moving the cursos
            if (!horizontal && keyboard_state.key_down(key::DOWN))
                || (horizontal && keyboard_state.key_down(key::RIGHT))
            {
                cursor_pos += 1;
                if cursor_pos >= config.choices.len() as u8 {
                    // loop back
                    cursor_pos = 0;
                }
                last_action_key = if keyboard_state.key_down(key::DOWN) {
                    key::DOWN
                } else {
                    key::RIGHT
                };
            } else if (!horizontal && keyboard_state.key_down(key::UP))
                || (horizontal && keyboard_state.key_down(key::LEFT))
            {
                if cursor_pos == 0 {
                    // loop back
                    cursor_pos = config.choices.len() as u8 - 1;
                } else {
                    cursor_pos -= 1;
                }
                last_action_key = if keyboard_state.key_down(key::UP) {
                    key::UP
                } else {
                    key::LEFT
                }
            }
            draw_selection_string(cursor_pos, color, config, true, horizontal);
            last_action = timing::millis();
        } else if keyboard_state.key_down(key::OK) {
            // selection
            loop {
                // Activate the press only when "OK" is unpressed.
                let keyboard_state_test = keyboard::scan();
                if !keyboard_state_test.key_down(key::OK) {
                    return cursor_pos;
                }
            }
        } else if keyboard_state.key_down(key::BACK) {
            return config.back_key_return;
        } else if !keyboard_state.key_down(last_action_key) {
            // if we let go of the key, we consider the last_action to have never occured.
            last_action = 0;
        }
    }
}

/// Draws the line corresponding to the position given
/// The options are automatically regularly placed inside the defined menu
/// If selected, color will be the alt one and a little rect thingy will be drawn next to it.
fn draw_selection_string(
    cursor_pos: u8,
    color: &ColorConfig,
    config: &MenuConfig,
    selected: bool,
    horizontal: bool,
) {
    let text: &str = config.choices[cursor_pos as usize];
    // very complicated calculation for the y position when the menu is vertical, but it works lol
    let y_pos: u16 = if !horizontal {
        (CENTER.y - config.dimensions.1 / 2
            + config.rect_margins.1
            + cursor_pos as u16
                * (config.dimensions.1 - config.rect_margins.1 * 2 - LARGE_CHAR_HEIGHT)
                / (config.choices.len() as u16 - 1)) as i16
            + config.offset.1
    } else {
        (CENTER.y - LARGE_CHAR_HEIGHT / 2) as i16 + config.offset.1
    } as u16;

    // complicated for the x position if menu is horizontal
    let x_coordos: u16 = if horizontal {
        if cursor_pos == 0 {
            if config.rect_margins.0 != 0 {
                config.rect_margins.0
            } else {
                10
            }
        } else if cursor_pos == config.choices.len() as u8 - 1 {
            SCREEN_WIDTH
                - get_string_pixel_size(config.choices[cursor_pos as usize], true)
                - config.rect_margins.0
        } else {
            // calculates the position it needs to be at so it is centered. Not perfect, but good enough.
            let start_x = get_string_pixel_size(config.choices[0], true);
            let max_x = SCREEN_WIDTH
                - get_string_pixel_size(config.choices.last().unwrap(), true)
                - config.rect_margins.0;
            start_x + cursor_pos as u16 * (max_x - start_x) / (config.choices.len() as u16 - 1)
                - get_string_pixel_size(config.choices[cursor_pos as usize], true) / 2
        }
    } else {
        (get_centered_text_x_coordo(text, true) as i16 + config.offset.0) as u16
    };

    draw_string_cfg(text, Point::new(x_coordos, y_pos), true, color, selected); // text
    push_rect_uniform(
        // little rect thingy next to the selected text
        Rect {
            x: if x_coordos > 15 { x_coordos - 15 } else { 0 },
            y: y_pos + LARGE_CHAR_HEIGHT / 2,
            width: if x_coordos > 15 { 10 } else { x_coordos - 2 },
            height: 2,
        },
        if selected { color.alt } else { color.bckgrd },
    );
}

/// The "How to Play" page of the menu. Very basic, no selection.
/// It could be better by being able to draw multiple pages, automatically split.
/// Not necessary for now.
fn controls(text: &str, cfg: &ColorConfig) -> u8 {
    wait_for_vblank();
    fill_screen(cfg.bckgrd);
    let back_text = "Menu : <Back>  \0";
    draw_string_cfg(
        back_text,
        Point::new(
            SCREEN_WIDTH - get_string_pixel_size(back_text, false) - 5,
            SCREEN_HEIGHT - SMALL_CHAR_HEIGHT - 5,
        ),
        false,
        cfg,
        false,
    );
    wait_for_vblank();
    for (i, line) in text.lines().enumerate() {
        draw_string_cfg(
            line,
            Point::new(0, i as u16 * (SMALL_CHAR_HEIGHT + 4)),
            false,
            cfg,
            false,
        );
    }
    loop {
        let keyboard_state = keyboard::scan();
        if keyboard_state.key_down(key::BACK) {
            break;
        }
    }
    0
}
