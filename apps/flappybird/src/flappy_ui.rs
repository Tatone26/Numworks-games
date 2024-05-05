use numworks_utils::{
    eadk::{
        display::{push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        Color, Point, Rect,
    },
    utils::{draw_image, draw_tile, get_tile, Tileset},
};

/// Images work really well with square tiles. You can still draw other images, but it is less good.
pub static TILESET: Tileset = Tileset {
    tile_size: 20,
    image: include_bytes!("./image.ppm"),
};
pub const PIXELS: usize = { 20 * 20 } as usize;

pub const BACKGROUND: Color = Color::from_rgb888(128, 212, 255);
pub const UI_BACKGROUND: Color = Color::from_rgb888(50, 50, 50);

#[allow(dead_code)]
pub fn draw_ui() {
    push_rect_uniform(
        Rect {
            x: SCREEN_WIDTH - 40,
            y: 0,
            width: 40,
            height: SCREEN_HEIGHT,
        },
        UI_BACKGROUND,
    );
}

pub fn draw_bird(pos: Point) {
    draw_tile::<PIXELS>(&TILESET, pos, Point { x: 0, y: 0 }, 1, true);
}

pub fn clear_tile(pos: Point) {
    push_rect_uniform(
        Rect {
            x: pos.x,
            y: pos.y,
            width: 20,
            height: 20,
        },
        BACKGROUND,
    )
}

/// Interval is where the player CAN get through. posx is the position of the left side of the pipes.
pub fn draw_pipes(posx: u16, interval: (u16, u16)) {
    draw_pipe_entrance(posx, interval.0 - TILESET.tile_size, 1);
    draw_pipe_entrance(posx, interval.1, 2);
    draw_pipes_pipes(posx, interval);
}

pub fn clear_pipes(posx: u16, interval: (u16, u16)) {
    push_rect_uniform(
        Rect {
            x: posx,
            y: interval.0 - TILESET.tile_size,
            width: 4 * TILESET.tile_size,
            height: TILESET.tile_size,
        },
        BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: posx + TILESET.tile_size,
            y: 0,
            width: 2 * TILESET.tile_size,
            height: interval.0 - TILESET.tile_size,
        },
        BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: posx + TILESET.tile_size,
            y: interval.1 + TILESET.tile_size,
            width: 2 * TILESET.tile_size,
            height: SCREEN_HEIGHT - interval.1 + TILESET.tile_size,
        },
        BACKGROUND,
    );
    push_rect_uniform(
        Rect {
            x: posx,
            y: interval.1,
            width: 4 * TILESET.tile_size,
            height: TILESET.tile_size,
        },
        BACKGROUND,
    )
}

/// number 1 for top pipe, 2 for bottom.
fn draw_pipe_entrance(posx: u16, posy: u16, number: u16) {
    draw_tile::<PIXELS>(
        &TILESET,
        Point { x: posx, y: posy },
        Point { x: 0, y: number },
        1,
        true,
    );
    draw_tile::<PIXELS>(
        &TILESET,
        Point {
            x: posx + TILESET.tile_size,
            y: posy,
        },
        Point { x: 1, y: number },
        1,
        false,
    );
    draw_tile::<PIXELS>(
        &TILESET,
        Point {
            x: posx + 2 * TILESET.tile_size,
            y: posy,
        },
        Point { x: 2, y: number },
        1,
        false,
    );
    draw_tile::<PIXELS>(
        &TILESET,
        Point {
            x: posx + 3 * TILESET.tile_size,
            y: posy,
        },
        Point { x: 3, y: number },
        1,
        true,
    )
}

fn draw_pipes_pipes(posx: u16, interval: (u16, u16)) {
    let left_tile: [Color; PIXELS] = get_tile::<PIXELS>(&TILESET, Point { x: 1, y: 0 });
    let right_tile: [Color; PIXELS] = get_tile::<PIXELS>(&TILESET, Point { x: 2, y: 0 });
    draw_image(
        &left_tile,
        Point {
            x: posx + TILESET.tile_size,
            y: 0,
        },
        (TILESET.tile_size, TILESET.tile_size),
        1,
        false,
    );
    draw_image(
        &right_tile,
        Point {
            x: posx + 2 * TILESET.tile_size,
            y: 0,
        },
        (TILESET.tile_size, TILESET.tile_size),
        1,
        false,
    );
    for i in ((interval.0 % TILESET.tile_size)..(interval.0 - 2 * TILESET.tile_size + 1))
        .step_by(TILESET.tile_size as usize)
    {
        draw_image(
            &left_tile,
            Point {
                x: posx + TILESET.tile_size,
                y: i,
            },
            (TILESET.tile_size, TILESET.tile_size),
            1,
            false,
        );
        draw_image(
            &right_tile,
            Point {
                x: posx + 2 * TILESET.tile_size,
                y: i,
            },
            (TILESET.tile_size, TILESET.tile_size),
            1,
            false,
        );
    }

    draw_image(
        &left_tile,
        Point {
            x: posx + TILESET.tile_size,
            y: SCREEN_HEIGHT - TILESET.tile_size,
        },
        (TILESET.tile_size, TILESET.tile_size),
        1,
        false,
    );
    draw_image(
        &right_tile,
        Point {
            x: posx + 2 * TILESET.tile_size,
            y: SCREEN_HEIGHT - TILESET.tile_size,
        },
        (TILESET.tile_size, TILESET.tile_size),
        1,
        false,
    );
    for i in ((interval.1 + TILESET.tile_size)..(SCREEN_HEIGHT)).step_by(TILESET.tile_size as usize)
    {
        draw_image(
            &left_tile,
            Point {
                x: posx + TILESET.tile_size,
                y: i,
            },
            (TILESET.tile_size, TILESET.tile_size),
            1,
            false,
        );
        draw_image(
            &right_tile,
            Point {
                x: posx + 2 * TILESET.tile_size,
                y: i,
            },
            (TILESET.tile_size, TILESET.tile_size),
            1,
            false,
        );
    }
}
