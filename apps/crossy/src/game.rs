use heapless::Vec;
use numworks_utils::{
    eadk::{
        display::{self, push_rect_uniform, wait_for_vblank},
        key, keyboard, timing, Color, Point, Rect,
    },
    graphical::{draw_centered_string, draw_string_cfg, fill_screen, tiling::Tileset, ColorConfig},
    include_bytes_align_as,
    menu::{
        pause_menu,
        settings::{write_values_to_file, Setting},
        start_menu,
    },
    utils::{string_from_u16, string_from_u32, wait_for_no_keydown, CENTER},
};

use crate::{
    frog::{self, Frog},
    world::{self, World},
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
            "FROGGER\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./data/model_controls.txt"),
            "frogger", // filename to store settings
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

/// The entire game is here.
pub fn game(_exemple: bool, high_score: &mut u32) -> u8 {
    let mut world = World::new();
    world.draw_world();

    let mut frog = Frog::new(Point { x: 5, y: 9 });

    loop {
        let scan = keyboard::scan();
        if scan.key_down(key::OK) {
            let answer = pause_menu(&COLOR_CONFIG, 30);
            if answer != 0 {
                return answer;
            }
        }

        frog.update();
        // will also update the moveables when they exist.

        if let Some(direction) = frog.read_input(scan) {
            let next_position = direction.add_to_point(frog.grid_pos());
            if let Some(tile) = world.get_tile_at(next_position) {
                if !tile.has_obstacle {
                    frog.jump(direction);
                }
            }
        }

        frog.draw_frog();
        // will also draw the moveables when they exist.
    }
    1
}
