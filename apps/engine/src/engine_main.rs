use enginelib::{image::Image, scene::Scene};
use heapless::{String, Vec};
use numworks_utils::{
    menu::MAX_OPTIONS_VALUES,
    utils::{draw_centered_string, string_from_u16, TRANSPARENCY_COLOR},
};

use crate::{
    eadk::{
        display::{self, push_rect_uniform},
        key, keyboard, Color, Point, Rect,
    },
    menu::{menu, MyOption, OptionType},
    utils::{draw_string_cfg, fill_screen, wait_for_no_keydown, ColorConfig},
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
    let mut opt: [&mut MyOption; 1] = [&mut MyOption {
        name: "Option !\0",
        value: 0,
        possible_values: {
            let mut v: Vec<(OptionType, &str), MAX_OPTIONS_VALUES> = Vec::new();
            unsafe { v.push_unchecked((OptionType::Bool(true), "True\0")) };
            unsafe { v.push_unchecked((OptionType::Bool(false), "False\0")) };
            v
        },
    }];
    loop {
        let start = menu(
            "Engine TEST\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./model_controls.txt"),
        );
        // The menu does everything itself !
        if start == 0 {
            unsafe {
                EXEMPLE = opt[0].get_param_value(); // You could use mutable statics, but it is not very good
            }
            loop {
                // a loop where the game is played again and again, which means it should be 100% contained after the menu
                let action = game(opt[0].get_param_value()); // calling the game based on the parameters is better
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

/// The entire game is here.
pub fn game(_exemple: bool) -> u8 {
    {
        fill_screen(Color::WHITE);
        draw_string_cfg(
            "Push <OK> for some magic ! <Back> to quit back to menu.\0",
            Point::new(0, 0),
            false,
            &COLOR_CONFIG,
            true,
        );
        let _memory_test: Vec<u16, 64000> = Vec::new(); // yes to 32000, 64000,  no to 320000, 264000, 128000, 96000
                                                        // from this, I can maybe conclude that I have around 128 000 bytes at my disposition, which is around 125 Kb.
                                                        // Good news, it passes the apparent 32Kb stack size !
        let mut text: String<52> = String::new();
        text.push_str("Memory test passed ! ").unwrap();
        draw_centered_string(&text, 50, true, &COLOR_CONFIG, true);

        loop {
            let keyboard_state = keyboard::scan();
            if keyboard_state.key_down(key::OK) {
                wait_for_no_keydown();
                // fill_screen(Color::WHITE);
                let image: Image = enginelib::image::Image::from_bytes(enginelib::TEST_DATA);
                let mut text: String<52> = String::new();
                text.push_str("checks : ").unwrap();
                let level_str: String<16> = string_from_u16(image.height as u32);
                text.push_str(&level_str).unwrap();
                text.push_str(" : ").unwrap();
                text.push_str(&string_from_u16(image.width as u32)).unwrap();
                draw_centered_string(&text, 100, true, &COLOR_CONFIG, false);

                let background = enginelib::sprite::Sprite::new(
                    Point { x: 0, y: 0 },
                    &image,
                    Rect {
                        x: 0,
                        y: 0,
                        width: image.width,
                        height: image.height,
                    },
                    None,
                    0,
                );
                let sprite = enginelib::sprite::Sprite::new(
                    Point { x: 50, y: 50 },
                    &image,
                    Rect {
                        x: 100,
                        y: 70,
                        width: 35,
                        height: 56,
                    },
                    Some(TRANSPARENCY_COLOR),
                    1,
                );
                let sprite2 = enginelib::sprite::Sprite::new(
                    Point { x: 10, y: 10 },
                    &image,
                    Rect {
                        x: 20,
                        y: 70,
                        width: 35,
                        height: 56,
                    },
                    Some(TRANSPARENCY_COLOR),
                    2,
                );
                let mut scene: Scene<'_, 10> = Scene::default();
                scene.insert(&sprite);
                scene.insert(&sprite2);
                scene.insert(&background);
                display::wait_for_vblank();
                image.draw(Point { x: 0, y: 0 });
                scene.draw_entire_scene();
            } else if keyboard_state.key_down(key::ONE) {
                display::wait_for_vblank();
                fill_screen(Color::WHITE);
            } else if keyboard_state.key_down(key::BACK) {
                break;
            }
        }
    }
    fill_screen(Color::GREEN);
    return 1;
}
