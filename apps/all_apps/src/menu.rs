use crate::eadk::display::{push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::eadk::{display, key, keyboard, timing, Point, Rect};
use crate::utils::{
    draw_centered_string, draw_string_cfg, fading, fill_screen, get_centered_text_x_coordo,
    get_string_pixel_size, wait_for_no_keydown, ColorConfig, CENTER, LARGE_CHAR_HEIGHT,
    SMALL_CHAR_HEIGHT,
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
    pub first_choice: &'static str,
    pub second_choice: &'static str,
    pub null_choice: &'static str,
    pub rect_margins: (u16, u16), // How much between text and sides
    pub dimensions: (u16, u16),   // Width and Height
    pub offset: (u16, u16),       // Offset of the entire menu
    pub back_key_return: u8,      // The return value bound to the BACK key
}

/// Duration of fadings, in milliseconds
const FADING_TIME: u32 = 500;

/// Creates a fully fonctional start menu, with [Options][MyOption] as second choice
pub fn menu<const N : usize>(
    title: &str,
    opt: &mut [&mut MyOption<bool, 2>; N],
    cfg: &ColorConfig,
    vis_addon: fn()
) -> u8 {
    loop {
        fill_screen(cfg.bckgrd);
        vis_addon();
        draw_centered_string(title, 20, true, cfg, false);
        let action = selection_menu(
            cfg,
            &MenuConfig {
                first_choice: "Play\0",
                second_choice: "Options\0",
                null_choice: "Exit\0",
                rect_margins: (10, 10),
                dimensions: (SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
                offset: (0, SCREEN_HEIGHT / 5),
                back_key_return: 0,
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

/// Creates a fully fonctional pause menu, mostly an exemple
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
            back_key_return: 1,
        },
    );
}

/// In milliseconds, the time between each action if we keep a key pushed.
const REPETITION_SPEED: u16 = 200;

/// The working part of any menu.
/// It currently works for only three choices (Start, Option and Exit for exemple)
/// Returns 1 for first_choice, 2 for second_choice and 0 for null_choice
pub fn selection_menu(color: &ColorConfig, config: &MenuConfig) -> u8 {
    let mut cursor_pos: CursorPos = CursorPos::START;
    display::push_rect_uniform(
        Rect {
            x: (SCREEN_WIDTH - config.dimensions.0) / 2 + config.offset.0,
            y: (SCREEN_HEIGHT - config.dimensions.1) / 2 + config.offset.1,
            width: config.dimensions.0,
            height: config.dimensions.1,
        },
        color.bckgrd,
    );
    draw_selection_string(&cursor_pos, color, config, true);
    draw_selection_string(&CursorPos::OPTIONS, color, config, false);
    draw_selection_string(&CursorPos::EXIT, color, config, false);
    display::wait_for_vblank();
    wait_for_no_keydown();
    let mut last_action: u64 = timing::millis();
    let mut last_action_key: u32 = key::ALPHA;
    loop {
        let keyboard_state = keyboard::scan();
        if (keyboard_state.key_down(key::DOWN) | keyboard_state.key_down(key::UP))
            & (timing::millis() >= (last_action + REPETITION_SPEED as u64))
        {
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
            loop {
                let keyboard_state_test = keyboard::scan();
                if !keyboard_state_test.key_down(key::OK) {
                    match &cursor_pos {
                        CursorPos::START => return 1,
                        CursorPos::OPTIONS => return 2,
                        CursorPos::EXIT => return 0,
                    }
                }
            }
        } else if keyboard_state.key_down(key::BACK) {
            return config.back_key_return;
        } else if !keyboard_state.key_down(last_action_key) {
            // if we let go of the key
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
            y_pos = CENTER.y + config.offset.1 + config.rect_margins.1 - config.dimensions.1 / 2;
        }
        CursorPos::OPTIONS => {
            text = config.second_choice;
            y_pos = CENTER.y + config.offset.1 - LARGE_CHAR_HEIGHT / 2;
        }
        CursorPos::EXIT => {
            text = config.null_choice;
            y_pos = CENTER.y + config.offset.1 + config.dimensions.1 / 2
                - config.rect_margins.1
                - LARGE_CHAR_HEIGHT;
        }
    }
    let x_coordos = get_centered_text_x_coordo(text, true) + config.offset.0;
    draw_string_cfg(text, Point::new(x_coordos, y_pos), true, color, selected);
    push_rect_uniform(
        Rect {
            x: get_centered_text_x_coordo(text, true) - 15,
            y: y_pos + LARGE_CHAR_HEIGHT / 2,
            width: 10,
            height: 2,
        },
        if selected { color.alt } else { color.bckgrd },
    );
}

/// For the [options] menu, the space between each line.
const SPACE_BETWEEN_LINES: u16 = LARGE_CHAR_HEIGHT;
/// Where in the x coordinate will the names of the [option][MyOption] be placed
const XPOS_NAMES: u16 = 30;
/// Where in the x coordinate will the values ([option][MyOption].value.1) be placed
const XPOS_VALUES: u16 = 170;

/// Create a fully fonctional option menu, which changes directly the [options][MyOption] values. (no option return)
fn options<const N : usize>(list: &mut [&mut MyOption<bool, 2>; N], cfg: &ColorConfig) -> u8 {
    fill_screen(cfg.bckgrd);
    draw_centered_string("OPTIONS\0", 20u16, true, cfg, false);
    // Only taking care of boolean options for now.
    let first_y: u16;
    let items_number: u16 = list.iter().count() as u16;
    match (SCREEN_HEIGHT + LARGE_CHAR_HEIGHT)
        .checked_sub((LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * items_number)
    {
        None | Some(0) => first_y = 0,
        x_ @ Some(1u16..=u16::MAX) => first_y = x_.unwrap() / 2,
    }
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
        draw_options_selection(
            x.get_value().1,
            y_pos,
            if i == 0 { true } else { false },
            cfg,
        )
    }
    let mut cursor_pos: u16 = 0;
    display::wait_for_vblank();
    wait_for_no_keydown();
    let mut last_action: u64 = timing::millis();
    let mut last_action_key: u32 = key::ALPHA;
    loop {
        let keyboard_scan = keyboard::scan();
        if keyboard_scan.key_down(key::BACK) {
            break;
        } else if (keyboard_scan.key_down(key::UP) | keyboard_scan.key_down(key::DOWN))
            & (timing::millis() >= (last_action + REPETITION_SPEED as u64))
        {
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
                last_action_key = key::UP;
            } else if keyboard_scan.key_down(key::DOWN) {
                if cursor_pos < list.len() as u16 - 1 {
                    cursor_pos += 1;
                } else {
                    cursor_pos = 0;
                }
                last_action_key = key::DOWN;
            }
            let new_selection: &MyOption<bool, 2> = &list[cursor_pos as usize];
            draw_options_selection(
                new_selection.get_value().1,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                true,
                cfg,
            );
            display::wait_for_vblank();
            last_action = timing::millis();
        } else if keyboard_scan.key_down(key::OK)
            & (timing::millis() >= (last_action + REPETITION_SPEED as u64))
        {
            let selection: &mut MyOption<bool, 2> = list[cursor_pos as usize];
            push_rect_uniform(
                Rect {
                    x: XPOS_VALUES,
                    y: first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                    width: SCREEN_WIDTH - XPOS_VALUES,
                    height: LARGE_CHAR_HEIGHT,
                },
                cfg.bckgrd,
            );
            selection.increment_value();
            draw_options_selection(
                selection.get_value().1,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                true,
                cfg,
            );
            display::wait_for_vblank();
            last_action_key = key::OK;
            last_action = timing::millis();
        } else if !keyboard_scan.key_down(last_action_key) {
            // if we let go of the key, then we can use a key just after (even the same one)
            last_action = timing::millis() - REPETITION_SPEED as u64;
        }
    }
    return 0;
}

/// Draws the line corresponding to the given option value
fn draw_options_selection(text: &str, ypos: u16, selected: bool, cfg: &ColorConfig) {
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
