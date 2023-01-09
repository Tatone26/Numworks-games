use heapless::Vec;

use crate::eadk::backlight::{brightness, set_brightness};
use crate::eadk::display::{
    self, draw_string, push_rect, push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use crate::eadk::{keyboard, random, timing, Color, Point, Rect};

/// Width in pixels of a large character
pub const LARGE_CHAR_WIDTH: u16 = 9;
/// Width in pixels of a small character
pub const SMALL_CHAR_WIDTH: u16 = 6;

/// Height in pixels of a large character
pub const LARGE_CHAR_HEIGHT: u16 = 16;
/// Height in pixels of a small character
pub const SMALL_CHAR_HEIGHT: u16 = 10;

/// This center is the upper left corner of the 4 squares that make the center
pub const CENTER: Point = Point::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2);

/// A structure that facilitates passing color configuration to functions.
pub struct ColorConfig {
    /// Color of the text
    pub text: Color,
    /// Color of the background
    pub bckgrd: Color,
    /// Other color, used for selection here.
    pub alt: Color,
}

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
pub fn get_centered_text_x_coordo(string: &str, large: bool) -> u16 {
    match (display::SCREEN_WIDTH).checked_sub(get_string_pixel_size(string, large)) {
        x @ Some(1_u16..=u16::MAX) => return x.unwrap() / 2,
        None | Some(0) => return 0u16,
    }
}

/// Returns the size IN PIXELS of the given string
pub fn get_string_pixel_size(string: &str, large: bool) -> u16 {
    return string.chars().count() as u16
        * (if large {
            LARGE_CHAR_WIDTH
        } else {
            SMALL_CHAR_WIDTH
        });
}

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
    );
}

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
        bs -= 1;
        timing::msleep(dur / start_brightness as u32);
        set_brightness(bs);
    }
    fill_screen(Color::BLACK);
    display::wait_for_vblank();
    set_brightness(start_brightness);
}

/// Gives a [random] integer between a and b (included)
pub fn randint(a: u32, b: u32) -> u32 {
    return random() % (b - a) + a;
}

/// Bloque le programme tend qu'une touche est appuyée
pub fn wait_for_no_keydown() {
    loop {
        let keyboard_state = keyboard::scan();
        if !keyboard_state.any_down() {
            break;
        }
    }
}

pub fn decimal_to_ascii_number(number: u8) -> Option<u8> {
    if !(48 <= number && number <= 57) {
        return None;
    } else {
        return Some(number - 48);
    }
}

const TRANSPARENCY_COLOR: Color = Color::from_rgb888(231, 0, 255);

fn get_image_parameters(image_bytes: &[u8]) -> (u16, u16, usize) {
    let mut width: u16 = 0;
    let mut height: u16 = 0;
    let mut image_start: usize = 0;
    {
        // Reading
        let mut word = Vec::<u8, 15>::new();
        let mut header_reader: u8 = 0;
        for i in 0..image_bytes.len() {
            let b = image_bytes[i];
            if b == 10 {
                if header_reader == 0 {
                    // word has info about file in it -> not very useful as I only care about RGB
                } else if header_reader == 1 {
                    let mut number = 0;
                    for n in &word {
                        let converted = decimal_to_ascii_number(*n);
                        if converted.is_none() {
                            width = number;
                            number = 0;
                        } else {
                            number = number * 10 + converted.unwrap() as u16;
                        }
                    }
                    height = number;
                } else if header_reader == 2 {
                    // word has max value in it (255 most likely)
                    word.clear();
                    image_start = i + 1;
                    break;
                }
                //do something
                header_reader += 1;
                word.clear();
            } else {
                word.push(b).unwrap();
            }
        }
    }
    return (width, height, image_start);
}

/// Draws a .ppm image from its bytes (read as u8 with include_bytes)
pub fn draw_image(
    image: &[Color],
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    upscaling: u16,
    transparency: bool,
) {
    // let (width, height, image_start) = get_image_parameters(image_bytes);
    if upscaling > 1 || transparency {
        let mut x_pos = 0;
        let mut y_pos = 0;
        for i in 0..image.len() {
            let y_temp: u16 = y + y_pos * upscaling;
            if y_temp < SCREEN_HEIGHT {
                let c = image[i];
                if !transparency || c.rgb565 != TRANSPARENCY_COLOR.rgb565 {
                    push_rect_uniform(
                        Rect {
                            x: x + x_pos * upscaling,
                            y: y_temp,
                            width: upscaling,
                            height: upscaling,
                        },
                        c,
                    );
                }
            }
            x_pos += 1;
            if x_pos >= width {
                x_pos = 0;
                y_pos += 1;
                if y_pos >= height {
                    break;
                }
            }
        }
    } else {
        push_rect(
            Rect {
                x,
                y,
                width,
                height,
            },
            image,
        );
    }
}

/// Draws an image of width*height pixels (can be scaled) from a given tilemap and its position on this tilemap.
pub fn draw_image_from_tilemap(
    tilemap: &[u8],
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    scaling: u16,
    x_map: u16,
    y_map: u16,
) {
    let (tilemap_width, _, image_start) = get_image_parameters(tilemap);
    let mut x_pos = 0;
    let mut y_pos = 0;
    for _ in 0..(width * height) as usize {
        let y_temp: u16 = y + y_pos * scaling;
        if y_temp < SCREEN_HEIGHT {
            let c = Color::from_rgb888(
                tilemap
                    [image_start + 3 * (x_map + x_pos + (y_pos + y_map) * tilemap_width) as usize],
                tilemap[image_start
                    + 3 * (x_map + x_pos + (y_pos + y_map) * tilemap_width) as usize
                    + 1],
                tilemap[image_start
                    + 3 * (x_map + x_pos + (y_pos + y_map) * tilemap_width) as usize
                    + 2],
            );
            if c.rgb565 != TRANSPARENCY_COLOR.rgb565 {
                push_rect_uniform(
                    Rect {
                        x: x + x_pos * scaling,
                        y: y_temp,
                        width: scaling,
                        height: scaling,
                    },
                    c,
                );
            }
        }
        x_pos += 1;
        if x_pos >= width {
            x_pos = 0;
            y_pos += 1;
            if y_pos >= height {
                break;
            }
        }
    }
}

/// For really fast tiling : use scaling = 1 and transparency = false.
/// In other case, every pixel will be drawn after the other, with a dedicated Rect (= far slower than a single Rect) 
pub fn tiling<const PIXELS: usize>(
    tilemap: &[u8],
    pos: Point,
    width: u16,
    height: u16,
    pos_in_tilemap: Point,
    size_in_tilemap: (u16, u16),
    transparency: bool,
    scaling: u16
) {
    let image: [Color; PIXELS] = get_image_from_tilemap(tilemap, pos_in_tilemap, size_in_tilemap);
    for x in 0..width {
        for y in 0..height {
            draw_image(
                &image,
                x * size_in_tilemap.0 + pos.x,
                y * size_in_tilemap.1 + pos.y,
                size_in_tilemap.0,
                size_in_tilemap.1,
                scaling,
                transparency
            );
        }
    }
}

/// Be careful ! Can be really useful for optimisation, but RAM is very limited, so don't use it to get too big images.
/// [draw_image] is better in the case of big images.
/// To use transparency : use draw_image with the image returned by this function.
pub fn get_image_from_tilemap<const PIXELS: usize>(
    tilemap: &[u8],
    pos_in_tilemap: Point,
    size: (u16, u16),
) -> [Color; PIXELS] {
    let (tilemap_width, _, image_start) = get_image_parameters(tilemap);
    let mut image: [Color; PIXELS] = [TRANSPARENCY_COLOR; PIXELS];
    let mut x_pos = 0;
    let mut y_pos = 0;
    for a in 0..(size.0 * size.1) as usize {
        let i = image_start
            + 3 * (pos_in_tilemap.x + x_pos + (y_pos + pos_in_tilemap.y) * tilemap_width) as usize;
        let c = Color::from_rgb888(tilemap[i], tilemap[i + 1], tilemap[i + 2]);
        image[a] = c;
        x_pos += 1;
        if x_pos >= size.0 {
            x_pos = 0;
            y_pos += 1;
            if y_pos >= size.1 {
                break;
            }
        }
    }
    return image;
}
