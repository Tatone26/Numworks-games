/* et pipes_interval = (50, 150);

// !! more to the left = bad wrapping.
let mut pipes_x_pos = SCREEN_WIDTH - 3 * TILESET.tile_size;
let mut last_pipes_pos: u16 = pipes_x_pos;
let mut pipes_speed = 5;
let mut bottom_need_to_move: bool = true;
let mut last_move: u64 = timing::millis(); */

use numworks_utils::{
    eadk::{
        display::{SCREEN_HEIGHT, SCREEN_WIDTH},
        timing,
    },
    utils::randint,
};

use crate::{
    flappy_ui::{clear_bottom_pipes, clear_top_pipe, draw_bottom_pipes, draw_top_pipe, TILESET},
    game::WINDOW_SIZE,
};

pub struct Pipes {
    pub interval: (u16, u16),
    pub x_pos: u16,
    speed: u16,

    last_move: u64,
    last_pos: u16,
    has_moved: bool,
}

/// visual, but dictates the speed too.
const PIPES_REFRESH_SPEED: u64 = 25;

impl Pipes {
    pub fn new(speed: u16, interval_size: u16) -> Self {
        Self {
            interval: Self::random_interval(interval_size),
            x_pos: SCREEN_WIDTH - 3 * TILESET.tile_size,
            speed,
            last_move: timing::millis(),
            last_pos: SCREEN_WIDTH - 3 * TILESET.tile_size,
            has_moved: false,
        }
    }

    fn random_interval(interval_size: u16) -> (u16, u16) {
        let up = randint(
            (WINDOW_SIZE + TILESET.tile_size) as u32,
            (SCREEN_HEIGHT - WINDOW_SIZE - TILESET.tile_size - interval_size) as u32,
        ) as u16;
        (up, up + interval_size)
    }

    pub fn increase_speed(self: &mut Self) {
        self.speed += 1;
    }

    pub fn action_function(self: &mut Self) -> u16 {
        if self.has_moved {
            clear_bottom_pipes(self.last_pos, self.interval);
            draw_bottom_pipes(self.x_pos, self.interval);
            self.has_moved = false;
        }
        let time = timing::millis();
        let mut result = 0;
        if time - self.last_move >= PIPES_REFRESH_SPEED {
            clear_top_pipe(self.x_pos, self.interval);
            self.last_pos = self.x_pos;
            if self.x_pos <= self.speed {
                clear_bottom_pipes(self.x_pos, self.interval);
                self.interval = Self::random_interval(self.interval.1 - self.interval.0);
                self.x_pos = SCREEN_WIDTH - 3 * TILESET.tile_size;
                result += 1;
                draw_bottom_pipes(self.x_pos, self.interval);
            } else {
                self.x_pos -= self.speed;
            }
            draw_top_pipe(self.x_pos, self.interval);
            self.last_move = time;
            self.has_moved = true;
        }
        result
    }

    pub fn draw_self(self: &mut Self) {
        draw_bottom_pipes(self.x_pos, self.interval);
        draw_top_pipe(self.x_pos, self.interval);
    }
}
