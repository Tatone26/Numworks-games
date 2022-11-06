use crate::eadk::backlight::{brightness, set_brightness};
use crate::eadk::display::{self, draw_string, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::eadk::{random, timing, Color, Point, Rect, keyboard};

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
    match (display::SCREEN_WIDTH).checked_sub(get_string_pixel_size(string, large))
    {
        x @ Some(1_u16..=u16::MAX) => return x.unwrap()/2,
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
pub fn wait_for_no_keydown(){
    loop {
        let keyboard_state = keyboard::scan();
        if !keyboard_state.any_down(){
            break
        }
    }
}