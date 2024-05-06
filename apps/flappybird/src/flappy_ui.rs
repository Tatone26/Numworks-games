use numworks_utils::{
    eadk::{
        display::{push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        Color, Point, Rect,
    },
    utils::{draw_image, draw_tile, get_tile, Tileset},
};

use crate::game::WINDOW_SIZE;

/// Images work really well with square tiles. You can still draw other images, but it is less good.
pub static TILESET: Tileset = Tileset {
    tile_size: 20,
    image: include_bytes!("./image.ppm"),
};
pub const PIXELS: usize = { 20 * 20 } as usize;

pub const BACKGROUND: Color = Color::from_rgb888(128, 212, 255);
pub const UI_BACKGROUND: Color = Color::from_rgb888(50, 50, 50);

pub fn draw_constant_ui() {
    push_rect_uniform(
        Rect {
            x: WINDOW_SIZE,
            y: 0,
            width: SCREEN_WIDTH - 2 * WINDOW_SIZE,
            height: WINDOW_SIZE,
        },
        UI_BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: WINDOW_SIZE,
            y: SCREEN_HEIGHT - WINDOW_SIZE,
            width: SCREEN_WIDTH - 2 * WINDOW_SIZE,
            height: WINDOW_SIZE,
        },
        UI_BACKGROUND,
    )
}
pub fn draw_ui() {
    push_rect_uniform(
        Rect {
            x: 0,
            y: 0,
            width: WINDOW_SIZE,
            height: SCREEN_HEIGHT,
        },
        UI_BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: SCREEN_WIDTH - WINDOW_SIZE,
            y: 0,
            width: WINDOW_SIZE,
            height: SCREEN_HEIGHT,
        },
        UI_BACKGROUND,
    );
}

pub fn draw_bird(pos: Point, frame: u8) {
    draw_tile::<PIXELS>(
        &TILESET,
        pos,
        Point {
            x: frame as u16 * 3,
            y: 0,
        },
        1,
        true,
    );
}

pub fn clear_tile(pos: Point) {
    push_rect_uniform(
        Rect {
            x: pos.x,
            y: pos.y,
            width: TILESET.tile_size,
            height: TILESET.tile_size,
        },
        BACKGROUND,
    )
}

pub fn draw_top_pipe(posx: u16, interval: (u16, u16)) {
    draw_top_pipes_pipes(posx, interval);
    draw_pipe_entrance(posx, interval.0 - TILESET.tile_size, 1);
}

pub fn draw_bottom_pipes(posx: u16, interval: (u16, u16)) {
    draw_bottom_pipes_pipes(posx, interval);
    draw_pipe_entrance(posx, interval.1, 2);
}

pub fn clear_top_pipe(posx: u16, interval: (u16, u16)) {
    push_rect_uniform(
        Rect {
            x: posx - if posx >= 5 { 5 } else { 0 },
            y: interval.0 - TILESET.tile_size,
            width: 2 * TILESET.tile_size + 10,
            height: TILESET.tile_size,
        },
        BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: posx,
            y: WINDOW_SIZE,
            width: 2 * TILESET.tile_size,
            height: interval.0 - TILESET.tile_size,
        },
        BACKGROUND,
    );
}

pub fn clear_bottom_pipes(posx: u16, interval: (u16, u16)) {
    push_rect_uniform(
        Rect {
            x: posx - if posx >= 5 { 5 } else { 0 },
            y: interval.1,
            width: 2 * TILESET.tile_size + 10,
            height: TILESET.tile_size,
        },
        BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: posx,
            y: interval.1 + TILESET.tile_size,
            width: 2 * TILESET.tile_size,
            height: SCREEN_HEIGHT - WINDOW_SIZE - (interval.1 + TILESET.tile_size),
        },
        BACKGROUND,
    );
}

/// number 1 for top pipe, 2 for bottom.
fn draw_pipe_entrance(posx: u16, posy: u16, number: u16) {
    if posx >= TILESET.tile_size {
        draw_tile::<PIXELS>(
            &TILESET,
            Point {
                x: posx - TILESET.tile_size,
                y: posy,
            },
            Point { x: 0, y: number },
            1,
            true,
        );
    }
    draw_tile::<PIXELS>(
        &TILESET,
        Point { x: posx, y: posy },
        Point { x: 1, y: number },
        1,
        false,
    );
    draw_tile::<PIXELS>(
        &TILESET,
        Point {
            x: posx + 1 * TILESET.tile_size,
            y: posy,
        },
        Point { x: 2, y: number },
        1,
        false,
    );
    draw_tile::<PIXELS>(
        &TILESET,
        Point {
            x: posx + 2 * TILESET.tile_size,
            y: posy,
        },
        Point { x: 3, y: number },
        1,
        true,
    )
}

fn draw_top_pipes_pipes(posx: u16, interval: (u16, u16)) {
    let left_tile: [Color; PIXELS] = get_tile::<PIXELS>(&TILESET, Point { x: 1, y: 0 });
    let right_tile: [Color; PIXELS] = get_tile::<PIXELS>(&TILESET, Point { x: 2, y: 0 });
    draw_image(
        &left_tile,
        Point {
            x: posx,
            y: WINDOW_SIZE,
        },
        (TILESET.tile_size, TILESET.tile_size),
        1,
        false,
    );
    draw_image(
        &right_tile,
        Point {
            x: posx + TILESET.tile_size,
            y: WINDOW_SIZE,
        },
        (TILESET.tile_size, TILESET.tile_size),
        1,
        false,
    );
    for i in ((interval.0 % TILESET.tile_size + WINDOW_SIZE)
        ..(interval.0 - 2 * TILESET.tile_size + 1))
        .step_by(TILESET.tile_size as usize)
    {
        draw_image(
            &left_tile,
            Point { x: posx, y: i },
            (TILESET.tile_size, TILESET.tile_size),
            1,
            false,
        );
        draw_image(
            &right_tile,
            Point {
                x: posx + TILESET.tile_size,
                y: i,
            },
            (TILESET.tile_size, TILESET.tile_size),
            1,
            false,
        );
    }
}

fn draw_bottom_pipes_pipes(posx: u16, interval: (u16, u16)) {
    let left_tile: [Color; PIXELS] = get_tile::<PIXELS>(&TILESET, Point { x: 1, y: 0 });
    let right_tile: [Color; PIXELS] = get_tile::<PIXELS>(&TILESET, Point { x: 2, y: 0 });
    draw_image(
        &left_tile,
        Point {
            x: posx,
            y: SCREEN_HEIGHT - WINDOW_SIZE - TILESET.tile_size,
        },
        (TILESET.tile_size, TILESET.tile_size),
        1,
        false,
    );
    draw_image(
        &right_tile,
        Point {
            x: posx + TILESET.tile_size,
            y: SCREEN_HEIGHT - WINDOW_SIZE - TILESET.tile_size,
        },
        (TILESET.tile_size, TILESET.tile_size),
        1,
        false,
    );
    for i in ((interval.1 + TILESET.tile_size)..(SCREEN_HEIGHT - WINDOW_SIZE - TILESET.tile_size))
        .step_by(TILESET.tile_size as usize)
    {
        draw_image(
            &left_tile,
            Point { x: posx, y: i },
            (TILESET.tile_size, TILESET.tile_size),
            1,
            false,
        );
        draw_image(
            &right_tile,
            Point {
                x: posx + TILESET.tile_size,
                y: i,
            },
            (TILESET.tile_size, TILESET.tile_size),
            1,
            false,
        );
    }
}
