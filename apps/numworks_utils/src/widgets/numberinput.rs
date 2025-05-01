use crate::{
    eadk::{display::push_rect_uniform, key, keyboard, Point, Rect},
    graphical::{draw_string_cfg, ColorConfig},
    utils::{
        string_from_u32, wait_for_no_keydown, LARGE_CHAR_HEIGHT, LARGE_CHAR_WIDTH,
        SMALL_CHAR_HEIGHT, SMALL_CHAR_WIDTH,
    },
};

/// Allows entering a number between min and max. Will block everything as long as it is not confirmed.
/// Will print the input at the given position, and use the ColorConfig to choose the colors.
pub fn numberinput(
    default: u32,
    min: u32,
    max: u32,
    large: bool,
    pos: Point,
    c: &ColorConfig,
) -> u32 {
    // Max digits : 10
    push_rect_uniform(
        Rect {
            x: pos.x,
            y: pos.y,
            width: 10 * {
                if large {
                    LARGE_CHAR_WIDTH
                } else {
                    SMALL_CHAR_WIDTH
                }
            } + 6,
            height: {
                if large {
                    LARGE_CHAR_HEIGHT
                } else {
                    SMALL_CHAR_HEIGHT
                }
            } + 6,
        },
        c.alt,
    );
    push_rect_uniform(
        Rect {
            x: pos.x + 3,
            y: pos.y + 3,
            width: 10 * {
                if large {
                    LARGE_CHAR_WIDTH
                } else {
                    SMALL_CHAR_WIDTH
                }
            },
            height: {
                if large {
                    LARGE_CHAR_HEIGHT
                } else {
                    SMALL_CHAR_HEIGHT
                }
            },
        },
        c.bckgrd,
    );
    let mut current_value = default;
    let mut current_string = string_from_u32(default);
    draw_string_cfg(
        &current_string,
        Point {
            x: pos.x + 3,
            y: pos.y + 3,
        },
        large,
        c,
        false,
    );
    wait_for_no_keydown();
    loop {
        let scan = keyboard::scan();
        if scan.key_down(key::OK) {
            break;
        }
        let mut temp = current_value;

        fn try_push_digit(temp: u32, digit: u32, max: u32) -> u32 {
            temp.checked_mul(10)
                .and_then(|v| v.checked_add(digit))
                .filter(|&v| v <= max)
                .unwrap_or(temp)
        }

        if scan.key_down(key::ZERO) {
            temp = try_push_digit(temp, 0, max);
        } else if scan.key_down(key::ONE) {
            temp = try_push_digit(temp, 1, max);
        } else if scan.key_down(key::TWO) {
            temp = try_push_digit(temp, 2, max);
        } else if scan.key_down(key::THREE) {
            temp = try_push_digit(temp, 3, max);
        } else if scan.key_down(key::FOUR) {
            temp = try_push_digit(temp, 4, max);
        } else if scan.key_down(key::FIVE) {
            temp = try_push_digit(temp, 5, max);
        } else if scan.key_down(key::SIX) {
            temp = try_push_digit(temp, 6, max);
        } else if scan.key_down(key::SEVEN) {
            temp = try_push_digit(temp, 7, max);
        } else if scan.key_down(key::EIGHT) {
            temp = try_push_digit(temp, 8, max);
        } else if scan.key_down(key::NINE) {
            temp = try_push_digit(temp, 9, max);
        } else if scan.key_down(key::UP) {
            temp = temp.checked_add(1).filter(|&v| v <= max).unwrap_or(temp)
        } else if scan.key_down(key::DOWN) {
            temp = temp.checked_sub(1).filter(|&v| v >= min).unwrap_or(temp)
        } else if scan.key_down(key::BACKSPACE) {
            temp /= 10;
        }

        if temp != current_value {
            current_value = temp;
            current_string = string_from_u32(current_value);
            push_rect_uniform(
                Rect {
                    x: pos.x + 3,
                    y: pos.y + 3,
                    width: 10 * {
                        if large {
                            LARGE_CHAR_WIDTH
                        } else {
                            SMALL_CHAR_WIDTH
                        }
                    },
                    height: {
                        if large {
                            LARGE_CHAR_HEIGHT
                        } else {
                            SMALL_CHAR_HEIGHT
                        }
                    },
                },
                c.bckgrd,
            );
            draw_string_cfg(
                &current_string,
                Point {
                    x: pos.x + 3,
                    y: pos.y + 3,
                },
                large,
                c,
                false,
            );
        }

        if scan.any_down() {
            wait_for_no_keydown();
        }
    }
    push_rect_uniform(
        Rect {
            x: pos.x,
            y: pos.y,
            width: 10 * {
                if large {
                    LARGE_CHAR_WIDTH
                } else {
                    SMALL_CHAR_WIDTH
                }
            } + 6,
            height: {
                if large {
                    LARGE_CHAR_HEIGHT
                } else {
                    SMALL_CHAR_HEIGHT
                }
            } + 6,
        },
        c.bckgrd,
    );
    current_value
}
