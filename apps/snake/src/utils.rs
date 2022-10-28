use crate::eadk::backlight::{brightness, set_brightness};
use crate::eadk::display::{self, draw_string, SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::eadk::{random, timing, Color, Point, Rect};

pub const LARGE_CHAR_WIDTH: u16 = 10;
pub const SMALL_CHAR_WIDTH: u16 = 7;

pub const LARGE_CHAR_HEIGHT: u16 = 14;
pub const SMALL_CHAR_HEIGHT: u16 = 8;

pub const CENTER: Point = Point::new(SCREEN_WIDTH/2, SCREEN_HEIGHT/2);

pub struct ColorConfig{
    /// Color of the text
    pub text: Color,
    /// Color of the background
    pub bckgrd: Color,
    /// Other color, used for selection here.
    pub alt: Color
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

/// Returns the size in pixel of the given string
pub fn get_string_size(string : &str, large : bool) -> u16{
    let size: u16 = string.chars().count() as u16;
    return size*(if large{LARGE_CHAR_WIDTH} else {SMALL_CHAR_WIDTH})
}

/// Draws the given string centered in the x coordinate
pub fn draw_centered_string(
    string: &str,
    posy: u16,
    large: bool,
    cfg: &ColorConfig,
    // Use alt text color or not
    alt: bool
) {
    draw_string(
        string,
        Point::new(get_centered_text_left_coordo(string, large), posy),
        large,
        if alt {cfg.alt} else {cfg.text},
        cfg.bckgrd,
    );
}

/// Like [draw_string] but with a ColorConfig instead of two colors.
pub fn draw_string_cfg(string: &str, pos:Point, large:bool, cfg:&ColorConfig, alt:bool){
    draw_string(string, pos, large, if alt {cfg.alt} else {cfg.text}, cfg.bckgrd)
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
    display::wait_for_vblank();
}

/// Gives a [random] integer between a and b (included)
pub fn randint(a: u32, b: u32) -> u32 {
    return random() % (b - a) + a;
}
