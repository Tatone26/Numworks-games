use numworks_utils::{
    eadk::{
        display::{draw_string, push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        Color, Point, Rect,
    },
    graphical::{draw_image, tiling::Tileset},
    include_bytes_align_as,
    utils::{get_string_pixel_size, string_from_u16},
};

use crate::game::WINDOW_SIZE;

pub const TILESET_TILE_SIZE: u16 = 20;
/// Images work really well with square tiles. You can still draw other images, but it is less good.
pub static TILESET: Tileset = Tileset {
    tile_size: TILESET_TILE_SIZE,
    width: 4 * TILESET_TILE_SIZE,
    image: include_bytes_align_as!(Color, "./data/image.nppm"),
};
pub const PIXELS: usize = { 20 * 20 } as usize;

pub const BACKGROUND: Color = Color::from_rgb888(128, 212, 255);
pub const UI_BACKGROUND: Color = Color::from_rgb888(50, 50, 50);

fn draw_cloud(pos: Point, left_tile: &[Color; PIXELS], right_tile: &[Color; PIXELS]) {
    draw_image(
        left_tile,
        pos,
        (TILESET_TILE_SIZE, TILESET_TILE_SIZE),
        1,
        false,
    );
    draw_image(
        right_tile,
        Point {
            x: pos.x + TILESET_TILE_SIZE,
            y: pos.y,
        },
        (TILESET_TILE_SIZE, TILESET_TILE_SIZE),
        1,
        false,
    );
}

pub fn move_cloud(
    pos: &mut Point,
    counter: u16,
    left_tile: &[Color; PIXELS],
    right_tile: &[Color; PIXELS],
) {
    if pos.x <= WINDOW_SIZE {
        push_rect_uniform(
            Rect {
                x: pos.x,
                y: pos.y,
                width: TILESET_TILE_SIZE * 2,
                height: TILESET_TILE_SIZE,
            },
            BACKGROUND,
        );
        pos.x = SCREEN_WIDTH - WINDOW_SIZE - TILESET_TILE_SIZE * 2;
    } else if counter % 2 == 0 {
        push_rect_uniform(
            Rect {
                x: pos.x + TILESET_TILE_SIZE * 2 - 1,
                y: pos.y,
                width: 1,
                height: TILESET_TILE_SIZE,
            },
            BACKGROUND,
        );
        pos.x -= 1;
    }
    draw_cloud(*pos, left_tile, right_tile);
}

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
            y: SCREEN_HEIGHT - WINDOW_SIZE + 5,
            width: SCREEN_WIDTH - 2 * WINDOW_SIZE,
            height: WINDOW_SIZE - 5,
        },
        UI_BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: WINDOW_SIZE,
            y: SCREEN_HEIGHT - WINDOW_SIZE,
            width: SCREEN_WIDTH - 2 * WINDOW_SIZE,
            height: 5,
        },
        Color::from_rgb888(20, 70, 20),
    )
}
pub fn draw_ui(score: u16) {
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
    draw_string(
        "Score : \0",
        Point { x: 5, y: 1 },
        true,
        Color::WHITE,
        UI_BACKGROUND,
    );
    draw_string(
        &string_from_u16(score),
        Point {
            x: 5 + get_string_pixel_size("Score :  ", true),
            y: 1,
        },
        true,
        Color::WHITE,
        UI_BACKGROUND,
    )
}

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

pub fn clear_tile(pos: Point) {
    push_rect_uniform(
        Rect {
            x: pos.x,
            y: pos.y,
            width: TILESET_TILE_SIZE,
            height: TILESET_TILE_SIZE,
        },
        BACKGROUND,
    )
}

pub fn draw_top_pipe(
    posx: u16,
    interval: (u16, u16),
    left_tile: &[Color; PIXELS],
    right_tile: &[Color; PIXELS],
) {
    draw_top_pipes_pipes(posx, interval, left_tile, right_tile);
    draw_pipe_entrance(posx, interval.0 - TILESET_TILE_SIZE, 1);
}

pub fn draw_bottom_pipes(
    posx: u16,
    interval: (u16, u16),
    left_tile: &[Color; PIXELS],
    right_tile: &[Color; PIXELS],
) {
    draw_bottom_pipes_pipes(posx, interval, left_tile, right_tile);
    draw_pipe_entrance(posx, interval.1, 2);
}

pub fn clear_top_pipe(posx: u16, interval: (u16, u16), speed: u16) {
    if posx <= speed {
        push_rect_uniform(
            Rect {
                x: posx - if posx >= 5 { 5 } else { 0 },
                y: WINDOW_SIZE,
                width: TILESET_TILE_SIZE * 2 + 10,
                height: interval.1 - WINDOW_SIZE,
            },
            BACKGROUND,
        );
    } else {
        push_rect_uniform(
            Rect {
                x: posx + TILESET_TILE_SIZE * 2 - speed,
                y: WINDOW_SIZE,
                width: speed + 5,
                height: interval.0 - WINDOW_SIZE,
            },
            BACKGROUND,
        );
    }
}

pub fn clear_bottom_pipes(posx: u16, interval: (u16, u16), speed: u16) {
    if posx <= speed {
        push_rect_uniform(
            Rect {
                x: posx - if posx >= 5 { 5 } else { 0 },
                y: interval.1,
                width: 2 * TILESET_TILE_SIZE + 10,
                height: SCREEN_HEIGHT - WINDOW_SIZE - interval.1,
            },
            BACKGROUND,
        );
    } else {
        push_rect_uniform(
            Rect {
                x: posx + TILESET_TILE_SIZE * 2 - speed,
                y: interval.1,
                width: speed + 5,
                height: SCREEN_HEIGHT - WINDOW_SIZE - interval.1,
            },
            BACKGROUND,
        );
    }
}

/// number 1 for top pipe, 2 for bottom.
fn draw_pipe_entrance(posx: u16, posy: u16, number: u16) {
    if posx >= TILESET_TILE_SIZE {
        TILESET.draw_tile::<PIXELS>(
            Point {
                x: posx - TILESET_TILE_SIZE,
                y: posy,
            },
            Point { x: 0, y: number },
            1,
            true,
        );
    }
    TILESET.draw_tile::<PIXELS>(
        Point { x: posx, y: posy },
        Point { x: 1, y: number },
        1,
        false,
    );
    TILESET.draw_tile::<PIXELS>(
        Point {
            x: posx + TILESET_TILE_SIZE,
            y: posy,
        },
        Point { x: 2, y: number },
        1,
        false,
    );
    TILESET.draw_tile::<PIXELS>(
        Point {
            x: posx + 2 * TILESET_TILE_SIZE,
            y: posy,
        },
        Point { x: 3, y: number },
        1,
        true,
    )
}

pub fn draw_top_pipes_pipes(
    posx: u16,
    interval: (u16, u16),
    left_tile: &[Color; PIXELS],
    right_tile: &[Color; PIXELS],
) {
    draw_image(
        left_tile,
        Point {
            x: posx,
            y: WINDOW_SIZE,
        },
        (TILESET_TILE_SIZE, TILESET_TILE_SIZE),
        1,
        false,
    );
    draw_image(
        right_tile,
        Point {
            x: posx + TILESET_TILE_SIZE,
            y: WINDOW_SIZE,
        },
        (TILESET_TILE_SIZE, TILESET_TILE_SIZE),
        1,
        false,
    );
    for i in ((interval.0 % TILESET_TILE_SIZE + WINDOW_SIZE)
        ..(interval.0 - 2 * TILESET_TILE_SIZE + 1))
        .step_by(TILESET_TILE_SIZE as usize)
    {
        draw_image(
            left_tile,
            Point { x: posx, y: i },
            (TILESET_TILE_SIZE, TILESET_TILE_SIZE),
            1,
            false,
        );
        draw_image(
            right_tile,
            Point {
                x: posx + TILESET_TILE_SIZE,
                y: i,
            },
            (TILESET_TILE_SIZE, TILESET_TILE_SIZE),
            1,
            false,
        );
    }
}

fn draw_bottom_pipes_pipes(
    posx: u16,
    interval: (u16, u16),
    left_tile: &[Color; PIXELS],
    right_tile: &[Color; PIXELS],
) {
    draw_image(
        left_tile,
        Point {
            x: posx,
            y: SCREEN_HEIGHT - WINDOW_SIZE - TILESET_TILE_SIZE,
        },
        (TILESET_TILE_SIZE, TILESET_TILE_SIZE),
        1,
        false,
    );
    draw_image(
        right_tile,
        Point {
            x: posx + TILESET_TILE_SIZE,
            y: SCREEN_HEIGHT - WINDOW_SIZE - TILESET_TILE_SIZE,
        },
        (TILESET_TILE_SIZE, TILESET_TILE_SIZE),
        1,
        false,
    );
    for i in ((interval.1 + TILESET_TILE_SIZE)..(SCREEN_HEIGHT - WINDOW_SIZE - TILESET_TILE_SIZE))
        .step_by(TILESET_TILE_SIZE as usize)
    {
        draw_image(
            left_tile,
            Point { x: posx, y: i },
            (TILESET_TILE_SIZE, TILESET_TILE_SIZE),
            1,
            false,
        );
        draw_image(
            right_tile,
            Point {
                x: posx + TILESET_TILE_SIZE,
                y: i,
            },
            (TILESET_TILE_SIZE, TILESET_TILE_SIZE),
            1,
            false,
        );
    }
}
