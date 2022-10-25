use crate::eadk::backlight::{brightness, set_brightness};
use crate::eadk::display::{self, draw_string};
use crate::eadk::{timing, Color, Point, Rect};

pub const LARGE_CHAR_WIDTH: u16 = 10;
pub const SMALL_CHAR_WIDTH: u16 = 7;

pub const LARGE_CHAR_HEIGHT: u16 = 14;

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
        Some(1_u16..=u16::MAX) => {
            return (display::SCREEN_WIDTH / 2)
                - (size / 2)
                    * if large {
                        LARGE_CHAR_WIDTH
                    } else {
                        SMALL_CHAR_WIDTH
                    }
        }
        None | Some(0) => return 0u16,
    }
}

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

pub fn fading(dur: u32) { // conserves the original brightness. If brightness too low and duration too high, it's pretty bad.
    let mut start_brightness = brightness();
    if start_brightness <= 0 {
        start_brightness = 16
    };
    let mut bs = start_brightness;
    while bs != 0 {
        set_brightness(bs);
        timing::msleep(dur / start_brightness as u32);
        bs -= 1;
    }
    fill_screen(Color::BLACK);
    set_brightness(start_brightness);
}
