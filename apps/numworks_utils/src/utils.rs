/// Here are some useful functions that should not have to be remade every time.
/// There are also some useful constants, that some would argue should be in the graphical file.
use heapless::String;

use crate::eadk::display::{self, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::eadk::{keyboard, random, Point};

/// Those constants have been found by hand and seem to not be perfect.

/// Width in pixels of a large character
pub const LARGE_CHAR_WIDTH: u16 = 9;
/// Width in pixels of a small character
pub const SMALL_CHAR_WIDTH: u16 = 6;
/// Height in pixels of a large character
pub const LARGE_CHAR_HEIGHT: u16 = 16;
/// Height in pixels of a small character
pub const SMALL_CHAR_HEIGHT: u16 = 10;

/// This center is the upper left corner of the 4 pixels that make the center
pub const CENTER: Point = Point::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2);

/// Returns the left x coordinate that centers the given string
pub fn get_centered_text_x_coordo(string: &str, large: bool) -> u16 {
    match (display::SCREEN_WIDTH).checked_sub(get_string_pixel_size(string, large)) {
        x @ Some(1_u16..=u16::MAX) => x.unwrap() / 2,
        None | Some(0) => 0,
    }
}

/// Returns the size IN PIXELS, on the screen, of the given string
pub fn get_string_pixel_size(string: &str, large: bool) -> u16 {
    return string.chars().count() as u16
        * (if large {
            LARGE_CHAR_WIDTH
        } else {
            SMALL_CHAR_WIDTH
        });
}

#[inline]
/// Gives a [random] integer between a and b (included)
pub fn randint(a: u32, b: u32) -> u32 {
    if a == b {
        0
    } else {
        random() % (b - a) + a
    }
}

/// Blocks the program until *nothing* is pressed.
pub fn wait_for_no_keydown() {
    loop {
        let keyboard_state = keyboard::scan();
        if !keyboard_state.any_down() {
            break;
        }
    }
}

#[inline]
/// Takes a decimal (from [include_bytes!]) and returns its value as a ascii number.
/// Used to read images parameters.
pub fn decimal_to_ascii_number(number: u8) -> Option<u8> {
    if !((48..=57).contains(&number)) {
        // not (48 <= number <= 57) == not an ascii number
        None
    } else {
        Some(number - 48)
    }
}

/// Returns the string corresponding to the decimal number given.
pub fn string_from_u16(number: u16) -> String<6> {
    let mut text: String<6> = String::new();
    if number == 0 {
        text.push('0').unwrap();
        text.truncate(1);
        text.push('\0').unwrap();
        return text;
    }
    let mut n = number;
    let mut cpt = 0;
    while n != 0 {
        text.push(((n % 10) as u8 + b'0') as char).unwrap();
        n /= 10;
        cpt += 1;
    }
    text.truncate(cpt);

    let mut second_text: String<6> = String::new();
    for i in (0..cpt).rev() {
        second_text.push(text.chars().nth(i).unwrap()).unwrap();
    }
    second_text.truncate(cpt);
    second_text.push('\0').unwrap();
    second_text
}

/// Returns the string corresponding to the decimal number given.
pub fn string_from_u32(number: u32) -> String<11> {
    let mut text: String<11> = String::new();
    if number == 0 {
        text.push('0').unwrap();
        text.truncate(1);
        text.push('\0').unwrap();
        return text;
    }
    let mut n = number;
    let mut cpt = 0;
    while n != 0 {
        text.push(((n % 10) as u8 + b'0') as char).unwrap();
        n /= 10;
        cpt += 1;
    }
    text.truncate(cpt);

    let mut second_text: String<11> = String::new();
    for i in (0..cpt).rev() {
        second_text.push(text.chars().nth(i).unwrap()).unwrap();
    }
    second_text.truncate(cpt);
    second_text.push('\0').unwrap();
    second_text
}
