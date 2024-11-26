pub mod tiling;

/// A structure that facilitates passing color configuration to functions.
pub struct ColorConfig {
    /// Color of the text
    pub text: Color,
    /// Color of the background
    pub bckgrd: Color,
    /// Other color, often used as a selection color.
    pub alt: Color,
}
use crate::{
    eadk::{
        backlight::{brightness, set_brightness},
        display::{self, draw_string, push_rect, push_rect_uniform, SCREEN_HEIGHT},
        timing, Color, Point, Rect,
    },
    utils::get_centered_text_x_coordo,
};

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

/// Make the screen fade to 0 [brightness], then back to original brightness
pub fn fading(dur: u32) {
    let start_brightness: u8 = if brightness() == 0 { 16 } else { brightness() };
    let mut bs = start_brightness;
    while bs != 0 {
        bs -= 1;
        timing::msleep(dur / start_brightness as u32);
        set_brightness(bs);
    }
    fill_screen(Color::BLACK);
    display::wait_for_vblank();
    set_brightness(start_brightness);
}

#[inline(always)]
/// Draws the given string centered in the x coordinate
pub fn draw_centered_string(
    string: &str,
    posy: u16,
    large: bool,
    cfg: &ColorConfig,
    // Use alt text color or not
    alt: bool,
) {
    draw_string_cfg(
        string,
        Point::new(get_centered_text_x_coordo(string, large), posy),
        large,
        cfg,
        alt,
    )
}

#[inline(always)]
/// Like [draw_string] but with a [ColorConfig] instead of two colors.
pub fn draw_string_cfg(string: &str, pos: Point, large: bool, cfg: &ColorConfig, alt: bool) {
    draw_string(
        string,
        pos,
        large,
        if alt { cfg.alt } else { cfg.text },
        cfg.bckgrd,
    )
}

/// A random and most likely never used [Color] to use as transparency. e700ff, or (231, 0, 255)
pub const TRANSPARENCY_COLOR: Color = Color::from_rgb888(231, 0, 255);

/// Draws an image from its pixel data, given as an array of [Color].
///
/// Can be scaled and can take care of transparency, but no scaling and no transparency is incomparably faster. Use carefully.
///
/// Scaling is very slow, please consider using full size images if speed is needed !
pub fn draw_image(image: &[Color], pos: Point, size: (u16, u16), scaling: u16, transparency: bool) {
    if scaling > 1 {
        // If scaling is > 1, there is no choice : we need to print one rect / pixel.
        let mut x_pos = 0;
        let mut y_pos = 0;
        for c in image {
            let y_temp: u16 = pos.y + y_pos * scaling;
            if y_temp < SCREEN_HEIGHT && (!transparency || c.rgb565 != TRANSPARENCY_COLOR.rgb565) {
                push_rect_uniform(
                    Rect {
                        x: pos.x + x_pos * scaling,
                        y: y_temp,
                        width: scaling,
                        height: scaling,
                    },
                    *c,
                );
            }

            x_pos += 1;
            if x_pos >= size.0 {
                x_pos = 0;
                y_pos += 1;
                if y_pos >= size.1 {
                    break;
                }
            }
        }
    } else if transparency && scaling == 1 {
        // not so fast option : we can send less rectangles by sending pixels line by line, splitting at transparent pixels.
        for (line, c) in image.chunks(size.0 as usize).enumerate() {
            // line by line
            let mut x_pos = 0;
            for chunk in c.split_inclusive(|d| d.rgb565 == TRANSPARENCY_COLOR.rgb565) {
                // for each chunk, separated by transparent pixel, print it
                if chunk.len() > 1 {
                    push_rect(
                        Rect {
                            x: pos.x + x_pos,
                            y: pos.y + line as u16,
                            width: chunk.len() as u16
                                - if chunk.last().unwrap().rgb565 == TRANSPARENCY_COLOR.rgb565 {
                                    1
                                } else {
                                    0
                                },
                            height: 1,
                        },
                        chunk,
                    );
                    x_pos += chunk.len() as u16;
                } else {
                    x_pos += 1;
                }
            }
        }
    } else {
        // fast option
        push_rect(
            Rect {
                x: pos.x,
                y: pos.y,
                width: size.0,
                height: size.1,
            },
            image,
        );
    }
}
