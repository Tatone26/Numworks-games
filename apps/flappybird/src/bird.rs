use numworks_utils::eadk::{
    display::{SCREEN_HEIGHT, SCREEN_WIDTH},
    key, Point, State,
};

use crate::{
    flappy_ui::{clear_tile, draw_bird, TILESET},
    game::WINDOW_SIZE,
};

const X_BIRD_POS: u16 = SCREEN_WIDTH / 3;

const GRAVITY: f32 = 0.7;
const JUMP_POWER: f32 = 7.0;

pub struct Player {
    jump_pressed: bool,  // if the jump button is pressed.
    animation_frame: u8, // where we are on animation.
    y_speed: f32,        // up if negative, down if positive
    y_pos: u16,
    x_pos: u16,
}

impl Player {
    pub fn new() -> Self {
        Self {
            jump_pressed: false,
            animation_frame: 0,
            y_speed: 0.0,
            y_pos: SCREEN_HEIGHT / 2,
            x_pos: X_BIRD_POS,
        }
    }

    /// Execute each frame (or a soon as the calculator can)
    pub fn action_function(self: &mut Self, keyboard_state: State) {
        self.y_speed += GRAVITY;
        if !self.jump_pressed && keyboard_state.key_down(key::OK) {
            self.jump_pressed = true;
            self.y_speed = -JUMP_POWER;
        } else if !keyboard_state.key_down(key::OK) {
            self.jump_pressed = false;
        }
        self.clear_self();
        let new_pos = self.y_pos as i16 + self.y_speed as i16;
        if new_pos <= WINDOW_SIZE as i16 {
            self.y_speed += GRAVITY; // head bump
            self.y_pos = WINDOW_SIZE;
        } else if new_pos >= (SCREEN_HEIGHT - WINDOW_SIZE - TILESET.tile_size) as i16 {
            self.y_speed = 0.1; // foot bump
            self.y_pos = SCREEN_HEIGHT - WINDOW_SIZE - TILESET.tile_size;
        } else {
            self.y_pos = new_pos as u16;
        }
        self.draw_self();
    }

    fn clear_self(self: &Self) {
        clear_tile(Point {
            x: self.x_pos,
            y: self.y_pos,
        })
    }

    pub fn draw_self(self: &mut Self) {
        if self.y_speed.is_sign_negative() {
            self.animation_frame = 1;
        } else {
            self.animation_frame = 0;
        }
        draw_bird(
            Point {
                x: self.x_pos,
                y: self.y_pos,
            },
            self.animation_frame,
        );
    }
}
