use heapless::Vec;
use numworks_utils::{
    eadk::{
        display::{self, push_rect_uniform, wait_for_vblank},
        key, keyboard, timing, Color, Point, Rect,
    },
    graphical::{draw_centered_string, draw_string_cfg, fill_screen, tiling::Tileset, ColorConfig},
    include_bytes_align_as,
    menu::{
        settings::{write_values_to_file, Setting},
        start_menu,
    },
    utils::{string_from_u16, string_from_u32, wait_for_no_keydown, CENTER},
};

// This dictates the principal colors that will be used
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: Color::WHITE,
    alt: Color::RED,
};

static mut EXEMPLE: bool = false;

fn vis_addon() {
    push_rect_uniform(
        Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        },
        Color::BLACK,
    );
}
/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut Setting; 2] = [
        &mut Setting {
            name: "Modifiable option !\0",
            choice: 0,
            values: Vec::from_slice(&[1, 0]).unwrap(),
            texts: Vec::from_slice(&["True\0", "False\0"]).unwrap(),
            fixed_values: true,
            user_modifiable: true,
        },
        &mut Setting {
            name: "High-score option !\0",
            choice: 0,                                       // forced
            values: Vec::from_slice(&[0, 0, 1000]).unwrap(), // default value, min, max
            texts: Vec::new(),
            fixed_values: false,    // allows using any value
            user_modifiable: false, // will not appear in "setting" page
        },
    ];
    loop {
        let start = start_menu(
            "TEST\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./data/model_controls.txt"),
            "model", // filename to store settings
        );
        // The menu does everything itself !
        if start == 0 {
            unsafe {
                EXEMPLE = opt[0].get_setting_value() != 0; // You could use mutable statics, but you shouldn't
            }
            // exemple of a way to have a stored value modified by the game (like a high_score)
            let mut high_score: u32 = opt[1].get_setting_value();
            loop {
                // a loop where the game is played again and again, which means it should be 100% contained after the menu
                // calling the game based on the parameters is better
                let action = game(opt[0].get_setting_value() != 0, &mut high_score);
                // necessary to store the high_score (or other similar data):
                opt[1].set_value(high_score);
                write_values_to_file(&mut opt, "model");
                // this shoudln't change
                if action == 2 {
                    // 2 means quitting
                    return;
                } else if action == 1 {
                    // 1 means back to menu
                    break;
                } // if action == 0 : rejouer
            }
        } else {
            return;
        }
    }
}

const IMAGE_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/image.nppm");

/// Images work really well with square tiles. You can still draw other images, but it is less good.
static TILESET: Tileset = Tileset::new(55, 1, IMAGE_BYTES);

/// The entire game is here.
pub fn game(_exemple: bool, high_score: &mut u32) -> u8 {
    {
        fill_screen(Color::WHITE);
        *high_score += 1; // will be stored in the file with the parameters !
        draw_string_cfg(
            &string_from_u32(*high_score),
            Point { x: 0, y: 50 },
            false,
            &COLOR_CONFIG,
            false,
        );
        draw_string_cfg(
            "Push <OK> for some magic ! <Back> to quit back to menu.\0",
            Point::new(0, 0),
            false,
            &COLOR_CONFIG,
            true,
        );
        loop {
            let keyboard_state = keyboard::scan();
            if keyboard_state.key_down(key::OK) {
                wait_for_no_keydown();
                // fill_screen(Color::WHITE);
                display::wait_for_vblank();
                TILESET.tiling(Point::new(0, 0), (5, 4), Point::new(0, 0), false, 1);
                TILESET.draw_tile(Point::new(0, 0), Point::new(0, 0), 2, false);
            } else if keyboard_state.key_down(key::BACK) {
                break;
            } else if keyboard_state.key_down(key::ONE) {
                measure_refresh_rate();
            }
        }
    }
    fill_screen(Color::GREEN);
    1
}

fn measure_refresh_rate() {
    draw_string_cfg(
        "Measuring refresh rate mean...\0",
        Point { x: 50, y: 50 },
        false,
        &COLOR_CONFIG,
        true,
    );
    let mut mean = 0;
    let mut last_timing = timing::millis();
    for i in 0..500_u64 {
        wait_for_vblank();
        let t = timing::millis();
        draw_centered_string(
            &string_from_u16(i as u16),
            CENTER.y,
            false,
            &COLOR_CONFIG,
            false,
        );
        mean = (mean * i + (t - last_timing)) / (i + 1);
        last_timing = t;
    }
    draw_string_cfg(
        "Result found (ms/frame): \0",
        Point { x: 50, y: 160 },
        true,
        &COLOR_CONFIG,
        false,
    );
    draw_centered_string(
        &string_from_u32(mean as u32),
        200,
        true,
        &COLOR_CONFIG,
        true,
    );
}
