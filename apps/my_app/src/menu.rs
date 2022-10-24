use crate::eadk::display::{draw_string, push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::eadk::{display, key, keyboard, timing, Color, Point, Rect};
use crate::BOOL_OPTIONS_NUMBER;

use crate::utils::{
    draw_centered_string, fill_screen, get_centered_text_left_coordo, LARGE_CHAR_HEIGHT,
};

#[derive(Debug)]
enum CursorPos {
    START,
    OPTIONS,
    EXIT,
}

#[derive(Debug, Copy, Clone)]
pub struct MyOption<T: PartialEq + Copy, const COUNT: usize> {
    pub name: &'static str,
    pub value: (T, &'static str),
    pub possible_values: [T; COUNT],
    pub possible_values_str: [&'static str; COUNT],
}

impl<T: PartialEq + Copy, const COUNT: usize> MyOption<T, COUNT> {
    pub fn get_next_value(&self) -> (T, &'static str) {
        for item in self.possible_values.iter().enumerate() {
            let (i, x) = item;
            if x == &self.value.0 {
                if i < self.possible_values.len() - 1 {
                    return (self.possible_values[i + 1], self.possible_values_str[i + 1]);
                } else {
                    return (self.possible_values[0], self.possible_values_str[0]);
                }
            }
        }
        return (self.value.0, self.possible_values_str[0]);
    }
}

const START_POS: u16 = 120;
const OPTIONS_POS: u16 = 160;
const EXIT_POS: u16 = 200;

const START_TXT: &str = "Start\0";
const OPTIONS_TXT: &str = "Options\0";
const EXIT_TXT: &str = "Exit\0";

pub fn menu(
    title: &str,
    opt: &mut [&mut MyOption<bool, 2>; 2],
    text_color: Color,
    background_color: Color,
    selection_color: Color,
) -> u8 {
    fill_screen(background_color);
    draw_centered_string(title, 20u16, true, text_color, background_color);
    draw_selection_string(&CursorPos::START, selection_color, background_color, true);
    draw_selection_string(&CursorPos::OPTIONS, text_color, background_color, false);
    draw_selection_string(&CursorPos::EXIT, text_color, background_color, false);
    let mut cursor_pos: CursorPos = CursorPos::START;
    timing::msleep(300);
    loop {
        let keyboard_state = keyboard::scan();
        if keyboard_state.key_down(key::OK) {
            match &cursor_pos {
                CursorPos::START => return 1,
                CursorPos::OPTIONS => {
                    _ = options(opt, text_color, background_color, selection_color);
                    return menu(title, opt, text_color, background_color, selection_color);
                }
                CursorPos::EXIT => return 0,
            }
        } else if keyboard_state.key_down(key::DOWN) | keyboard_state.key_down(key::UP) {
            draw_selection_string(&cursor_pos, text_color, background_color, false);
            if keyboard_state.key_down(key::DOWN) {
                match &cursor_pos {
                    CursorPos::START => cursor_pos = CursorPos::OPTIONS,
                    CursorPos::OPTIONS => cursor_pos = CursorPos::EXIT,
                    CursorPos::EXIT => cursor_pos = CursorPos::START,
                }
            } else if keyboard_state.key_down(key::UP) {
                match &cursor_pos {
                    CursorPos::START => cursor_pos = CursorPos::EXIT,
                    CursorPos::OPTIONS => cursor_pos = CursorPos::START,
                    CursorPos::EXIT => cursor_pos = CursorPos::OPTIONS,
                }
            }
            draw_selection_string(&cursor_pos, selection_color, background_color, true);
            display::wait_for_vblank();
            timing::msleep(200);
        } else if keyboard_state.key_down(key::BACK) {
            return 0;
        }
    }
}

fn draw_selection_string(
    cursor_pos: &CursorPos,
    text_color: Color,
    background_color: Color,
    selected: bool,
) {
    match cursor_pos {
        CursorPos::START => {
            draw_centered_string(START_TXT, START_POS, true, text_color, background_color);
            push_rect_uniform(
                Rect {
                    x: get_centered_text_left_coordo(START_TXT, true) - 15,
                    y: START_POS + LARGE_CHAR_HEIGHT / 2,
                    width: 10,
                    height: 2,
                },
                if selected {
                    text_color
                } else {
                    background_color
                },
            );
        }
        CursorPos::OPTIONS => {
            draw_centered_string(OPTIONS_TXT, OPTIONS_POS, true, text_color, background_color);
            push_rect_uniform(
                Rect {
                    x: get_centered_text_left_coordo(OPTIONS_TXT, true) - 15,
                    y: OPTIONS_POS + LARGE_CHAR_HEIGHT / 2,
                    width: 10,
                    height: 2,
                },
                if selected {
                    text_color
                } else {
                    background_color
                },
            );
        }
        CursorPos::EXIT => {
            draw_centered_string(EXIT_TXT, EXIT_POS, true, text_color, background_color);
            push_rect_uniform(
                Rect {
                    x: get_centered_text_left_coordo(EXIT_TXT, true) - 15,
                    y: EXIT_POS + LARGE_CHAR_HEIGHT / 2,
                    width: 10,
                    height: 2,
                },
                if selected {
                    text_color
                } else {
                    background_color
                },
            );
        }
    }
}

const SPACE_BETWEEN_LINES: u16 = 20;
const XPOS_NAMES: u16 = 30;
const XPOS_VALUES: u16 = 170;

fn options(
    list: &mut [&mut MyOption<bool, 2>; BOOL_OPTIONS_NUMBER],
    text_color: Color,
    background_color: Color,
    selection_color: Color,
) -> u8 {
    fill_screen(background_color);
    draw_centered_string("OPTIONS\0", 20u16, true, text_color, background_color);
    // Only taking care of boolean options for now.
    let first_y: u16;
    let items_number: u16 = list.iter().count() as u16;
    match (SCREEN_HEIGHT - (SCREEN_HEIGHT - LARGE_CHAR_HEIGHT) / 2)
        .checked_sub((LARGE_CHAR_HEIGHT / 2) * items_number)
    {
        None | Some(0) => first_y = 0,
        Some(1u16..=u16::MAX) => {
            first_y = (SCREEN_HEIGHT - (SCREEN_HEIGHT - LARGE_CHAR_HEIGHT) / 2)
                - ((LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) / 2) * items_number
        }
    }
    for item in list.iter().enumerate() {
        let (i, x) = item;
        let pos: u16 = i as u16;
        display::draw_string(
            x.name,
            Point::new(
                XPOS_NAMES,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * pos,
            ),
            x.name.len() < 12,
            text_color,
            background_color,
        );
        draw_options_selection(
            x.value.1,
            first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * pos,
            if pos == 0 { true } else { false },
            selection_color,
            background_color,
            text_color,
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
                current_selection.value.1,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                false,
                selection_color,
                background_color,
                text_color,
            );
            if keyboard_scan.key_down(key::DOWN) {
                if cursor_pos > 0 {
                    cursor_pos -= 1;
                } else {
                    cursor_pos = (list.len() as u16) - 1;
                }
            } else if keyboard_scan.key_down(key::UP) {
                if cursor_pos < list.len() as u16 - 1 {
                    cursor_pos += 1;
                } else {
                    cursor_pos = 0;
                }
            }
            let new_selection: &MyOption<bool, 2> = &list[cursor_pos as usize];
            draw_options_selection(
                new_selection.value.1,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                true,
                selection_color,
                background_color,
                text_color,
            );
            display::wait_for_vblank();
            timing::msleep(200);
        } else if keyboard_scan.key_down(key::OK) {
            let mut selection: &mut MyOption<bool, 2> = list[cursor_pos as usize];
            selection.value = selection.get_next_value();
            draw_options_selection(
                selection.value.1,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                true,
                selection_color,
                background_color,
                text_color,
            );
            display::wait_for_vblank();
            timing::msleep(200);
        }
    }
    return 0;
}

fn draw_options_selection(
    text: &str,
    ypos: u16,
    selected: bool,
    selection_color: Color,
    background_color: Color,
    text_color: Color,
) {
    draw_string(
        text,
        Point::new(XPOS_VALUES, ypos),
        true,
        if selected {
            selection_color
        } else {
            text_color
        },
        background_color,
    );
    push_rect_uniform(
        Rect {
            x: XPOS_VALUES - 15,
            y: ypos + LARGE_CHAR_HEIGHT / 2,
            width: 10,
            height: 2,
        },
        if selected {
            selection_color
        } else {
            background_color
        },
    );
}

const RESUME_TXT: &str = "Resume\0";
const PAUSE_RECT_SIZE: u16 = 20; // La marge sur le côté

pub fn pause_menu(text_color: Color, background_color: Color, selection_color: Color) -> u16 {
    let mut cursor_pos: CursorPos = CursorPos::START;
    let rect_x: u16 = get_centered_text_left_coordo(RESUME_TXT, true);
    display::push_rect_uniform(
        Rect {
            x: rect_x - PAUSE_RECT_SIZE,
            y: (SCREEN_HEIGHT / 2) - LARGE_CHAR_HEIGHT - SPACE_BETWEEN_LINES - PAUSE_RECT_SIZE,
            width: SCREEN_WIDTH - (rect_x - PAUSE_RECT_SIZE) * 2,
            height: (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES + PAUSE_RECT_SIZE) * 2,
        },
        background_color,
    );
    draw_pause_selection_string(&cursor_pos, text_color, background_color, true, selection_color);
    draw_pause_selection_string(&CursorPos::EXIT, text_color, background_color, false, selection_color);
    display::wait_for_vblank();
    timing::msleep(200);
    loop {
        let keyboard_state = keyboard::scan();
        if keyboard_state.key_down(key::DOWN) | keyboard_state.key_down(key::UP) {
            draw_pause_selection_string(&cursor_pos, text_color, background_color, false, selection_color);
            if keyboard_state.key_down(key::DOWN) {
                match &cursor_pos {
                    CursorPos::START => cursor_pos = CursorPos::EXIT,
                    CursorPos::EXIT => cursor_pos = CursorPos::START,
                    CursorPos::OPTIONS => cursor_pos = CursorPos::OPTIONS,
                }
            } else if keyboard_state.key_down(key::UP) {
                match &cursor_pos {
                    CursorPos::START => cursor_pos = CursorPos::EXIT,
                    CursorPos::OPTIONS => cursor_pos = CursorPos::OPTIONS,
                    CursorPos::EXIT => cursor_pos = CursorPos::START,
                }
            }
            draw_pause_selection_string(&cursor_pos, text_color, background_color, true, selection_color);
            display::wait_for_vblank();
            timing::msleep(200);
        } else if keyboard_state.key_down(key::OK) {
            match &cursor_pos {
                CursorPos::START => return 1,
                CursorPos::OPTIONS => cursor_pos = CursorPos::OPTIONS,
                CursorPos::EXIT => return 0,
            }
        } else if keyboard_state.key_down(key::BACK) {
            return 1;
        }
    }
}

fn draw_pause_selection_string(
    cursor_pos: &CursorPos,
    text_color: Color,
    background_color: Color,
    selected: bool,
    selection_color: Color
) {
    match cursor_pos {
        CursorPos::START => {
            draw_centered_string(
                RESUME_TXT,
                (SCREEN_HEIGHT / 2) - LARGE_CHAR_HEIGHT - SPACE_BETWEEN_LINES,
                true,
                if selected{ selection_color } else {text_color},
                background_color,
            );
            push_rect_uniform(
                Rect {
                    x: get_centered_text_left_coordo(RESUME_TXT, true) - 15,
                    y: (SCREEN_HEIGHT / 2) - LARGE_CHAR_HEIGHT/2 - SPACE_BETWEEN_LINES,
                    width: 10,
                    height: 2,
                },
                if selected {
                    selection_color
                } else {
                    background_color
                },
            );
        }
        CursorPos::OPTIONS => {}
        CursorPos::EXIT => {
            draw_centered_string(
                EXIT_TXT,
                (SCREEN_HEIGHT / 2) + SPACE_BETWEEN_LINES,
                true,
                if selected{ selection_color } else {text_color},
                background_color,
            );
            push_rect_uniform(
                Rect {
                    x: get_centered_text_left_coordo(EXIT_TXT, true) - 15,
                    y: (SCREEN_HEIGHT / 2) + SPACE_BETWEEN_LINES + LARGE_CHAR_HEIGHT/2,
                    width: 10,
                    height: 2,
                },
                if selected {
                    selection_color
                } else {
                    background_color
                },
            );
        }
    }
}
