#![no_std]
#![no_main]
#![allow(dead_code)]

use numworks_utils::eadk::{display::wait_for_vblank, keyboard, State};

pub mod image;
pub mod scene;
pub mod sprite;

use scene::Scene;

pub static TEST_DATA: &[u8] = include_bytes!("./cartes.decode");

pub fn main_loop<const MAX_SPRITES: usize>(
    action: fn(State, &mut Scene<MAX_SPRITES>),
    scene: &mut Scene<MAX_SPRITES>,
) {
    loop {
        action_loop(action, scene);
        draw_loop(scene);
    }
}

fn action_loop<const MAX_SPRITES: usize>(
    action: fn(State, &mut Scene<MAX_SPRITES>),
    scene: &mut Scene<MAX_SPRITES>,
) {
    let keyboard_state: State = keyboard::scan();
    action(keyboard_state, scene);
}

fn draw_loop<const MAX_SPRITES: usize>(scene: &mut Scene<MAX_SPRITES>) {
    wait_for_vblank();
    scene.draw_entire_scene();
}
