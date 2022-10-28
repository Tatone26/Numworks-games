use crate::eadk::display::{draw_string, push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::eadk::{display, key, keyboard, timing, Point, Rect};
use crate::game::BOOL_OPTIONS_NUMBER;
use crate::utils::{
    draw_centered_string, fading, fill_screen, get_centered_text_left_coordo, ColorConfig,
    LARGE_CHAR_HEIGHT, SMALL_CHAR_HEIGHT, draw_string_cfg,
};

/// Used to symbolise the cursor position
#[derive(Debug)]
enum CursorPos {
    START,
    OPTIONS,
    EXIT,
}

/// An Option of T type, with COUNT possible values
#[derive(Debug)]
pub struct MyOption<T, const COUNT: usize> {
    pub name: &'static str,
    pub value: usize,
    pub possible_values: [(T, &'static str); COUNT],
}

impl<T, const COUNT: usize> MyOption<T, COUNT> {
    /// Set the value to the next one, 0 if needed
    pub fn increment_value(&mut self) {
        if self.value == COUNT - 1 {
            self.value = 0
        } else {
            self.value += 1
        }
    }
    /// Returns the value the option is currently set to
    pub fn get_value(&self) -> &(T, &'static str) {
        return &self.possible_values[self.value];
    }
}

/// Used to describe the menu texts, size, pos...*
/// This is THE thing to use to construct a beautiful menu !
/// It may not be really intuitive, but it is efficient.
pub struct MenuConfig {
    first_choice: &'static str,
    second_choice: &'static str,
    null_choice: &'static str,
    rect_margins: (u16, u16), // How much between text and sides
    dimensions: (u16, u16), // Width and Height
    offset: (u16, u16), // Offset of the entire menu
    back_key_return: u8, // The return value bound to the BACK key
}

/// Duration of fadings, in milliseconds
const FADING_TIME: u32 = 500;

/// Creates a fully fonctional start menu, with [Options][MyOption] as second choice
pub fn menu(
    title: &str,
    opt: &mut [&mut MyOption<bool, 2>; BOOL_OPTIONS_NUMBER],
    cfg: &ColorConfig,
) -> u8 {
    loop {
        fill_screen(cfg.bckgrd);
        draw_centered_string(title, 20, true, cfg, false);
        let action = selection_menu(
            cfg,
            &MenuConfig {
                first_choice: "Play\0",
                second_choice: "Options\0",
                null_choice: "Exit\0",
                rect_margins: (10, 10),
                dimensions: (SCREEN_WIDTH/2, SCREEN_HEIGHT / 2),
                offset: (0, SCREEN_HEIGHT / 5),
                back_key_return: 0
            },
        );
        if action == 2 {
            options(opt, cfg);
        } else {
            fading(FADING_TIME);
            return action;
        }
    }
}

/// Creates a fully fonctional pause menu
pub fn pause_menu(cfg: &ColorConfig, y_offset: u16) -> u8 {
    return selection_menu(
        cfg,
        &MenuConfig {
            first_choice: "Resume\0",
            second_choice: "Menu\0",
            null_choice: "Exit\0",
            rect_margins: (20, 10),
            dimensions: (
                SCREEN_WIDTH * 2 / 5,
                LARGE_CHAR_HEIGHT * 3 + SPACE_BETWEEN_LINES * 4,
            ),
            offset: (0, y_offset),
            back_key_return: 1
        },
    );
}

/// In milliseconds, the time between each action if we keep a key pushed.
const REPETITION_SPEED: u16 = 200;

/// The working part of any menu.
/// It currently works for only three choices (Start, Option and Exit for exemple)
/// Returns 1 for first_choice, 2 for second_choice and 0 for null_choice
fn selection_menu(color: &ColorConfig, config: &MenuConfig) -> u8 {
    let mut cursor_pos: CursorPos = CursorPos::START;
    display::push_rect_uniform(
        Rect {
            // Can get dirty here !! x and y need to be unsigned, so if < 0 then big boom
            x: SCREEN_WIDTH / 2 - config.dimensions.0 / 2 + config.offset.0,
            y: SCREEN_HEIGHT / 2 - config.dimensions.1 / 2 + config.offset.1,
            width: config.dimensions.0,
            height: config.dimensions.1,
        },
        color.bckgrd,
    );
    draw_selection_string(&cursor_pos, color, config, true);
    draw_selection_string(&CursorPos::OPTIONS, color, config, false);
    draw_selection_string(&CursorPos::EXIT, color, config, false);
    display::wait_for_vblank();
    timing::msleep(200);
    let mut last_action: u64 = timing::millis();
    let mut last_action_key: u32 = key::ALPHA;
    loop {
        let keyboard_state = keyboard::scan();
        if (keyboard_state.key_down(key::DOWN) | keyboard_state.key_down(key::UP)) & (timing::millis() >= (last_action + REPETITION_SPEED as u64)) {
            draw_selection_string(&cursor_pos, color, config, false);
            if keyboard_state.key_down(key::DOWN) {
                match &cursor_pos {
                    CursorPos::START => cursor_pos = CursorPos::OPTIONS,
                    CursorPos::EXIT => cursor_pos = CursorPos::START,
                    CursorPos::OPTIONS => cursor_pos = CursorPos::EXIT,
                }
                last_action_key = key::DOWN;
            } else if keyboard_state.key_down(key::UP) {
                match &cursor_pos {
                    CursorPos::START => cursor_pos = CursorPos::EXIT,
                    CursorPos::OPTIONS => cursor_pos = CursorPos::START,
                    CursorPos::EXIT => cursor_pos = CursorPos::OPTIONS,
                }
                last_action_key = key::UP;
            }
            draw_selection_string(&cursor_pos, color, config, true);
            display::wait_for_vblank();
            last_action = timing::millis();
        } else if keyboard_state.key_down(key::OK) {
            match &cursor_pos {
                CursorPos::START => return 1,
                CursorPos::OPTIONS => return 2,
                CursorPos::EXIT => return 0,
            }
        } else if keyboard_state.key_down(key::BACK) {
            return config.back_key_return;
        }else if !keyboard_state.key_down(last_action_key) { // if we let go of the key
            last_action = timing::millis() - REPETITION_SPEED as u64;
        }
    }
}

/// Draws the line corresponding to the [CursorPos]
/// The options are automatically regularly placed inside the defined menu
fn draw_selection_string(
    cursor_pos: &CursorPos,
    color: &ColorConfig,
    config: &MenuConfig,
    selected: bool,
) {
    let text: &str;
    let y_pos: u16;
    match cursor_pos {
        CursorPos::START => {
            text = config.first_choice;
            y_pos = SCREEN_HEIGHT / 2 + config.offset.1 + config.rect_margins.1
                - config.dimensions.1 / 2;
        }
        CursorPos::OPTIONS => {
            text = config.second_choice;
            y_pos = SCREEN_HEIGHT / 2 + config.offset.1 - LARGE_CHAR_HEIGHT / 2;
        }
        CursorPos::EXIT => {
            text = config.null_choice;
            y_pos = SCREEN_HEIGHT / 2 + config.offset.1 + config.dimensions.1 / 2
                - config.rect_margins.1
                - LARGE_CHAR_HEIGHT;
        }
    }
    let x_coordos = get_centered_text_left_coordo(text, true) + config.offset.0;
    draw_string_cfg(text, Point::new(x_coordos, y_pos), true, color, selected);
    push_rect_uniform(
        Rect {
            x: get_centered_text_left_coordo(text, true) - 15,
            y: y_pos + LARGE_CHAR_HEIGHT / 2,
            width: 10,
            height: 2,
        },
        if selected { color.alt } else { color.bckgrd },
    );
}

/// For the [options] menu, the space between each line.
const SPACE_BETWEEN_LINES: u16 = 14;
/// Where in the x coordinate will the names of the [option][MyOption] be placed
const XPOS_NAMES: u16 = 30;
/// Where in the x coordinate will the values ([option][MyOption].value.1) be placed
const XPOS_VALUES: u16 = 170;

/// Create a fully fonctional option menu, which changes directly the [options][MyOption] values. (no option return)
fn options(list: &mut [&mut MyOption<bool, 2>; BOOL_OPTIONS_NUMBER], cfg: &ColorConfig) -> u8 {
    fill_screen(cfg.bckgrd);
    draw_centered_string("OPTIONS\0", 20u16, true, cfg, false);
    // Only taking care of boolean options for now.
    let first_y: u16;
    let items_number: u16 = list.iter().count() as u16;
    match (SCREEN_HEIGHT - (SCREEN_HEIGHT - LARGE_CHAR_HEIGHT) / 2)
        .checked_sub(((LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) / 2) * items_number)
    {
        None | Some(0) => first_y = 0,
        x_ @ Some(1u16..=u16::MAX) => first_y = x_.unwrap(),
    }
    for item in list.iter().enumerate() {
        let (i, x) = item;
        let y_pos: u16 = first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * (i as u16);
        display::draw_string(
            x.name,
            Point::new(
                XPOS_NAMES,
                y_pos
                    + if x.name.len() > 12 {
                        (LARGE_CHAR_HEIGHT - SMALL_CHAR_HEIGHT) / 2
                    } else {
                        0
                    },
            ),
            x.name.len() < 12,
            cfg.text,
            cfg.bckgrd,
        );
        draw_options_selection(
            x.get_value().1,
            y_pos,
            if i == 0 { true } else { false },
            cfg,
        )
    }
    let mut cursor_pos: u16 = 0;
    display::wait_for_vblank();
    timing::msleep(200);
    loop {
        let keyboard_scan = keyboard::scan();
        if keyboard_scan.key_down(key::BACK) {
            break;
        } else if keyboard_scan.key_down(key::UP) | keyboard_scan.key_down(key::DOWN) {
            let current_selection: &MyOption<bool, 2> = &list[cursor_pos as usize];
            draw_options_selection(
                current_selection.get_value().1,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                false,
                cfg,
            );
            if keyboard_scan.key_down(key::UP) {
                if cursor_pos > 0 {
                    cursor_pos -= 1;
                } else {
                    cursor_pos = (list.len() as u16) - 1;
                }
            } else if keyboard_scan.key_down(key::DOWN) {
                if cursor_pos < list.len() as u16 - 1 {
                    cursor_pos += 1;
                } else {
                    cursor_pos = 0;
                }
            }
            let new_selection: &MyOption<bool, 2> = &list[cursor_pos as usize];
            draw_options_selection(
                new_selection.get_value().1,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                true,
                cfg,
            );
            display::wait_for_vblank();
            timing::msleep(200);
        } else if keyboard_scan.key_down(key::OK) {
            let selection: &mut MyOption<bool, 2> = list[cursor_pos as usize];
            selection.increment_value();
            draw_options_selection(
                selection.get_value().1,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                true,
                cfg,
            );
            display::wait_for_vblank();
            timing::msleep(200);
        }
    }
    return 0;
}

/// Draws the line corresponding to the given option value
fn draw_options_selection(text: &str, ypos: u16, selected: bool, cfg: &ColorConfig) {
    draw_string(
        text,
        Point::new(XPOS_VALUES, ypos),
        true,
        if selected { cfg.alt } else { cfg.text },
        cfg.bckgrd,
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
