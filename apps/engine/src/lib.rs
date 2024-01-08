#![no_std]
#![no_main]
#![allow(dead_code)]

use numworks_utils::eadk::{display::wait_for_vblank, keyboard, State};

pub mod image;
pub mod tiles;
/// WARNING : the image NEED to be made via MY ppm_decoder, that can be found in this repo too.
pub static TEST_DATA: &[u8; 49004] = include_bytes!("test1.txt");

pub struct Scene {}

pub fn main_loop(action: fn(State, &mut Scene), scene: &mut Scene) {
    loop {
        action_loop(action, scene);
        draw_loop();
    }
}

fn action_loop(action: fn(State, &mut Scene), scene: &mut Scene) {
    let keyboard_state: State = keyboard::scan();
    action(keyboard_state, scene);
}

fn draw_loop() {
    wait_for_vblank();
    // todo
}
