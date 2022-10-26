use crate::eadk::backlight::{brightness, set_brightness};
use crate::eadk::display::{self, draw_string};
use crate::eadk::{random, timing, Color, Point, Rect};

pub const LARGE_CHAR_WIDTH: u16 = 10;
pub const SMALL_CHAR_WIDTH: u16 = 7;

pub const LARGE_CHAR_HEIGHT: u16 = 14;
pub const SMALL_CHAR_HEIGHT: u16 = 8;

/// Fills the screen with the given [Color]
pub fn fill_screen(color: Color) {
    display::push_rect_uniform(
        Rect {
            x: Point::ZERO.x,
            y: Point::ZERO.y,
            width: display::SCREEN_WIDTH,
            height: display::SCREEN_HEIGHT,
        },
        color,
    );
}

/// Returns the left x coordinate that centers the given string
pub fn get_centered_text_left_coordo(string: &str, large: bool) -> u16 {
    let size: u16 = string.chars().count() as u16;
    match (display::SCREEN_WIDTH / 2).checked_sub(
        (size / 2)
            * (if large {
                LARGE_CHAR_WIDTH
            } else {
                SMALL_CHAR_WIDTH
            }),
    ) {
        x @ Some(1_u16..=u16::MAX) => return x.unwrap(),
        None | Some(0) => return 0u16,
    }
}

/// Draws the given string centered in the x coordinate
pub fn draw_centered_string(
    string: &str,
    posy: u16,
    large: bool,
    text_color: Color,
    background_color: Color,
) {
    draw_string(
        string,
        Point::new(get_centered_text_left_coordo(string, large), posy),
        large,
        text_color,
        background_color,
    );
}

/// Make the screen fade to 0 [brightness], then back to original brightness
pub fn fading(dur: u32) {
    let start_brightness: u8;
    if brightness() <= 0 {
        start_brightness = 16
    } else {
        start_brightness = brightness();
    }
    let mut bs = start_brightness;
    while bs != 0 {
        set_brightness(bs);
        timing::msleep(dur / start_brightness as u32);
        bs -= 1;
    }
    fill_screen(Color::BLACK);
    set_brightness(start_brightness);
}

/// Gives a [random] integer between a and b (included)
pub fn randint(a: u32, b: u32) -> u32 {
    return random() % (b - a) + a;
}
