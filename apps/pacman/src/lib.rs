#![no_std]

use heapless::String;
use numworks_utils::{
    eadk::{display::push_rect_uniform, Color, Point, Rect},
    utils::CENTER,
};

use game::Space;
use ghost::{GhostType, HouseState, MovementMode};
use moveable::Direction;
use pac_ui::{draw_fruit, draw_ghost, draw_player, draw_space};

/// Function that will be called by the multiple apps packages
pub use game::start;

mod game;
mod ghost;
mod levels;
mod moveable;
mod pac_ui;
mod player;

/// Function that returns a string with the name of the app for the launcher menu
pub fn get_name() -> String<15> {
    String::try_from("Pacman\0").unwrap()
}

pub fn thumbnail(_: Point) {
    push_rect_uniform(
        Rect {
            x: CENTER.x - 75,
            y: 15,
            width: 150,
            height: 100,
        },
        Color::BLUE,
    );

    push_rect_uniform(
        Rect {
            x: CENTER.x - 72,
            y: 18,
            width: 144,
            height: 94,
        },
        Color::BLACK,
    );

    // Draw dots & superball line along grid row y = 8
    for x in 8..=18 {
        let pixel_pos = Point {
            x: x * crate::game::TILE_SIZE + crate::game::X_GRID_OFFSET,
            y: 8 * crate::game::TILE_SIZE,
        };

        if x == 18 {
            draw_space(pixel_pos, Space::Superball);
        } else if x > 12 {
            draw_space(pixel_pos, Space::Point);
        }
    }

    draw_ghost(
        Point { x: 8, y: 8 },
        0,
        &Direction::Right,
        0,
        false,
        &GhostType::Blinky,
        HouseState::Outside,
        &MovementMode::Chase,
    );

    draw_player(Point { x: 11, y: 8 }, 0, &Direction::Right, 2, false);

    draw_fruit(Point { x: 14, y: 8 }, 0);

    draw_ghost(
        Point { x: 19, y: 8 },
        0,
        &Direction::Left,
        0,
        false,
        &GhostType::Inky,
        HouseState::Outside,
        &MovementMode::Frightened,
    );
}
