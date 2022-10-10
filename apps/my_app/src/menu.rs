use core::u8;

use crate::eadk::display::{push_rect_uniform, draw_string};
use crate::eadk::{display, key, keyboard, timing, Color, Point, Rect};

use crate::utils::{draw_centered_string, fill_screen, get_centered_left_coordo, LARGE_CHAR_HEIGHT};

#[derive(Debug)]
enum CursorPos {
    START,
    OPTIONS,
    EXIT,
}

#[derive(Debug)]
pub struct MyOption<T : PartialEq, const COUNT : usize> {
    name : &'static str,
    value : T, 
    possible_values : [T; COUNT],
    possible_values_str : [&'static str; COUNT]
}

impl<T : PartialEq, const COUNT : usize> MyOption<T, COUNT> {
    pub fn get_next_value(&self) -> (&T, &str) {
        for item in self.possible_values.iter().enumerate(){
            let (i, x) : (usize, &T) = item;
            if x == &self.value{
                if i < self.possible_values.len() - 1{
                    return (&self.possible_values[i + 1], self.possible_values_str[i + 1]);
                }else{
                    return (&self.possible_values[0], self.possible_values_str[0]);
                }
            }
        }
        return (&self.value, self.possible_values_str[0])
    }
}

const START_POS: u16 = 120;
const OPTIONS_POS: u16 = 160;
const EXIT_POS: u16 = 200;

const START_TXT: &str = "Start\0";
const OPTIONS_TXT: &str = "Options\0";
const EXIT_TXT: &str = "Exit\0";

pub fn menu(title: &str, text_color: Color, background_color: Color, selection_color: Color) -> u8 {
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
                CursorPos::OPTIONS => _ = options(text_color, background_color, selection_color),
                CursorPos::EXIT => return 0
            }
        } else if keyboard_state.key_down(key::DOWN) | keyboard_state.key_down(key::UP) {
            match &cursor_pos {
                CursorPos::START => push_rect_uniform(
                    Rect {
                        x: Point::ZERO.x,
                        y: START_POS,
                        width: display::SCREEN_WIDTH,
                        height: LARGE_CHAR_HEIGHT,
                    },
                    background_color,
                ),
                CursorPos::OPTIONS => push_rect_uniform(
                    Rect {
                        x: Point::ZERO.x,
                        y: OPTIONS_POS,
                        width: display::SCREEN_WIDTH,
                        height: LARGE_CHAR_HEIGHT,
                    },
                    background_color,
                ),
                CursorPos::EXIT => push_rect_uniform(
                    Rect {
                        x: Point::ZERO.x,
                        y: EXIT_POS,
                        width: display::SCREEN_WIDTH,
                        height: LARGE_CHAR_HEIGHT,
                    },
                    background_color,
                ),
            }
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
        }
    }
}

fn draw_selection_string(cursor_pos: &CursorPos, text_color: Color, background_color: Color, selected : bool) {
    match cursor_pos {
        CursorPos::START => {
            draw_centered_string(START_TXT, START_POS, true, text_color, background_color);
            push_rect_uniform(
                Rect {
                    x: get_centered_left_coordo(START_TXT, true) - 15,
                    y: START_POS + LARGE_CHAR_HEIGHT / 2,
                    width: 10,
                    height: 2,
                },
                if selected {text_color} else {background_color},
            );
        }
        CursorPos::OPTIONS => {
            draw_centered_string(OPTIONS_TXT, OPTIONS_POS, true, text_color, background_color);
            push_rect_uniform(
                Rect {
                    x: get_centered_left_coordo(OPTIONS_TXT, true) - 15,
                    y: OPTIONS_POS + LARGE_CHAR_HEIGHT / 2,
                    width: 10,
                    height: 2,
                },
                if selected {text_color} else {background_color},
            );
        }
        CursorPos::EXIT => {
            draw_centered_string(EXIT_TXT, EXIT_POS, true, text_color, background_color);
            push_rect_uniform(
                Rect {
                    x: get_centered_left_coordo(EXIT_TXT, true) - 15,
                    y: EXIT_POS + LARGE_CHAR_HEIGHT / 2,
                    width: 10,
                    height: 2,
                },
                if selected {text_color} else {background_color},
            );
        }
    }

}

fn options(text_color: Color, background_color: Color, selection_color: Color) -> u8 {
    let myoption1 = MyOption::<bool, 2>{name : "TestOption\0", value:true, possible_values:[true, false], possible_values_str:["Vrai\0", "Faux\0"]};
    draw_centered_string(myoption1.name, START_POS, true, text_color, background_color);
    draw_centered_string(myoption1.get_next_value().1, EXIT_POS, true, text_color, background_color);
    return 0
}