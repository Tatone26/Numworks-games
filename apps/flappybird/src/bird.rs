use numworks_utils::{
    eadk::{
        display::{push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, Point, Rect, State,
    },
    utils::CENTER,
};

use crate::{
    flappy_ui::{draw_bird, BACKGROUND, TILESET_TILE_SIZE},
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
    old_x_pos: u16,
    old_y_pos: u16,
}

impl Player {
    pub fn new(jump_power: f32) -> Self {
        Self {
            jump_pressed: false,
            animation_frame: 0,
            y_speed: 0.0,
            jump_power,
            y_pos: CENTER.y,
            x_pos: X_BIRD_POS,
            old_x_pos: X_BIRD_POS,
            old_y_pos: CENTER.y,
        }
    }

    /// Execute each frame (or a soon as the calculator can)
    pub fn action_function(&mut self, keyboard_state: State, killer_floor: bool) -> bool {
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
        self.old_x_pos = self.x_pos;
        self.old_y_pos = self.y_pos;
        let new_pos = self.y_pos as i16 + self.y_speed as i16;
        if new_pos <= WINDOW_SIZE as i16 {
            self.y_speed += GRAVITY; // head bump
            self.y_pos = WINDOW_SIZE;
        } else if new_pos >= (SCREEN_HEIGHT - WINDOW_SIZE - TILESET_TILE_SIZE) as i16 {
            self.y_speed = 0.1; // foot bump
            self.y_pos = SCREEN_HEIGHT - WINDOW_SIZE - TILESET_TILE_SIZE;
            if killer_floor {
                return true;
            }
        } else {
            self.y_pos = new_pos as u16;
        }
        false
    }

    #[inline]
    pub fn clear_old_self(&self) {
        push_rect_uniform(
            Rect {
                x: self.old_x_pos,
                y: self.old_y_pos,
                width: TILESET_TILE_SIZE,
                height: TILESET_TILE_SIZE,
            },
            BACKGROUND,
        );
    }

    pub fn draw_self(&mut self) {
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
