use numworks_utils::eadk::{
    display::{SCREEN_HEIGHT, SCREEN_WIDTH},
    key, Point, State,
};

use crate::{
    flappy_ui::{clear_tile, draw_bird, TILESET_TILE_SIZE},
    game::WINDOW_SIZE,
};

const X_BIRD_POS: u16 = SCREEN_WIDTH / 3;

const GRAVITY: f32 = 0.7;
const MAX_SPEED: f32 = 8.0;

pub struct Player {
    jump_pressed: bool,  // if the jump button is pressed.
    animation_frame: u8, // where we are on the animation.
    y_speed: f32,        // up if negative, down if positive
    jump_power: f32,
    pub y_pos: u16,
    pub x_pos: u16,
}

impl Player {
    pub fn new(jump_power: f32) -> Self {
        Self {
            jump_pressed: false,
            animation_frame: 0,
            y_speed: 0.0,
            jump_power,
            y_pos: SCREEN_HEIGHT / 2,
            x_pos: X_BIRD_POS,
        }
    }

    /// Execute each frame (or a soon as the calculator can)
    pub fn action_function(self: &mut Self, keyboard_state: State, killer_floor: bool) -> bool {
        self.y_speed += GRAVITY;
        if self.y_speed > MAX_SPEED {
            self.y_speed = MAX_SPEED;
        }
        if !self.jump_pressed && keyboard_state.key_down(key::OK) {
            self.jump_pressed = true;
            self.y_speed = -self.jump_power;
        } else if !keyboard_state.key_down(key::OK) {
            self.jump_pressed = false;
        }
        self.clear_self();
        let new_pos = self.y_pos as i16 + self.y_speed as i16;
        if new_pos <= WINDOW_SIZE as i16 {
            self.y_speed += GRAVITY; // head bump
            self.y_pos = WINDOW_SIZE;
        } else if new_pos >= (SCREEN_HEIGHT - WINDOW_SIZE - TILESET_TILE_SIZE) as i16 {
            self.y_speed = 0.1; // foot bump
            self.y_pos = SCREEN_HEIGHT - WINDOW_SIZE - TILESET_TILE_SIZE;
            if killer_floor {
                self.draw_self();
                return true;
            }
        } else {
            self.y_pos = new_pos as u16;
        }
        self.draw_self();
        false
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
