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
        display::{
            self, draw_string, push_rect, push_rect_uniform, wait_for_vblank, SCREEN_HEIGHT,
        },
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
    wait_for_vblank();
    fill_screen(Color::BLACK);
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
/// Using transparency on an image that doesn't have any will still use only one draw call, but use quite a bit more overhead.
///
/// Scaling is very VERY slow, don't use it for anything reactive.
pub fn draw_image(image: &[Color], pos: Point, size: (u16, u16), scaling: u16, transparency: bool) {
    if scaling > 1 {
        // As I almost never use scaling > 1, this is not optimised at all and I don't care for now.
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
        // This code here may be ugly, but it comes from an long optimisation work
        // I'm trying to limit as much as possible the number of push_rect calls and give the work to
        // the rust compiler, trying to limit as much as possible
        let base_ptr = image.as_ptr();
        let row_width = size.0 as usize;

        // Limit how many pixels we process vertically to stay within screen bounds
        let max_lines = u16::min(size.1, SCREEN_HEIGHT.saturating_sub(pos.y));
        let visible_pixels = (max_lines * size.0) as usize;

        // Process only the visible portion of the image buffer
        for opaque_run in image[..visible_pixels]
            .split(|c| c.rgb565 == TRANSPARENCY_COLOR.rgb565)
            .filter(|chunk| !chunk.is_empty())
        {
            // Find the starting index of this opaque run relative to the whole image buffer
            let chunk_start = unsafe { opaque_run.as_ptr().offset_from(base_ptr) as usize };

            // If the run starts mid-line (not aligned), print the partial line first
            let unaligned_offset = if chunk_start % row_width != 0 {
                let align_to_end_of_line = row_width - (chunk_start % row_width);
                let length = usize::min(opaque_run.len(), align_to_end_of_line);

                push_rect(
                    Rect {
                        x: pos.x + (chunk_start % row_width) as u16,
                        y: pos.y + (chunk_start / row_width) as u16,
                        width: length as u16,
                        height: 1,
                    },
                    &opaque_run[..length],
                );

                length
            } else {
                0
            };

            // If the entire run was drawn already (it was small and unaligned), continue
            if unaligned_offset >= opaque_run.len() {
                continue;
            }

            // Now we're aligned on the left edge of a line → draw full aligned blocks
            let full_lines = (opaque_run.len() - unaligned_offset) / row_width;
            if full_lines > 0 {
                push_rect(
                    Rect {
                        x: pos.x,
                        y: pos.y + ((chunk_start + unaligned_offset) / row_width) as u16,
                        width: size.0,
                        height: full_lines as u16,
                    },
                    &opaque_run[unaligned_offset..unaligned_offset + full_lines * row_width],
                );
            }

            // Handle any remaining trailing pixels (less than one line)
            let processed = unaligned_offset + full_lines * row_width;
            if processed < opaque_run.len() {
                push_rect(
                    Rect {
                        x: pos.x,
                        y: pos.y + ((chunk_start + processed) / row_width) as u16,
                        width: (opaque_run.len() - processed) as u16,
                        height: 1,
                    },
                    &opaque_run[processed..],
                );
            }
        }
    } else {
        // fastest option -> nothing to do but a syscall
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
