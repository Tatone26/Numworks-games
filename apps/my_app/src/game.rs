use crate::{
    eadk::{
        self,
        display::{self, push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, timing, Point, Rect,
    },
    menu::pause_menu,
    utils::{draw_centered_string, fading, fill_screen, randint},
};
use eadk::Color;
use heapless::Deque;
use heapless::String;

#[derive(Debug)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

const GRAY: Color = Color::from_rgb888(175, 175, 175);
const LIGHT_GRAY: Color = Color::from_rgb888(225, 225, 225);
const DARK_GREEN: Color = Color::from_rgb888(0, 120, 0);
const LIGHT_GREEN: Color = Color::from_rgb888(40, 200, 120);

const CASE_SIZE: u16 = 10;
const MAX_ARRAY_SIZE: usize = ((SCREEN_WIDTH / CASE_SIZE) ^ 2) as usize; // No array in this program should ever be higher than N*N, N being the number of cases.
                                                                         // It is almost impossible for any array to be this big, and it may cause memory issues. Don't know. Just gonna try.

const SPEED: u16 = 250; // Number of millis between each move

pub fn game() {
    fill_screen(Color::WHITE);
    draw_terrain();
    //This mutable variables are the heart of everything. Really.
    let mut snake: Deque<Point, MAX_ARRAY_SIZE> = Deque::new();
    unsafe {
        snake.push_front_unchecked(Point::new(3, 1));
        snake.push_front_unchecked(Point::new(2, 1));
        snake.push_front_unchecked(Point::new(1, 1));
    }
    let mut last_move: u64 = timing::millis();
    let mut direction: Direction = Direction::RIGHT;
    let mut fruit_pos: Point = get_random_point();
    let mut points: u16 = 0;

    draw_box(fruit_pos.x, fruit_pos.y, Color::RED);
    loop {
        let keyboard_state = keyboard::scan();
        if keyboard_state.key_down(key::BACKSPACE) {
            let mut string_points: String<15> = String::from(" Points : ");
            let points_str: String<15> = String::from(points);
            string_points.push_str(points_str.as_str()).unwrap();
            string_points.push_str(" \0").unwrap();
            draw_centered_string(&string_points, 5, true, Color::BLACK, Color::WHITE);
            let action = pause_menu(Color::BLACK, Color::WHITE, Color::GREEN);
            if action == 0 {
                fading(500);
                fill_screen(Color::BLACK);
                return;
            } else if action == 1 {
                draw_terrain();
                draw_snake(&snake);
                draw_box(fruit_pos.x, fruit_pos.y, Color::RED);
                // redraw the walls too, it is so fast that we can redraw everything and not just what was covered.
            }
        } else if keyboard_state.key_down(key::UP) {
            direction = Direction::UP;
        } else if keyboard_state.key_down(key::DOWN) {
            direction = Direction::DOWN;
        } else if keyboard_state.key_down(key::RIGHT) {
            direction = Direction::RIGHT;
        } else if keyboard_state.key_down(key::LEFT) {
            direction = Direction::LEFT;
        }
        if last_move + SPEED as u64 <= timing::millis() {
            let new_point: Point;
            let front_point: &Point;
            let last_point: Point;
            unsafe {
                // Dangerous place ! Have to be really cautious about what I'm doing to this array !
                front_point = snake.front().unwrap();
                new_point = get_point_from_dir(front_point, &direction);
                snake.push_front_unchecked(new_point);
                last_point = snake.pop_back().unwrap();
            }
            draw_terrain_box(last_point.x, last_point.y);
            draw_snake(&snake);
            if (new_point.x == fruit_pos.x) & (new_point.y == fruit_pos.y) {
                fruit_pos = get_random_point();
                draw_box(fruit_pos.x, fruit_pos.y, Color::RED);
                points += 1;
            }
            last_move = timing::millis();
        }
        display::wait_for_vblank();
    }
}

fn get_random_point() -> Point {
    let x = randint(0, (SCREEN_WIDTH / CASE_SIZE) as u32) as u16;
    let y = randint(0, (SCREEN_HEIGHT / CASE_SIZE) as u32) as u16;
    return Point::new(x, y);
}

fn get_point_from_dir(p: &Point, d: &Direction) -> Point {
    let mut new_point = Point::new(p.x, p.y);
    match d {
        Direction::UP => new_point.y -= 1,
        Direction::DOWN => new_point.y += 1,
        Direction::LEFT => new_point.x -= 1,
        Direction::RIGHT => new_point.x += 1,
    }
    return new_point;
}

fn draw_snake(snake: &Deque<Point, MAX_ARRAY_SIZE>) {
    for i in snake {
        if (i.x == snake.front().unwrap().x) & (i.y == snake.front().unwrap().y) {
            draw_box(i.x, i.y, LIGHT_GREEN);
        } else {
            draw_box(i.x, i.y, DARK_GREEN);
        }
    }
}

fn draw_box(x: u16, y: u16, c: Color) {
    push_rect_uniform(
        Rect {
            x: CASE_SIZE * x,
            y: CASE_SIZE * y,
            width: CASE_SIZE,
            height: CASE_SIZE,
        },
        c,
    );
}

fn draw_terrain_box(x: u16, y: u16) {
    if ((x % 2 == 0) & (y % 2 != 0)) | ((x % 2 != 0) & (y % 2 == 0)) {
        draw_box(x, y, GRAY)
    } else {
        draw_box(x, y, LIGHT_GRAY)
    }
}

fn draw_terrain() {
    for x in 0..SCREEN_WIDTH / CASE_SIZE {
        for y in 0..SCREEN_HEIGHT / CASE_SIZE {
            draw_terrain_box(x, y);
        }
    }
}
