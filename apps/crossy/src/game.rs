use heapless::Vec;
use numworks_utils::{
    eadk::{
        display::{push_rect_uniform, wait_for_vblank},
        key, keyboard, Color, Point, Rect,
    },
    graphical::ColorConfig,
    menu::{
        pause_menu,
        settings::{write_values_to_file, Setting},
        start_menu,
    },
};

use crate::{
    frog::{Direction, Frog},
    frogger_ui::ScrollController,
    world::World,
};

const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: Color::WHITE,
    alt: Color::RED,
};

/// Row threshold where scrolling begins (middle of screen)
pub const SCROLL_THRESHOLD_Y: u16 = 10;

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
            choice: 0,
            values: Vec::from_slice(&[0, 0, 1000]).unwrap(),
            texts: Vec::new(),
            fixed_values: false,
            user_modifiable: false,
        },
    ];
    loop {
        let start = start_menu(
            "FROGGER\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./data/model_controls.txt"),
            "frogger",
        );
        if start == 0 {
            unsafe {
                EXEMPLE = opt[0].get_setting_value() != 0;
            }
            let mut high_score: u32 = opt[1].get_setting_value();
            loop {
                let action = game(opt[0].get_setting_value() != 0, &mut high_score);
                opt[1].set_value(high_score);
                write_values_to_file(&mut opt, "model");
                if action == 2 {
                    return;
                } else if action == 1 {
                    break;
                }
            }
        } else {
            return;
        }
    }
}

pub fn game(_exemple: bool, high_score: &mut u32) -> u8 {
    let mut world = World::new();
    world.draw_world(0);

    let mut frog = Frog::new(Point { x: 5, y: 15 });
    let mut scroll_ctrl = ScrollController::new();

    loop {
        let scan = keyboard::scan();
        if scan.key_down(key::OK) {
            let answer = pause_menu(&COLOR_CONFIG, 30);
            if answer != 0 {
                return answer;
            }
        }

        if scroll_ctrl.is_scrolling() {
            if let Some(y_offset) = scroll_ctrl.current_y_offset() {
                wait_for_vblank();
                world.draw_world(y_offset);
                frog.draw_frog();

                scroll_ctrl.advance();

                if !scroll_ctrl.is_scrolling() {
                    world.shift_down();
                    world.draw_world(0);
                    frog.draw_frog();
                }
            }
        } else {
            if let Some(direction) = frog.read_input(scan) {
                // Determine target tile with strict grid boundary clamping
                if let Some(target_pos) = direction.try_move(frog.grid_pos()) {
                    if let Some(tile) = world.get_tile_at(target_pos) {
                        if !tile.has_obstacle {
                            // Check if moving UP at or above the camera threshold
                            if direction == Direction::Up && frog.grid_pos().y <= SCROLL_THRESHOLD_Y
                            {
                                scroll_ctrl.trigger_scroll();
                                frog.start_cooldown(); // Lock input during jump/scroll
                            } else {
                                frog.jump(direction);
                            }
                        }
                    }
                }
            }

            wait_for_vblank();
            frog.draw_frog();
        }

        frog.update();
    }
}
