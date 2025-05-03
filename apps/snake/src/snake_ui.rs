use heapless::Deque;
use numworks_utils::{
    eadk::{display::push_rect_uniform, Color, Point, Rect},
    graphical::{fill_screen, tiling::Tileset},
    include_bytes_align_as,
    utils::CENTER,
};

use crate::game_snake::{Direction, GRID_OFFSET, MAX_ARRAY_SIZE};

pub const DARK_GRAY: Color = Color::from_rgb888(60, 60, 80);
pub const GRAY: Color = Color::from_rgb888(175, 175, 175);
pub const BCKD_GRAY: Color = Color::from_rgb888(200, 200, 200);
pub const LIGHT_GRAY: Color = Color::from_rgb888(225, 225, 225);
pub const DARK_GREEN: Color = Color::from_rgb888(0, 120, 0);
pub const LIGHT_GREEN: Color = Color::from_rgb888(40, 200, 120);

/// The size of a grid case. Everything is linked to this value.
pub const CASE_SIZE: u16 = 10;

const MENU_FIGURE_Y: u16 = 70;

const IMAGE_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/image.nppm");

pub static TILEMAP: Tileset = Tileset::new(CASE_SIZE, 6, IMAGE_BYTES);

/// Draws a box (case) of the grid
pub fn draw_box(x: u16, y: u16, c: Color) {
    push_rect_uniform(
        Rect {
            x: unsafe { CASE_SIZE * x + GRID_OFFSET.0 },
            y: unsafe { CASE_SIZE * y + GRID_OFFSET.1 },
            width: CASE_SIZE,
            height: CASE_SIZE,
        },
        c,
    );
}

/// Draws a terrain box (a box with the right color)
pub fn draw_terrain_box(x: u16, y: u16) {
    if ((x % 2 == 0) && (y % 2 != 0)) || ((x % 2 != 0) && (y % 2 == 0)) {
        draw_box(x, y, GRAY)
    } else {
        draw_box(x, y, LIGHT_GRAY)
    }
}

pub fn draw_fruit(x: u16, y: u16, original: bool) {
    if original {
        draw_box(x, y, Color::RED);
    } else {
        unsafe {
            TILEMAP.draw_tile(
                Point::new(x * CASE_SIZE + GRID_OFFSET.0, y * CASE_SIZE + GRID_OFFSET.1),
                Point::new(0, 0),
                1,
                true,
            );
        }
    }
}
pub fn draw_snake_front(x: u16, y: u16, direction: Direction, original: bool) {
    if !original {
        TILEMAP.draw_tile(
            unsafe { Point::new(x * CASE_SIZE + GRID_OFFSET.0, y * CASE_SIZE + GRID_OFFSET.1) },
            Point::new(
                match direction {
                    Direction::Up => 1,
                    Direction::Down => 2,
                    Direction::Left => 4,
                    Direction::Right => 3,
                },
                0,
            ),
            1,
            true,
        );
    } else {
        draw_box(x, y, LIGHT_GREEN);
    }
}

pub fn draw_wall(x: u16, y: u16, original: bool) {
    if original {
        draw_box(x, y, Color::BLACK)
    } else {
        TILEMAP.draw_tile(
            unsafe { Point::new(x * CASE_SIZE + GRID_OFFSET.0, y * CASE_SIZE + GRID_OFFSET.1) },
            Point::new(5, 0),
            1,
            true,
        );
    }
}

/// Draws the snake !
pub fn draw_snake(snake: &Deque<Point, MAX_ARRAY_SIZE>, direction: Direction, original: bool) {
    for i in snake {
        if (i.x == snake.front().unwrap().x) && (i.y == snake.front().unwrap().y) {
            draw_snake_front(i.x, i.y, direction, original)
        } else {
            draw_box(i.x, i.y, DARK_GREEN);
        }
    }
}

/// Draws the entire terrain
pub fn draw_terrain(wrapping: bool, width: u16, height: u16) {
    // display::wait_for_vblank();
    unsafe {
        fill_screen(DARK_GRAY);
        push_rect_uniform(
            Rect {
                x: match GRID_OFFSET.0.checked_sub(CASE_SIZE / 2) {
                    x @ Some(_) => x.unwrap(),
                    None => 0,
                },
                y: match GRID_OFFSET.1.checked_sub(CASE_SIZE / 2) {
                    x @ Some(_) => x.unwrap(),
                    None => 0,
                },
                width: (width + 1) * CASE_SIZE,
                height: (height + 1) * CASE_SIZE,
            },
            if !wrapping {
                Color::BLACK
            } else {
                Color::from_rgb888(255, 100, 0)
            },
        );
        push_rect_uniform(
            Rect {
                x: GRID_OFFSET.0,
                y: GRID_OFFSET.1,
                width: width * CASE_SIZE,
                height: height * CASE_SIZE,
            },
            LIGHT_GRAY,
        );
    }
    for x in (1..(width)).step_by(2) {
        for y in (0..(height)).step_by(2) {
            draw_box(x, y, GRAY)
        }
    }
    for x in (0..width).step_by(2) {
        for y in (1..height).step_by(2) {
            draw_box(x, y, GRAY)
        }
    }

    //display::wait_for_vblank();
}

/// Menu Visual Addon
pub fn menu_vis_addon() {
    TILEMAP.draw_tile(
        Point::new(CENTER.x - CASE_SIZE, MENU_FIGURE_Y),
        Point::new(3, 0),
        2,
        false,
    );
    push_rect_uniform(
        Rect {
            x: CENTER.x - CASE_SIZE * 7,
            y: MENU_FIGURE_Y,
            width: CASE_SIZE * 6,
            height: CASE_SIZE * 2,
        },
        DARK_GREEN,
    );
    TILEMAP.draw_tile(
        Point::new(CENTER.x + CASE_SIZE * 3, MENU_FIGURE_Y),
        Point::new(0, 0),
        2,
        true,
    );
}
