use core::f32::consts::PI;

// This file represents all the function we can call in the Epsilon ecosystem.
// This is the like the libc or syscalls of the Numworks calculator.
// There are also some quality of life functions directly linked to the structures defined here, kindly given by the Numworks team.

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Color {
    pub rgb565: u16,
}

impl Color {
    pub const RED: Self = Self::from_rgb888(255, 0, 0);
    pub const GREEN: Self = Self::from_rgb888(0, 255, 0);
    pub const BLUE: Self = Self::from_rgb888(0, 0, 255);
    pub const WHITE: Self = Self::from_rgb888(255, 255, 255);
    pub const BLACK: Self = Self::from_rgb888(0, 0, 0);

    #[must_use]
    pub const fn new(rgb565: u16) -> Self {
        Self { rgb565 }
    }

    #[must_use]
    pub const fn from_rgb888(r: u8, g: u8, b: u8) -> Self {
        Self {
            rgb565: ((r as u16 & 0b1111_1000) << 8)
                | ((g as u16 & 0b1111_1100) << 3)
                | (b as u16 >> 3),
        }
    }

    #[must_use]
    pub fn from_hsv(hue: f32, saturation: f32, value: f32) -> Self {
        let f = |n: f32| {
            let k: f32 = (n + hue / PI * 3.) % 6.;
            value * (1. - saturation * k.min(4. - k).clamp(0., 1.))
        };
        Color::from_rgb888(
            (f(5.) * 255.) as u8,
            (f(3.) * 255.) as u8,
            (f(1.) * 255.) as u8,
        )
    }

    #[must_use]
    pub fn from_hv(hue: f32, value: f32) -> Self {
        let f = |n: f32| {
            let k: f32 = (n + hue / PI * 3.) % 6.;
            value * (1. - k.min(4. - k).clamp(0., 1.))
        };
        Color::from_rgb888(
            (f(5.) * 255.) as u8,
            (f(3.) * 255.) as u8,
            (f(1.) * 255.) as u8,
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub const ZERO: Point = Point::new(0, 0);

    #[must_use]
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct State(u64);

impl State {
    #[must_use]
    pub fn new(state: u64) -> Self {
        Self(state)
    }

    #[must_use]
    pub fn key_down(&self, k: u32) -> bool {
        self.0.wrapping_shr(k) & 1 != 0
    }

    #[must_use]
    pub fn any_down(&self) -> bool {
        self.0 != 0
    }
}

pub mod key {
    pub const LEFT: u32 = 0;
    pub const UP: u32 = 1;
    pub const DOWN: u32 = 2;
    pub const RIGHT: u32 = 3;
    pub const OK: u32 = 4;
    pub const BACK: u32 = 5;
    pub const HOME: u32 = 6;
    pub const SHIFT: u32 = 12;
    pub const ALPHA: u32 = 13;
    pub const XNT: u32 = 14;
    pub const VAR: u32 = 15;
    pub const TOOLBOX: u32 = 16;
    pub const BACKSPACE: u32 = 17;
    pub const EXP: u32 = 18;
    pub const LN: u32 = 19;
    pub const LOG: u32 = 20;
    pub const IMAGINARY: u32 = 21;
    pub const COMMA: u32 = 22;
    pub const POWER: u32 = 23;
    pub const SINE: u32 = 24;
    pub const COSINE: u32 = 25;
    pub const TANGENT: u32 = 26;
    pub const PI: u32 = 27;
    pub const SQRT: u32 = 28;
    pub const SQUARE: u32 = 29;
    pub const SEVEN: u32 = 30;
    pub const EIGHT: u32 = 31;
    pub const NINE: u32 = 32;
    pub const LEFTPARENTHESIS: u32 = 33;
    pub const RIGHTPARENTHESIS: u32 = 34;
    pub const FOUR: u32 = 36;
    pub const FIVE: u32 = 37;
    pub const SIX: u32 = 38;
    pub const MULTIPLICATION: u32 = 39;
    pub const DIVISION: u32 = 40;
    pub const ONE: u32 = 42;
    pub const TWO: u32 = 43;
    pub const THREE: u32 = 44;
    pub const PLUS: u32 = 45;
    pub const MINUS: u32 = 46;
    pub const ZERO: u32 = 48;
    pub const DOT: u32 = 49;
    pub const EE: u32 = 50;
    pub const ANS: u32 = 51;
    pub const EXE: u32 = 52;
}

pub mod backlight {
    pub fn set_brightness(brightness: u8) {
        unsafe {
            eadk_backlight_set_brightness(brightness);
        }
    }

    #[must_use]
    pub fn brightness() -> u8 {
        unsafe { eadk_backlight_brightness() } // 0 - 240, par pas de 16
    }

    extern "C" {
        fn eadk_backlight_set_brightness(brightness: u8);
        fn eadk_backlight_brightness() -> u8;
    }
}

pub mod display {
    use super::Color;
    use super::Point;
    use super::Rect;

    pub const SCREEN_WIDTH: u16 = 320;
    pub const SCREEN_HEIGHT: u16 = 240;

    pub fn push_rect(rect: Rect, pixels: &[Color]) {
        unsafe {
            eadk_display_push_rect(rect, pixels.as_ptr());
        }
    }

    /// Personnal addon : useful for some technical things.
    /// # Safety
    ///    Just be smart
    pub unsafe fn push_rect_ptr(rect: Rect, pixels: *const Color) {
        unsafe {
            eadk_display_push_rect(rect, pixels);
        }
    }

    pub fn push_rect_uniform(rect: Rect, color: Color) {
        unsafe {
            eadk_display_push_rect_uniform(rect, color);
        }
    }

    pub fn pull_rect(rect: Rect, pixels: &[Color]) {
        unsafe {
            eadk_display_pull_rect(rect, pixels.as_ptr());
        }
    }

    pub fn draw_string(
        string: &str,
        pos: Point,
        large: bool,
        text_color: Color,
        background_color: Color,
    ) {
        unsafe {
            eadk_display_draw_string(string.as_ptr(), pos, large, text_color, background_color);
        }
    }

    pub fn wait_for_vblank() {
        unsafe {
            eadk_display_wait_for_vblank();
        }
    }

    extern "C" {
        fn eadk_display_push_rect_uniform(rect: Rect, color: Color);
        fn eadk_display_push_rect(rect: Rect, color: *const Color);
        fn eadk_display_draw_string(
            text: *const u8,
            pos: Point,
            large: bool,
            text_color: Color,
            background_color: Color,
        );
        fn eadk_display_wait_for_vblank();
        fn eadk_display_pull_rect(rect: Rect, color: *const Color);
    }
}

pub mod keyboard {
    use super::State;

    #[must_use]
    pub fn scan() -> State {
        unsafe { State::new(eadk_keyboard_scan()) }
    }

    extern "C" {
        fn eadk_keyboard_scan() -> u64;
    }
}

pub mod timing {
    pub fn usleep(us: u32) {
        unsafe {
            eadk_timing_usleep(us);
        }
    }

    pub fn msleep(ms: u32) {
        unsafe {
            eadk_timing_msleep(ms);
        }
    }

    #[must_use]
    pub fn millis() -> u64 {
        unsafe { eadk_timing_millis() }
    }

    extern "C" {
        fn eadk_timing_usleep(us: u32);
        fn eadk_timing_msleep(us: u32);
        fn eadk_timing_millis() -> u64;
    }
}

/// Does not seem to work for now, at least with API = 1.
pub mod battery {
    /// not working
    pub fn battery_charging() -> bool {
        unsafe { eadk_battery_is_charging() }
    }
    /// not working
    #[must_use]
    pub fn battery_voltage() -> f32 {
        unsafe { eadk_battery_voltage() }
    }
    /// not working
    #[must_use]
    pub fn battery_level() -> u8 {
        unsafe { eadk_battery_level() }
    }

    extern "C" {
        fn eadk_battery_is_charging() -> bool;
        fn eadk_battery_voltage() -> f32;
        fn eadk_battery_level() -> u8;
    }
}

#[must_use]
pub fn random() -> u32 {
    unsafe { eadk_random() }
}

extern "C" {
    fn eadk_random() -> u32;
}
/// Doesn't seem to be working, at least with API = 1, if that changes anything.
pub fn usb_is_plugged() -> bool {
    unsafe { eadk_usb_is_plugged() }
}

extern "C" {
    fn eadk_usb_is_plugged() -> bool;
}

use core::fmt::Write;
use core::panic::PanicInfo;

use heapless::String;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    let mut panic_str: String<64> = String::new();
    if let Some(location) = info.location() {
        write!(
            &mut panic_str,
            "Error\nin {}\nline {}\0",
            location.file(),
            location.line()
        )
        .unwrap();
    }

    display::push_rect_uniform(
        Rect {
            x: 0,
            y: 0,
            width: display::SCREEN_WIDTH,
            height: display::SCREEN_HEIGHT,
        },
        Color::RED,
    );
    display::draw_string(
        &panic_str,
        Point::new(0, 0),
        true,
        Color::BLACK,
        Color::WHITE,
    );

    loop {}
}
