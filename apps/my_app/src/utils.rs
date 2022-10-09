use crate::eadk::display::{self, draw_string};
use crate::eadk::{Color, Point, Rect};

pub const LARGE_CHAR_SIZE: u16 = 10;
pub const SMALL_CHAR_SIZE: u16 = 7;

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

pub fn draw_centered_string(
    string: &str,
    posy: u16,
    large: bool,
    text_color: Color,
    bgd_color: Color,
) {
    let size: u16 = string.chars().count().try_into().unwrap();
    match (display::SCREEN_WIDTH / 2).checked_sub(
        (size / 2)
            * (if large {
                LARGE_CHAR_SIZE
            } else {
                SMALL_CHAR_SIZE
            }),
    ) {
        Some(1_u16..=u16::MAX) => {
            draw_string(
                string,
                Point::new(
                    (display::SCREEN_WIDTH) / 2
                        - ((size / 2)
                            * (if large {
                                LARGE_CHAR_SIZE
                            } else {
                                SMALL_CHAR_SIZE
                            })),
                    posy,
                ),
                large,
                text_color,
                bgd_color,
            );
        }
        None | Some(0) => {
            draw_string(string, Point::new(0, posy), large, text_color, bgd_color);
        }
    };
}
