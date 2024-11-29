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

/// A random and most likely never used [Color] to use as transparency. e700ff, or (231, 0, 255), which is 0xE01F in RGB565 format, the one used here.
pub const TRANSPARENCY_COLOR: Color = Color::from_rgb888(231, 0, 255);

/// Draws an image from its pixel data, given as an array of [Color].
///
/// Can be scaled and can take care of transparency, but no scaling and no transparency is incomparably faster. Use carefully.
///
/// Scaling is very slow, please consider using full size images if speed is needed !
pub fn draw_image(image: &[Color], pos: Point, size: (u16, u16), scaling: u16, transparency: bool) {
    if scaling > 1 {
        // If scaling is > 1, there is no choice : we need to print one rect / pixel. Which is very slow.
        let max_x = pos.x + size.0 * scaling;
        let max_y = pos.y + size.1 * scaling;
        let mut x_pos = pos.x;
        let mut y_pos = pos.y;
        for c in image {
            if !transparency || c.rgb565 != TRANSPARENCY_COLOR.rgb565 {
                push_rect_uniform(
                    Rect {
                        x: x_pos,
                        y: y_pos,
                        width: scaling,
                        height: scaling,
                    },
                    *c,
                );
            }
            x_pos += scaling;
            if x_pos >= max_x {
                x_pos = pos.x;
                y_pos += scaling;
                if y_pos >= max_y || y_pos >= SCREEN_HEIGHT {
                    break;
                }
            }
        }
    } else if transparency {
        // not so fast option : we can send less rectangles by sending pixels line by line, by chunks separated on transparent pixels
        for (line, row) in image.chunks(size.0 as usize).enumerate() {
            let mut start_idx = None; // Start of the current opaque run, indexed in the row.
            for (col, &pixel) in row.iter().enumerate() {
                if pixel.rgb565 == TRANSPARENCY_COLOR.rgb565 {
                    if let Some(start) = start_idx {
                        // Draw the current opaque span
                        push_rect(
                            Rect {
                                x: pos.x + start,
                                y: pos.y + line as u16,
                                width: col as u16 - start,
                                height: 1,
                            },
                            &row[start as usize..col],
                        );
                        start_idx = None;
                    }
                } else if start_idx.is_none() {
                    start_idx = Some(col as u16);
                }
            }

            // If the row ends with an opaque span, draw it
            if let Some(start) = start_idx {
                push_rect(
                    Rect {
                        x: pos.x + start,
                        y: pos.y + line as u16,
                        width: size.0 - start,
                        height: 1,
                    },
                    &row[start as usize..],
                );
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
