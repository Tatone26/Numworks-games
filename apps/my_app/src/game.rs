use crate::{
    eadk::{
        self,
        display::{self, push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, timing, Point, Rect,
    },
    menu::{menu, pause_menu, MyOption},
    utils::{draw_centered_string, fading, fill_screen, randint, CENTER},
};
use eadk::Color;
use heapless::String;
use heapless::{Deque, Vec};

/// Used to get directions, and nothing else !
#[derive(Debug, Copy, Clone, PartialEq)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

/// The number of Boolean Options used. Public so menu() can use it.
pub const BOOL_OPTIONS_NUMBER: usize = 2;

const GRAY: Color = Color::from_rgb888(175, 175, 175);
const LIGHT_GRAY: Color = Color::from_rgb888(225, 225, 225);
const DARK_GREEN: Color = Color::from_rgb888(0, 120, 0);
const LIGHT_GREEN: Color = Color::from_rgb888(40, 200, 120);

/// The size of a grid case. Everything is linked to this value.
const CASE_SIZE: u16 = 10;
/// The max y value on the grid
/// - 1 because the grid starts at (0, 0)
static mut MAX_HEIGHT: u16 = SCREEN_HEIGHT / CASE_SIZE - 1;
/// THe max x value on the grid
static mut MAX_WIDTH: u16 = SCREEN_WIDTH / CASE_SIZE - 1;
/// The offset of the grid
static mut GRID_OFFSET: (u16, u16) = (0, 0);
/// No array in this program can exceed this size ; it can be used to set the size.
const MAX_ARRAY_SIZE: usize =
    ((SCREEN_HEIGHT / CASE_SIZE - 1) * SCREEN_WIDTH / CASE_SIZE - 1) as usize;

/// Menu, Options and Game start
pub fn start() {
    let mut opt = [
        &mut MyOption {
            name: "HYPER RAPIDE !\0",
            value: (true, "Oui !\0"),
            possible_values: [true, false],
            possible_values_str: ["Oui !\0", "Non !\0"],
        },
        &mut MyOption {
            name: "MINI TERRAIN !\0",
            value: (true, "Oui !\0"),
            possible_values: [true, false],
            possible_values_str: ["Oui !\0", "Non !\0"],
        },
    ];
    let start = menu(
        "SNAKE 2.0\0",
        &mut opt,
        Color::BLACK,
        Color::WHITE,
        Color::GREEN,
    );
    if start == 1 {
        if opt[1].value.0 {
            unsafe {
                MAX_HEIGHT = 11;
                MAX_WIDTH = 16;
                GRID_OFFSET = (
                    CENTER.x - (MAX_WIDTH / 2) * CASE_SIZE,
                    CENTER.y - (MAX_HEIGHT / 2) * CASE_SIZE,
                )
            }
        }
        loop {
            if game(if opt[0].value.0 { 100 } else { 250 }) == 0 {
                return;
            }
        }
    } else {
        return;
    }
}

/// The entire game is here.
pub fn game(speed: u16) -> u8 {
    //This mutable variables are the heart of everything. Really.
    let mut snake: Deque<Point, MAX_ARRAY_SIZE> = Deque::new();
    unsafe {
        snake.push_front_unchecked(Point::new(1, 1));
        snake.push_front_unchecked(Point::new(2, 1));
        snake.push_front_unchecked(Point::new(3, 1));
    }
    let mut last_move: u64 = timing::millis();
    let mut direction: Direction = Direction::RIGHT;
    let mut last_direction: Direction = direction;
    let mut fruit_pos: Point = get_random_point();
    while is_in_snake(&fruit_pos, &snake) {
        fruit_pos = get_random_point();
    }
    let mut walls: Vec<Point, MAX_ARRAY_SIZE> = Vec::new();
    let mut points: u16 = 0;

    fill_screen(Color::WHITE);
    draw_terrain();
    draw_snake(&snake);
    draw_box(fruit_pos.x, fruit_pos.y, Color::RED);
    timing::msleep(300);
    loop {
        let keyboard_state = keyboard::scan();
        // very efficient scan, register most of the input, do any verification every move only.
        if keyboard_state.key_down(key::UP) {
            direction = Direction::UP;
        } else if keyboard_state.key_down(key::DOWN) {
            direction = Direction::DOWN;
        } else if keyboard_state.key_down(key::RIGHT) {
            direction = Direction::RIGHT;
        } else if keyboard_state.key_down(key::LEFT) {
            direction = Direction::LEFT;
        } else if keyboard_state.key_down(key::BACKSPACE) {
            let action = snake_pause(points);
            if action == 1{
                draw_terrain();
                draw_snake(&snake);
                draw_box(fruit_pos.x, fruit_pos.y, Color::RED);
                for i in &walls {
                    draw_box(i.x, i.y, Color::BLACK);
                }
            }else{
                return action
            }
        }
        // When the time is up !
        if last_move + speed as u64 <= timing::millis() {
            direction = check_direction(direction, last_direction);
            last_direction = direction;
            let new_point: Point;
            let front_point: Point;
            unsafe {
                // Dangerous place ! Have to be really cautious about what I'm doing to this array !
                front_point = snake.front().unwrap().clone();
                match get_point_from_dir(&front_point, &direction, &snake, &walls) {
                    // renvoie None si le nouveau point n'existe pas, est un mur ou une partie du snake (derniere partie exclue)
                    x @ Some(_) => {
                        new_point = x.unwrap();
                        snake.push_front_unchecked(new_point);
                    }
                    None => break, // You lose !
                }
                draw_box(front_point.x, front_point.y, DARK_GREEN);
            }
            if (new_point.x == fruit_pos.x) & (new_point.y == fruit_pos.y) {
                // if we ate the fruit
                points += 1;
                let next_direct_pos: Point; // Evite que les murs ou les fruits apparaissent directement devant le joueur
                match get_point_from_dir(&new_point, &direction, &snake, &walls) {
                    x @ Some(_) => next_direct_pos = x.unwrap(),
                    None => unsafe { next_direct_pos = Point::new(MAX_WIDTH, MAX_HEIGHT) },
                }
                if (points % 2) == 0 {
                    // add walls
                    let mut new_wall = get_random_point();
                    while is_in_walls(&new_wall, &walls)
                        | is_in_snake(&new_wall, &snake)
                        | ((new_wall.x == next_direct_pos.x) & (new_wall.y == next_direct_pos.y))
                    {
                        new_wall = get_random_point();
                    }
                    unsafe {
                        walls.push_unchecked(new_wall);
                    }
                    draw_box(new_wall.x, new_wall.y, Color::BLACK);
                }
                fruit_pos = get_random_point();
                while is_in_snake(&fruit_pos, &snake)
                    | is_in_walls(&fruit_pos, &walls)
                    | ((fruit_pos.x == next_direct_pos.x) & (fruit_pos.y == next_direct_pos.y))
                {
                    fruit_pos = get_random_point();
                }
                draw_box(fruit_pos.x, fruit_pos.y, Color::RED);
            } else {
                // if we didn't eat the fruit, then we get rid of the last case
                let last_point = snake.pop_back().unwrap();
                draw_terrain_box(last_point.x, last_point.y);
            }
            draw_box(new_point.x, new_point.y, LIGHT_GREEN);
            display::wait_for_vblank();
            last_move = timing::millis();
        }
    }
    draw_centered_string(
        "Perdu !\0",
        SCREEN_HEIGHT / 2,
        true,
        Color::RED,
        Color::WHITE,
    );
    let mut string_points: String<15> = String::from(" Points : ");
    string_points
        .push_str(String::<15>::from(points).as_str())
        .unwrap();
    string_points.push_str(" \0").unwrap();
    draw_centered_string(&string_points, 5, true, Color::BLACK, Color::WHITE);
    loop {
        let keyboard_state = keyboard::scan();
        if keyboard_state.key_down(key::OK) {
            return 1;
        }
    }
}

fn snake_pause(points: u16)->u8{
    let mut string_points: String<15> = String::from(" Points : ");
    string_points
        .push_str(String::<15>::from(points).as_str())
        .unwrap();
    string_points.push_str(" \0").unwrap();
    draw_centered_string(&string_points, 5, true, Color::BLACK, Color::WHITE);
    let action = pause_menu(Color::BLACK, Color::WHITE, DARK_GREEN);
    if action == 0 {
        fading(500);
        fill_screen(Color::BLACK);
        return 0;
    }
    else{
        return 1
    }
}

fn check_direction(direction: Direction, last_direction: Direction) -> Direction {
    match direction {
        // need to check if the direction is not the opposite of the actual direction.
        Direction::UP => {
            if last_direction == Direction::DOWN {
                return last_direction;
            }
        }
        Direction::DOWN => {
            if last_direction == Direction::UP {
                return last_direction;
            }
        }
        Direction::LEFT => {
            if last_direction == Direction::RIGHT {
                return last_direction;
            }
        }
        Direction::RIGHT => {
            if last_direction == Direction::LEFT {
                return last_direction;
            }
        }
    }
    direction
}

/// Check if a given Point is in a Deque (the snake)
fn is_in_snake(p: &Point, snake: &Deque<Point, MAX_ARRAY_SIZE>) -> bool {
    for i in snake {
        if (i.x == p.x) & (i.y == p.y) {
            return true;
        }
    }
    return false;
}

/// Check if a given Point is in a Vec (the walls)
fn is_in_walls(p: &Point, walls: &Vec<Point, MAX_ARRAY_SIZE>) -> bool {
    for i in walls {
        if (i.x == p.x) & (i.y == p.y) {
            return true;
        }
    }
    return false;
}

/// Give a random Point, corresponding to a position on the game grid
fn get_random_point() -> Point {
    unsafe {
        let x = randint(0, (MAX_WIDTH-1) as u32) as u16;
        let y = randint(0, (MAX_HEIGHT-1) as u32) as u16;
        return Point::new(x, y);
    }
}

/// Return a Point from a Point and a Direction, None if the Point is non-valid
fn get_point_from_dir(
    p: &Point,
    d: &Direction,
    snake: &Deque<Point, MAX_ARRAY_SIZE>,
    walls: &Vec<Point, MAX_ARRAY_SIZE>,
) -> Option<Point> {
    let mut new_point = Point::new(p.x, p.y);
    match d {
        // Big boom here ! x and y can not be < 0 !! (or too big for that matter)
        Direction::UP => {
            if new_point.y >= 1 {
                new_point.y -= 1
            } else {
                return None;
            }
        }
        Direction::DOWN => unsafe {
            if new_point.y < MAX_HEIGHT - 1{
                new_point.y += 1
            } else {
                return None;
            }
        },
        Direction::LEFT => {
            if new_point.x >= 1 {
                new_point.x -= 1
            } else {
                return None;
            }
        }
        Direction::RIGHT => unsafe {
            if new_point.x < MAX_WIDTH - 1{
                new_point.x += 1
            } else {
                return None;
            }
        },
    }
    if is_in_walls(&new_point, walls) {
        return None;
    }
    if is_in_snake(&new_point, snake) {
        let back = snake.back().unwrap();
        if (new_point.x != back.x) | (new_point.y != back.y) {
            return None;
        }
    }
    Some(new_point)
}

/// Draws the snake !
fn draw_snake(snake: &Deque<Point, MAX_ARRAY_SIZE>) {
    for i in snake {
        if (i.x == snake.front().unwrap().x) & (i.y == snake.front().unwrap().y) {
            draw_box(i.x, i.y, LIGHT_GREEN);
        } else {
            draw_box(i.x, i.y, DARK_GREEN);
        }
    }
}

/// Draws a box (case) of the grid
fn draw_box(x: u16, y: u16, c: Color) {
    unsafe {
        push_rect_uniform(
            Rect {
                x: CASE_SIZE * x + GRID_OFFSET.0,
                y: CASE_SIZE * y + GRID_OFFSET.1,
                width: CASE_SIZE,
                height: CASE_SIZE,
            },
            c,
        );
    }
}

/// Draws a terrain box (a box with the right color)
fn draw_terrain_box(x: u16, y: u16) {
    if ((x % 2 == 0) & (y % 2 != 0)) | ((x % 2 != 0) & (y % 2 == 0)) {
        draw_box(x, y, GRAY)
    } else {
        draw_box(x, y, LIGHT_GRAY)
    }
}

/// Draws the entire terrain
fn draw_terrain() {
    unsafe {
        if GRID_OFFSET != (0, 0) {
            push_rect_uniform(
                Rect {
                    x: match GRID_OFFSET.0.checked_sub(CASE_SIZE/2){
                        x @ Some(_) => x.unwrap(),
                        None => 0,
                    },
                    y: match GRID_OFFSET.1.checked_sub(CASE_SIZE/2){
                        x @ Some(_) => x.unwrap(),
                        None => 0,
                    },
                    width: (MAX_WIDTH + 1)*CASE_SIZE,
                    height: (MAX_HEIGHT + 1)*CASE_SIZE,
                },
                Color::BLACK,
            );
        }
        for x in 0..(MAX_WIDTH) {
            for y in 0..(MAX_HEIGHT) {
                draw_terrain_box(x, y);
            }
        }
    }
}
