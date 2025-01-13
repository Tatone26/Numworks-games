use numworks_utils::{
    eadk::{
        display::{draw_string, push_rect_uniform, wait_for_vblank, SCREEN_HEIGHT, SCREEN_WIDTH},
        timing, Color, Point, Rect,
    },
    graphical::{draw_image, tiling::Tileset},
    include_bytes_align_as,
    numbers::ceil,
    utils::{get_string_pixel_size, string_from_u16},
};

use crate::game::WINDOW_SIZE;

pub const TILESET_TILE_SIZE: u16 = 20;
/// Images work really well with square tiles. You can still draw other images, but it is worse. I optimised everything to work with Tilesets.
pub static TILESET: Tileset = Tileset {
    tile_size: TILESET_TILE_SIZE,
    width: 4 * TILESET_TILE_SIZE,
    image: include_bytes_align_as!(Color, "./data/image.nppm"),
};
pub const PIXELS: usize = { 20 * 20 } as usize;

pub const BACKGROUND: Color = Color::from_rgb888(128, 212, 255);
pub const UI_BACKGROUND: Color = Color::from_rgb888(50, 50, 50);

/// A Cloud is a simple visual object that will be drawn in the background.
pub struct Cloud {
    posx: f32,
    posy: u16,
    last_pos: u16,
    speed: f32,
}

impl Cloud {
    pub fn new(pos: Point, speed: f32) -> Self {
        Cloud {
            posx: pos.x as f32,
            posy: pos.y,
            last_pos: pos.x,
            speed,
        }
    }

    pub fn action(&mut self) -> u16 {
        let p = self.posx as u16;
        self.last_pos = p;
        if p <= self.speed as u16 {
            self.posx = (SCREEN_WIDTH - WINDOW_SIZE - TILESET_TILE_SIZE * 2) as f32;
        } else {
            self.posx -= self.speed;
        }
        0
    }

    pub fn draw_self(&self) {
        let pos_x = self.posx as u16;
        TILESET.draw_tile::<PIXELS>(
            Point {
                x: pos_x,
                y: self.posy,
            },
            Point { x: 1, y: 3 },
            1,
            false,
        );
        TILESET.draw_tile::<PIXELS>(
            Point {
                x: pos_x + TILESET_TILE_SIZE,
                y: self.posy,
            },
            Point { x: 2, y: 3 },
            1,
            false,
        );
    }

    pub fn clear_old_self(&self) {
        if self.last_pos <= WINDOW_SIZE {
            push_rect_uniform(
                Rect {
                    x: self.last_pos,
                    y: self.posy,
                    width: TILESET_TILE_SIZE * 2,
                    height: TILESET_TILE_SIZE,
                },
                BACKGROUND,
            );
        } else {
            let moving = ceil(self.speed) as u16;
            push_rect_uniform(
                Rect {
                    x: self.last_pos + TILESET_TILE_SIZE * 2 - moving,
                    y: self.posy,
                    width: moving,
                    height: TILESET_TILE_SIZE,
                },
                BACKGROUND,
            );
        }
    }
}

pub fn draw_constant_ui(high_score: u16) {
    push_rect_uniform(
        Rect {
            x: 0,
            y: 0,
            width: SCREEN_WIDTH,
            height: WINDOW_SIZE,
        },
        UI_BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: SCREEN_WIDTH - WINDOW_SIZE,
            y: WINDOW_SIZE,
            width: TILESET_TILE_SIZE,
            height: SCREEN_HEIGHT,
        },
        UI_BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: 0,
            y: SCREEN_HEIGHT - WINDOW_SIZE + TILESET_TILE_SIZE / 2,
            width: SCREEN_WIDTH - WINDOW_SIZE,
            height: TILESET_TILE_SIZE / 2,
        },
        UI_BACKGROUND,
    );
    draw_string(
        "Score: \0",
        Point { x: 5, y: 1 },
        true,
        Color::WHITE,
        UI_BACKGROUND,
    );
    let s = string_from_u16(high_score);
    draw_string(
        "Best: \0",
        Point {
            x: SCREEN_WIDTH
                - 5
                - get_string_pixel_size("Best: \0", true)
                - get_string_pixel_size(&s, true),
            y: 1,
        },
        true,
        Color::WHITE,
        UI_BACKGROUND,
    );
    draw_string(
        &string_from_u16(high_score),
        Point {
            x: SCREEN_WIDTH - 5 - get_string_pixel_size(&s, true),
            y: 1,
        },
        true,
        Color::WHITE,
        UI_BACKGROUND,
    );
}

/// Draws the two vertical lines of the UI and the score.
pub fn draw_ui(score: u16) {
    // Ground clearing on the right
    push_rect_uniform(
        // This seems to be too late to be mega-clean :(
        Rect {
            x: SCREEN_WIDTH - WINDOW_SIZE,
            y: SCREEN_HEIGHT - WINDOW_SIZE,
            width: TILESET_TILE_SIZE,
            height: TILESET_TILE_SIZE / 2,
        },
        UI_BACKGROUND,
    );
    // Right vertical line (needed to keep clear of the upcoming pipes)
    // Right vertical line moved to pipe.draw_self because that means it is not called every time but more importantly it is called just after the pipe drawing
    // Left vertical line (needed to keep clear of the last pipe) -> should be drawn only when necessary, but takes care of the ground too !
    push_rect_uniform(
        Rect {
            x: 0,
            y: WINDOW_SIZE,
            width: WINDOW_SIZE,
            height: SCREEN_HEIGHT - WINDOW_SIZE,
        },
        UI_BACKGROUND,
    );

    draw_string(
        &string_from_u16(score),
        Point {
            x: 5 + get_string_pixel_size("Score: \0", true),
            y: 1,
        },
        true,
        Color::WHITE,
        UI_BACKGROUND,
    );
}

/// No scaling and no transparency -> fast, no need to store in RAM.
pub fn draw_bird(pos: Point, frame: u8) {
    TILESET.draw_tile::<PIXELS>(
        pos,
        Point {
            x: frame as u16 * 3,
            y: 0,
        },
        1,
        false,
    );
}

#[inline]
/// Draws the dead bird. Transparency for nicer collisions !
pub fn draw_dead_bird(pos: Point) {
    TILESET.draw_tile::<PIXELS>(pos, Point { x: 0, y: 3 }, 1, true);
}

#[inline]
/// Draws the shaft of the pipe and the entrance on top.
pub fn draw_pipe(
    posx: u16,
    interval: (u16, u16),
    left_tile: &[Color; PIXELS],
    right_tile: &[Color; PIXELS],
    top: bool,
) {
    draw_full_pipe(posx, interval, top);
    draw_pipe_entrance(
        posx,
        if top {
            interval.0 - TILESET_TILE_SIZE
        } else {
            interval.1
        },
        left_tile,
        right_tile,
        top,
    );
}

#[inline]
/// Does NOT clear the entirety of a pipe, only the small part on the right that moves.
pub fn clear_moving_pipe(last_pos_x: u16, interval: (u16, u16), speed: u16, top: bool) {
    push_rect_uniform(
        Rect {
            x: last_pos_x + TILESET_TILE_SIZE * 2 - speed,
            y: if top { WINDOW_SIZE } else { interval.1 },
            width: speed + 5,
            height: if top {
                interval.0 - WINDOW_SIZE
            } else {
                SCREEN_HEIGHT - WINDOW_SIZE - interval.1
            },
        },
        BACKGROUND,
    );
}

/// Draws the top part, with the small transparent parts and all.
///
/// left_tile and right_tile are the tiles with transparency.
pub fn draw_pipe_entrance(
    posx: u16,
    posy: u16,
    left_tile: &[Color; PIXELS],
    right_tile: &[Color; PIXELS],
    top: bool,
) {
    if posx > WINDOW_SIZE {
        // don't need to try to draw if it is in the UI
        draw_image(
            left_tile,
            Point {
                x: posx - TILESET_TILE_SIZE,
                y: posy,
            },
            (
                TILESET_TILE_SIZE,
                if top {
                    TILESET_TILE_SIZE
                } else {
                    TILESET_TILE_SIZE / 2
                },
            ),
            1,
            true,
        );
    }
    TILESET.draw_tile::<PIXELS>(
        Point { x: posx, y: posy },
        Point {
            x: 1,
            y: if top { 1 } else { 2 },
        },
        1,
        false,
    );
    TILESET.draw_tile::<PIXELS>(
        Point {
            x: posx + TILESET_TILE_SIZE,
            y: posy,
        },
        Point {
            x: 2,
            y: if top { 1 } else { 2 },
        },
        1,
        false,
    );
    if posx < SCREEN_WIDTH - WINDOW_SIZE - TILESET_TILE_SIZE * 2 {
        // Same, don't need to draw if in the UI
        draw_image(
            right_tile,
            Point {
                x: posx + 2 * TILESET_TILE_SIZE,
                y: posy,
            },
            (
                TILESET_TILE_SIZE,
                if top {
                    TILESET_TILE_SIZE
                } else {
                    TILESET_TILE_SIZE / 2
                },
            ),
            1,
            true,
        )
    }
}

pub fn draw_full_pipe(posx: u16, interval: (u16, u16), top: bool) {
    let start_pos = if top {
        WINDOW_SIZE
    } else {
        SCREEN_HEIGHT - WINDOW_SIZE - TILESET_TILE_SIZE
    };
    TILESET.draw_tile::<PIXELS>(
        Point {
            x: posx,
            y: start_pos,
        },
        Point { x: 1, y: 0 },
        1,
        false,
    );
    TILESET.draw_tile::<PIXELS>(
        Point {
            x: posx + TILESET_TILE_SIZE,
            y: start_pos,
        },
        Point { x: 2, y: 0 },
        1,
        false,
    );
    for i in (if top {
        (interval.0 % TILESET_TILE_SIZE + WINDOW_SIZE)..(interval.0 - 2 * TILESET_TILE_SIZE + 1)
    } else {
        (interval.1 + TILESET_TILE_SIZE)..(SCREEN_HEIGHT - WINDOW_SIZE - TILESET_TILE_SIZE)
    })
    .step_by(TILESET_TILE_SIZE as usize)
    {
        TILESET.draw_tile::<PIXELS>(Point { x: posx, y: i }, Point { x: 1, y: 0 }, 1, false);
        TILESET.draw_tile::<PIXELS>(
            Point {
                x: posx + TILESET_TILE_SIZE,
                y: i,
            },
            Point { x: 2, y: 0 },
            1,
            false,
        );
    }
}

pub fn countdown(pos: Point) {
    wait_for_vblank();
    for n in (1..=3).rev() {
        TILESET.draw_tile::<PIXELS>(pos, Point { x: n, y: 4 }, 2, true);
        timing::msleep(900);
        wait_for_vblank();
        push_rect_uniform(
            Rect {
                x: pos.x,
                y: pos.y,
                width: TILESET_TILE_SIZE * 2,
                height: TILESET_TILE_SIZE * 2,
            },
            BACKGROUND,
        );
    }
}

pub fn draw_ground(frame_counter: u16) {
    TILESET.reverse_half_tiling(
        Point {
            x: WINDOW_SIZE - frame_counter % TILESET_TILE_SIZE,
            y: SCREEN_HEIGHT - WINDOW_SIZE,
        },
        (
            (SCREEN_WIDTH - 2 * WINDOW_SIZE + TILESET_TILE_SIZE) / TILESET_TILE_SIZE,
            1,
        ),
        Point { x: 0, y: 4 },
        false,
    );
}
