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
    /// Other color, often used as a selection color.
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

/// Blocks the program until nothing is pressed.
pub fn wait_for_no_keydown() {
    loop {
        let keyboard_state = keyboard::scan();
        if !keyboard_state.any_down() {
            break;
        }
    }
}

/// Takes a decimal (from [include_bytes!]) and returns its value as a ascii number.
/// Used to read images parameters.
pub fn decimal_to_ascii_number(number: u8) -> Option<u8> {
    if !(48 <= number && number <= 57) {
        return None;
    } else {
        return Some(number - 48);
    }
}

/// A random and most likely never used [Color] to use as transparency. e700ff, ou (231, 0, 255)
pub const TRANSPARENCY_COLOR: Color = Color::from_rgb888(231, 0, 255);

/// Looks for the parameters at the start of a PPM file, and returns them as width, height and position of the first pixel in the array.
/// Probably never used outside of the drawing functions defined below.
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
                    // "P6" most likely
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
pub fn draw_image(image: &[Color], pos: Point, size: (u16, u16), scaling: u16, transparency: bool) {
    if transparency || scaling > 1 {
        let mut x_pos = 0;
        let mut y_pos = 0;
        for i in 0..image.len() {
            let y_temp: u16 = pos.y + y_pos * scaling;
            if y_temp < SCREEN_HEIGHT {
                let c = image[i];
                if !transparency || c.rgb565 != TRANSPARENCY_COLOR.rgb565 {
                    push_rect_uniform(
                        Rect {
                            x: pos.x + x_pos * scaling,
                            y: y_temp,
                            width: scaling,
                            height: scaling,
                        },
                        c,
                    );
                }
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
    } else {
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

pub struct Tileset {
    pub tile_size: u16,
    pub image: &'static [u8],
}

/// Draws an image of width*height pixels (can be scaled) from a given tileset and its position on this tileset.
pub fn draw_tile<const PIXELS: usize>(
    tileset: &Tileset,
    pos: Point,
    tile: Point,
    scaling: u16,
    transparency: bool,
) {
    let image: [Color; PIXELS] = get_tile::<PIXELS>(tileset, tile);
    draw_image(
        &image,
        pos,
        (tileset.tile_size, tileset.tile_size),
        scaling,
        transparency,
    );
}

/// For really fast tiling : use scaling = 1 and transparency = false.
/// In other case, every pixel will be drawn after the other, with a dedicated Rect (= far slower than a single Rect)
pub fn tiling<const PIXELS: usize>(
    tileset: &Tileset,
    pos: Point,
    dimensions: (u16, u16),
    // The position (in tiles) of the tile in the tilemap.
    tile: Point,
    transparency: bool,
    scaling: u16,
) {
    let image: [Color; PIXELS] = get_tile::<PIXELS>(tileset, tile);
    for x in 0..dimensions.0 {
        for y in 0..dimensions.1 {
            draw_image(
                &image,
                Point::new(x * tileset.tile_size + pos.x, y * tileset.tile_size + pos.y),
                (tileset.tile_size, tileset.tile_size),
                scaling,
                transparency,
            );
        }
    }
}

/// Be careful ! Can be really useful for optimisation, but RAM is very limited, so don't use it to get too big images.
/// [draw_image] is better in the case of big images.
/// To use transparency : use draw_image with the image returned by this function.
pub fn get_tile<const PIXELS: usize>(
    tileset: &Tileset,
    pos_in_tileset: Point, // as tiles
) -> [Color; PIXELS] {
    let (tileset_width, _, image_start) = get_image_parameters(tileset.image);
    let mut image: [Color; PIXELS] = [TRANSPARENCY_COLOR; PIXELS];
    let mut x_pos = 0;
    let mut y_pos = 0;
    for a in 0..(tileset.tile_size * tileset.tile_size) as usize {
        let i = image_start
            + 3 * (pos_in_tileset.x * tileset.tile_size
                + x_pos
                + (y_pos + pos_in_tileset.y * tileset.tile_size) * tileset_width)
                as usize;
        let c = Color::from_rgb888(tileset.image[i], tileset.image[i + 1], tileset.image[i + 2]);
        image[a] = c;
        x_pos += 1;
        if x_pos >= tileset.tile_size {
            x_pos = 0;
            y_pos += 1;
            if y_pos >= tileset.tile_size {
                break;
            }
        }
    }
    return image;
}
