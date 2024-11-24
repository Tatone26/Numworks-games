use heapless::Vec;

use crate::{
    eadk::{
        display::{self, push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, timing, Point, Rect,
    },
    graphical::{draw_centered_string, draw_string_cfg, fill_screen, ColorConfig},
    utils::{get_string_pixel_size, wait_for_no_keydown, LARGE_CHAR_HEIGHT, SMALL_CHAR_HEIGHT},
};

use super::{REPETITION_SPEED, SPACE_BETWEEN_LINES};

// The maximum number of different values an option can take.
pub const MAX_SETTINGS_VALUES: usize = 10;

/// Where in the x coordinate will the names of the [option][Setting] be placed
const XPOS_NAMES: u16 = 30;
/// Where in the x coordinate will the values ([option][Setting].value.1) be placed
const XPOS_VALUES: u16 = 170;

/// Used to represent the type of the option, instead of plain datatypes as Bool or Int
/// Without it, impossible to use multiple data types in options
#[derive(Debug)]
pub enum SettingType {
    Bool(bool),
    Int(u16),
    Double(f32),
}
/// An Option of T type, with COUNT possible values
#[derive(Debug)]
pub struct Setting {
    pub name: &'static str,
    pub value: usize,
    pub possible_values: Vec<(SettingType, &'static str), MAX_SETTINGS_VALUES>,
}

impl Setting {
    /// Set the value to the next one, 0 if needed
    fn increment_value(&mut self) {
        if self.value == self.possible_values.len() - 1 {
            self.value = 0
        } else {
            self.value += 1
        }
    }

    fn decrement_value(&mut self) {
        if self.value == 0 {
            self.value = self.possible_values.len() - 1
        } else {
            self.value -= 1
        }
    }

    #[inline(always)]
    /// Returns the "value" the option is currently set to
    fn get_value(&self) -> &(SettingType, &'static str) {
        &self.possible_values[self.value]
    }

    #[inline(always)]
    /// Returns the true value contained in the option
    pub fn get_param_value<T: FromValue>(&self) -> T {
        T::from_value(&self.possible_values[self.value].0)
    }
}

pub trait FromValue {
    fn from_value(value: &SettingType) -> Self;
}

impl FromValue for bool {
    fn from_value(value: &SettingType) -> Self {
        match value {
            SettingType::Bool(b) => *b,
            _ => panic!(),
        }
    }
}
impl FromValue for u16 {
    fn from_value(value: &SettingType) -> Self {
        match value {
            SettingType::Int(b) => *b,
            _ => panic!(),
        }
    }
}
impl FromValue for f32 {
    fn from_value(value: &SettingType) -> Self {
        match value {
            SettingType::Double(b) => *b,
            _ => panic!(),
        }
    }
}

/// Create a fully fonctional settings menu, which changes directly the [options][Setting] values. (no settings return)
/// For now, only allows a limited number of settings because making multiple pages is too complicated and I don't have time for that
pub(crate) fn settings(list: &mut [&mut Setting], cfg: &ColorConfig) {
    let items_number: u16 = list.len() as u16;
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
        "OPTIONS\0",
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
    for item in list.iter().enumerate() {
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
        draw_setting_selection(x.get_value().1, y_pos, i == 0, cfg)
    }
    setting_selection(list, cfg, first_y);
}

/// Takes care of all the difficult stuff, like moving the cursor and modifying the text.
///
/// Similar to [super::selection] but redone to account for more things (and not account for horizontal versions)
fn setting_selection(list: &mut [&mut Setting], cfg: &ColorConfig, first_y: u16) {
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

            let current_selection: &Setting = list[cursor_pos as usize];
            draw_setting_selection(
                current_selection.get_value().1,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                false,
                cfg,
            );
            // moving cursor
            if keyboard_scan.key_down(key::UP) {
                if cursor_pos > 0 {
                    cursor_pos -= 1;
                } else {
                    cursor_pos = (list.len() as u16) - 1;
                }
                last_action_key = key::UP;
            } else if keyboard_scan.key_down(key::DOWN) {
                if cursor_pos < list.len() as u16 - 1 {
                    cursor_pos += 1;
                } else {
                    cursor_pos = 0;
                }
                last_action_key = key::DOWN;
            }

            let new_selection: &Setting = list[cursor_pos as usize];
            draw_setting_selection(
                new_selection.get_value().1,
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

            let selection: &mut Setting = list[cursor_pos as usize];

            if keyboard_scan.key_down(key::OK) || keyboard_scan.key_down(key::RIGHT) {
                selection.increment_value();
                last_action_key = if keyboard_scan.key_down(key::OK) {
                    key::OK
                } else {
                    key::RIGHT
                };
            } else {
                selection.decrement_value();
                last_action_key = key::LEFT
            }
            draw_setting_selection(
                selection.get_value().1,
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
/// Like draw_selection_string but simpler
fn draw_setting_selection(text: &str, ypos: u16, selected: bool, cfg: &ColorConfig) {
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
