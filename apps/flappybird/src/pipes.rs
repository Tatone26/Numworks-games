use numworks_utils::{
    eadk::{
        display::{push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        Rect,
    },
    numbers::{abs, ceil, floor},
    utils::randint,
};

use crate::{
    flappy_ui::{clear_moving_pipe, draw_pipe, BACKGROUND, TILESET_TILE_SIZE, UI_BACKGROUND},
    game::WINDOW_SIZE,
};

/// A [Pipes] represent a pair of pipes, between which the bird need to pass.
///
/// An inactive pipe will not be drawn and is waiting for its turn to start moving
pub struct Pipes {
    pub interval: (u16, u16),
    pub x_pos: u16,
    pub active: bool,
    pub speed: f32,
    pub true_pos: f32,
    last_pos: u16,
    clear: bool,
    pub has_moved: bool,
}

impl Pipes {
    /// Last problem : with 4 pipes on the screen, it can't !
    pub fn draw_self(&self) {
        if self.active && self.has_moved {
            draw_pipe(self.x_pos, self.interval, false);
            draw_pipe(self.x_pos, self.interval, true);
            if self.x_pos >= SCREEN_WIDTH - 2 * TILESET_TILE_SIZE - WINDOW_SIZE - 10 {
                // 10 is probably overkilled but that doesn't change anything
                // I tried not to put any UI drawing here, but it was necessary optimisation wise.
                push_rect_uniform(
                    Rect {
                        x: SCREEN_WIDTH - WINDOW_SIZE,
                        y: WINDOW_SIZE,
                        width: TILESET_TILE_SIZE / 2,
                        height: SCREEN_HEIGHT - WINDOW_SIZE * 2,
                    },
                    UI_BACKGROUND,
                );
            }
        }
    }

    /// If self.clear == true, will clear the pipe considering it is on the left only !
    pub fn clear_old_self(&self) {
        if self.has_moved {
            if self.clear {
                push_rect_uniform(
                    Rect {
                        x: WINDOW_SIZE,
                        y: WINDOW_SIZE,
                        width: TILESET_TILE_SIZE * 2,
                        height: SCREEN_HEIGHT - WINDOW_SIZE * 2,
                    },
                    BACKGROUND,
                );
            } else if self.active
                && self.last_pos <= SCREEN_WIDTH - 2 * TILESET_TILE_SIZE - WINDOW_SIZE
            {
                clear_moving_pipe(self.last_pos, self.interval, ceil(self.speed) as u16, false);
                clear_moving_pipe(self.last_pos, self.interval, ceil(self.speed) as u16, true);
            }
        }
    }

    pub fn new(speed: f32, interval_size: u16) -> Self {
        Self {
            interval: Self::random_interval(interval_size),
            x_pos: SCREEN_WIDTH - 2 * TILESET_TILE_SIZE - TILESET_TILE_SIZE / 2,
            true_pos: (SCREEN_WIDTH - 2 * TILESET_TILE_SIZE - TILESET_TILE_SIZE / 2) as f32,
            active: false,
            clear: false,
            has_moved: false,
            speed,
            last_pos: SCREEN_WIDTH - 2 * TILESET_TILE_SIZE - TILESET_TILE_SIZE / 2,
        }
    }

    fn random_interval(interval_size: u16) -> (u16, u16) {
        let up = randint(
            (WINDOW_SIZE + TILESET_TILE_SIZE) as u32,
            (SCREEN_HEIGHT - WINDOW_SIZE - TILESET_TILE_SIZE - interval_size) as u32,
        ) as u16;
        (up, up + interval_size)
    }

    pub fn increase_speed(&mut self) {
        self.speed *= 1.15;
    }

    pub fn action(&mut self) -> u16 {
        let mut result = 0;
        self.clear = false;
        if self.active {
            // which means that the decimal part exceeded one
            self.move_pipe(-self.speed);
            if self.x_pos != self.last_pos {
                self.has_moved = true;
                if self.x_pos <= ceil(self.speed) as u16 {
                    self.interval = Self::random_interval(self.interval.1 - self.interval.0);
                    self.x_pos = SCREEN_WIDTH - 2 * TILESET_TILE_SIZE - TILESET_TILE_SIZE / 2;
                    self.true_pos = self.x_pos as f32;
                    self.active = false;
                    self.clear = true;

                    result = 1;
                }
            }
        } else {
            self.last_pos = SCREEN_WIDTH;
        }
        result
    }

    /// Offset the position by a given number.
    ///
    /// Used to make all pipes move on the same frame !
    ///
    /// And also to clean up a bit the action function.
    pub fn move_pipe(&mut self, offset: f32) {
        self.last_pos = self.x_pos;
        self.true_pos += offset;
        self.x_pos = abs(floor(self.true_pos) as i32) as u16;
    }
}
