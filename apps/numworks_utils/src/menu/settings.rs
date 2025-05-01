use heapless::Vec;

use crate::{
    eadk::{
        display::{self, push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, timing, Point, Rect,
    },
    graphical::{draw_centered_string, draw_string_cfg, fill_screen, ColorConfig},
    storage::{read_file, write_data, MAX_STORAGE_VALUES},
    utils::{
        get_string_pixel_size, string_from_u32, wait_for_no_keydown, LARGE_CHAR_HEIGHT,
        SMALL_CHAR_HEIGHT,
    },
    widgets::numberinput::numberinput,
};

use super::{REPETITION_SPEED, SPACE_BETWEEN_LINES};

// The maximum number of different values an option can take.
pub const MAX_SETTINGS_VALUES: usize = MAX_STORAGE_VALUES;

/// Where in the x coordinate will the names of the [option][Setting] be placed
const XPOS_NAMES: u16 = 30;
/// Where in the x coordinate will the values ([option][Setting].value.1) be placed
const XPOS_VALUES: u16 = 170;

/// An Option of T type, with COUNT possible values
/// Old values are represented as u32.
#[derive(Debug)]
pub struct Setting {
    pub name: &'static str,
    pub choice: usize, // the selected position in the arrays
    pub values: Vec<u32, MAX_SETTINGS_VALUES>,
    pub texts: Vec<&'static str, MAX_SETTINGS_VALUES>,
    pub user_modifiable: bool,
    pub fixed_values: bool,
    // If true : will iterate through the values defined in the vec. If false : will use a number-choosing widget with values[1] and values[2] as the boundaries. (TODO)
    // false will probably mostly be used to store data like high scores.
}

impl Setting {
    /// Set the value to the next one, 0 if needed
    fn increment_value(&mut self) {
        if self.choice == self.values.len() - 1 {
            self.choice = 0
        } else {
            self.choice += 1
        }
    }

    fn decrement_value(&mut self) {
        if self.choice == 0 {
            self.choice = self.values.len() - 1
        } else {
            self.choice -= 1
        }
    }

    pub fn set_value(&mut self, value: u32) {
        self.values[if self.fixed_values { 0 } else { self.choice }] = value;
    }

    #[inline(always)]
    fn get_text(&self) -> &'static str {
        self.texts[self.choice]
    }

    pub fn get_setting_value(&self) -> u32 {
        self.values[self.choice]
    }
}

pub(super) fn set_values_from_file(list: &mut [&mut Setting], filename: &str) {
    let v = read_file(filename);
    if v.is_empty() {
        return;
    }
    for (x, y) in list.iter_mut().zip(v.iter()) {
        if x.fixed_values {
            x.choice = *y as usize;
        } else {
            x.choice = 0;
            x.set_value(*y);
        }
    }
}

pub fn write_values_to_file(list: &mut [&mut Setting], filename: &str) {
    for (i, v) in list.iter().enumerate() {
        if v.fixed_values {
            write_data(filename, Some(i as u32), v.choice as u32);
        } else {
            write_data(filename, Some(i as u32), v.values[0]);
        }
    }
}

/// Create a fully fonctional settings menu, which changes directly the [options][Setting] values. (no settings return)
/// For now, only allows a limited number of settings because making multiple pages is too complicated and I don't have time for that
pub(crate) fn settings(
    list: &mut [&mut Setting],
    cfg: &ColorConfig,
    filename: &str,
    godmode: bool,
) {
    let items_number: u16 = list.iter().filter(|p| godmode || p.user_modifiable).count() as u16;
    let first_y: u16 = match (SCREEN_HEIGHT + LARGE_CHAR_HEIGHT)
        .checked_sub((LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * items_number)
    {
        None | Some(0) => 0,
        x_ @ Some(1u16..=u16::MAX) => x_.unwrap() / 2,
    }; // the y position of the first item. Calculated taking account of the number of options.
       // visual things
    display::wait_for_vblank();
    fill_screen(cfg.bckgrd);
    draw_centered_string(
        "SETTINGS\0",
        if items_number < 6 {
            20u16
        } else {
            10u16.saturating_sub((items_number - 6 + 1) * 5)
        },
        true,
        cfg,
        false,
    );
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
    // printing the options
    for item in list
        .iter()
        .filter(|p| godmode || p.user_modifiable)
        .enumerate()
    {
        let (i, x) = item;
        let y_pos: u16 = first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * (i as u16);
        let large: bool =
            get_string_pixel_size(x.name, true) + XPOS_NAMES < XPOS_VALUES - LARGE_CHAR_HEIGHT * 2;
        display::draw_string(
            x.name,
            Point::new(
                XPOS_NAMES,
                y_pos
                    + if !large {
                        (LARGE_CHAR_HEIGHT - SMALL_CHAR_HEIGHT) / 2
                    } else {
                        0
                    },
            ),
            large,
            cfg.text,
            cfg.bckgrd,
        );
        draw_setting_selection(x, y_pos, i == 0, cfg)
    }
    setting_selection(list, cfg, first_y, godmode);
    write_values_to_file(list, filename);
}

/// Takes care of all the difficult stuff, like moving the cursor and modifying the text.
///
/// Similar to [super::selection] but redone to account for more things (and not account for horizontal versions)
fn setting_selection(list: &mut [&mut Setting], cfg: &ColorConfig, first_y: u16, godmode: bool) {
    wait_for_no_keydown();

    let mut cursor_pos: u16 = 0;
    let mut last_action: u64 = timing::millis();
    let mut last_action_key: u32 = key::ALPHA;

    loop {
        let keyboard_scan = keyboard::scan();
        if keyboard_scan.key_down(key::BACK) {
            break;
        } else if (keyboard_scan.key_down(key::UP) || keyboard_scan.key_down(key::DOWN))
            && (timing::millis() >= (last_action + REPETITION_SPEED as u64))
        {
            display::wait_for_vblank();

            let current_selection: &Setting = list
                .iter()
                .filter(|p| godmode || p.user_modifiable)
                .nth(cursor_pos as usize)
                .unwrap();
            draw_setting_selection(
                current_selection,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                false,
                cfg,
            );
            // moving cursor
            if keyboard_scan.key_down(key::UP) {
                if cursor_pos > 0 {
                    cursor_pos -= 1;
                } else {
                    cursor_pos =
                        (list.iter().filter(|p| godmode || p.user_modifiable).count() as u16) - 1;
                }
                last_action_key = key::UP;
            } else if keyboard_scan.key_down(key::DOWN) {
                if cursor_pos
                    < list.iter().filter(|p| godmode || p.user_modifiable).count() as u16 - 1
                {
                    cursor_pos += 1;
                } else {
                    cursor_pos = 0;
                }
                last_action_key = key::DOWN;
            }

            let new_selection: &Setting = list
                .iter()
                .filter(|p| godmode || p.user_modifiable)
                .nth(cursor_pos as usize)
                .unwrap();
            draw_setting_selection(
                new_selection,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                true,
                cfg,
            );
            last_action = timing::millis();
        } else if (keyboard_scan.key_down(key::OK)
            || keyboard_scan.key_down(key::RIGHT)
            || keyboard_scan.key_down(key::LEFT))
            && (timing::millis() >= (last_action + REPETITION_SPEED as u64))
        {
            display::wait_for_vblank();
            push_rect_uniform(
                // remove last text
                Rect {
                    x: XPOS_VALUES,
                    y: first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                    width: SCREEN_WIDTH - XPOS_VALUES,
                    height: LARGE_CHAR_HEIGHT + 2, // I got some problems with characters going under the line (like g)
                },
                cfg.bckgrd,
            );

            let selection: &mut Setting = list
                .iter_mut()
                .filter(|p| godmode || p.user_modifiable)
                .nth(cursor_pos as usize)
                .unwrap();

            if !selection.fixed_values {
                if keyboard_scan.key_down(key::OK) {
                    let v = numberinput(
                        selection.values[0],
                        selection.values[1],
                        selection.values[2],
                        true,
                        Point {
                            x: XPOS_VALUES,
                            y: first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                        },
                        cfg,
                    );
                    selection.set_value(v);
                    last_action_key = key::OK;
                } else if keyboard_scan.key_down(key::RIGHT) {
                    // increment one step
                    if selection.values[0] < selection.values[2] {
                        selection.set_value(selection.values[0] + 1);
                    }
                    last_action_key = key::RIGHT;
                } else if keyboard_scan.key_down(key::LEFT) {
                    if selection.values[0] > selection.values[1] {
                        selection.set_value(selection.values[0] - 1);
                    }
                    last_action_key = key::LEFT;
                }
            } else if keyboard_scan.key_down(key::OK) {
                selection.increment_value();
                last_action_key = key::OK
            } else if keyboard_scan.key_down(key::RIGHT) {
                selection.increment_value();
                last_action_key = key::RIGHT;
            } else {
                selection.decrement_value();
                last_action_key = key::LEFT
            }
            draw_setting_selection(
                selection,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                true,
                cfg,
            );
            last_action = timing::millis();
        } else if !keyboard_scan.key_down(last_action_key) {
            // if we let go of the key, then we can use a key just after (even the same one)
            last_action = 0;
        }
    }
}

/// Draws the line corresponding to the given setting value
fn draw_setting_selection(set: &Setting, ypos: u16, selected: bool, cfg: &ColorConfig) {
    let text = if set.fixed_values {
        set.get_text()
    } else {
        &string_from_u32(set.get_setting_value())
    };
    let large: bool = get_string_pixel_size(text, true) < SCREEN_WIDTH - XPOS_VALUES;
    draw_string_cfg(
        text,
        Point::new(
            XPOS_VALUES,
            ypos + if !large {
                (LARGE_CHAR_HEIGHT - SMALL_CHAR_HEIGHT) / 2
            } else {
                0
            },
        ),
        large,
        cfg,
        selected,
    );
    push_rect_uniform(
        Rect {
            x: XPOS_VALUES - 15,
            y: ypos + LARGE_CHAR_HEIGHT / 2,
            width: 10,
            height: 2,
        },
        if selected { cfg.alt } else { cfg.bckgrd },
    );
}
