use crate::{
    eadk::{
        self,
        display::{self, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, timing, Point,
    },
    menu::{menu, selection_menu, MenuConfig, MyOption, OptionType},
    utils::{
        draw_centered_string, fading, randint, wait_for_no_keydown,
        ColorConfig, CENTER, LARGE_CHAR_HEIGHT,
    }, snake_ui::{CASE_SIZE, BCKD_GRAY, DARK_GREEN, draw_terrain, draw_snake, draw_fruit, draw_wall, draw_box, draw_terrain_box, draw_snake_front},
};
use eadk::Color;
use heapless::String;
use heapless::{Deque, Vec};

/// Used to get directions, and nothing else !
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

/// The max y value on the grid
/// - 1 because the grid starts at (0, 0)
pub static mut MAX_HEIGHT: u16 = SCREEN_HEIGHT / CASE_SIZE - 1;
/// THe max x value on the grid
pub static mut MAX_WIDTH: u16 = SCREEN_WIDTH / CASE_SIZE - 1;
/// The offset of the grid
pub static mut GRID_OFFSET: (u16, u16) = (CASE_SIZE / 2, CASE_SIZE / 2);
/// No array in this program can exceed this size ; it can be used to set the size.
pub const MAX_ARRAY_SIZE: usize =
    ((SCREEN_HEIGHT / CASE_SIZE - 1) * SCREEN_WIDTH / CASE_SIZE - 1) as usize;
/// The [ColorConfig] to use for most of the text in menus etc
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: BCKD_GRAY,
    alt: DARK_GREEN,
};



/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut MyOption; 5] = [
        &mut MyOption {
            name: "Speed :\0",
            value: 1,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((OptionType::Int(3), "Fast\0")) };
                unsafe { v.push_unchecked((OptionType::Int(2), "Normal\0")) };
                unsafe { v.push_unchecked((OptionType::Int(1), "Slow\0")) };
                v
            },
        },
        &mut MyOption {
            name: "Terrain :\0",
            value: 1,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((OptionType::Int(1), "Small\0")) };
                unsafe { v.push_unchecked((OptionType::Int(2), "Medium\0")) };
                unsafe { v.push_unchecked((OptionType::Int(3), "Immense\0")) };
                v
            },
        },
        &mut MyOption {
            name: "Walls :\0",
            value: 0,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((OptionType::Bool(true), "Yes\0")) };
                unsafe { v.push_unchecked((OptionType::Bool(false), "No\0")) };
                v
            },
        },
        &mut MyOption {
            name: "Wrapping :\0",
            value: 1,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((OptionType::Bool(true), "Yes\0")) };
                unsafe { v.push_unchecked((OptionType::Bool(false), "No\0")) };
                v
            },
        },
        &mut MyOption {
            name: "Sprites :\0",
            value: 0,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((OptionType::Bool(false), "Yes\0")) };
                unsafe { v.push_unchecked((OptionType::Bool(true), "No\0")) };
                v
            },
        },
    ];
    loop {
        let start = menu("SNAKE\0", &mut opt, &COLOR_CONFIG, crate::snake_ui::menu_vis_addon, include_str!("./data/snake_controls.txt"));
        if start == 0 {
            match opt[1].get_param_value() {
                1 => unsafe {
                    MAX_HEIGHT = 11;
                    MAX_WIDTH = 16;
                },
                2 => unsafe {
                    MAX_HEIGHT = 15;
                    MAX_WIDTH = 24;
                },
                _ => unsafe {
                    MAX_HEIGHT = SCREEN_HEIGHT / CASE_SIZE - 1;
                    MAX_WIDTH = SCREEN_WIDTH / CASE_SIZE - 1;
                },
            }
            unsafe {
                GRID_OFFSET = (
                    CENTER.x - (MAX_WIDTH * CASE_SIZE) / 2,
                    CENTER.y - (MAX_HEIGHT * CASE_SIZE) / 2,
                )
            }
            loop {
                let action = game(
                    325 - opt[0].get_param_value::<u16>() * 75,
                    opt[2].get_param_value(),
                    opt[3].get_param_value(),
                    opt[4].get_param_value(),
                );
                if action == 2 {
                    return;
                } else if action == 1 {
                    break;
                }
            }
        } else {
            return;
        }
    }
}



/// The entire game is here.
pub fn game(speed: u16, has_walls: bool, wrapping: bool, original: bool) -> u8 {
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

    display::wait_for_vblank();
    draw_terrain(wrapping);
    draw_snake(&snake, direction, original);
    draw_fruit(fruit_pos.x, fruit_pos.y, original);
    display::wait_for_vblank();
    wait_for_no_keydown();
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
        } else if keyboard_state.key_down(key::OK) {
            let action = snake_pause(points, false);
            if action == 0 {
                display::wait_for_vblank();
                draw_terrain(wrapping);
                draw_snake(&snake, direction, original);
                draw_fruit(fruit_pos.x, fruit_pos.y, original);
                for i in &walls {
                    draw_wall(i.x, i.y, original);
                }
                display::wait_for_vblank();
            } else {
                return action;
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
                match get_point_from_dir(&front_point, &direction, &snake, &walls, wrapping) {
                    // renvoie None si le nouveau point n'existe pas, est un mur ou une partie du snake (derniere partie exclue)
                    x @ Some(_) => {
                        new_point = x.unwrap();
                        snake.push_front_unchecked(new_point);
                    }
                    None => break, // You lose !
                }
                draw_box(front_point.x, front_point.y, DARK_GREEN);
            }
            if (new_point.x == fruit_pos.x) && (new_point.y == fruit_pos.y) {
                // if we ate the fruit
                points += 1;
                let next_direct_pos: Point; // Evite que les murs ou les fruits apparaissent directement devant le joueur
                match get_point_from_dir(&new_point, &direction, &snake, &walls, wrapping) {
                    x @ Some(_) => next_direct_pos = x.unwrap(),
                    None => unsafe { next_direct_pos = Point::new(MAX_WIDTH, MAX_HEIGHT) },
                }
                if has_walls && ((points % 2) == 0) {
                    // add walls
                    let mut new_wall = get_random_point();
                    while is_in_walls(&new_wall, &walls)
                        || is_in_snake(&new_wall, &snake)
                        || ((new_wall.x == next_direct_pos.x) && (new_wall.y == next_direct_pos.y))
                    {
                        new_wall = get_random_point();
                    }
                    unsafe {
                        walls.push_unchecked(new_wall);
                    }
                    draw_wall(new_wall.x, new_wall.y, original);
                }
                fruit_pos = get_random_point();
                while is_in_snake(&fruit_pos, &snake)
                    || is_in_walls(&fruit_pos, &walls)
                    || ((fruit_pos.x == next_direct_pos.x) && (fruit_pos.y == next_direct_pos.y))
                {
                    fruit_pos = get_random_point();
                }
                draw_fruit(fruit_pos.x, fruit_pos.y, original);
            } else {
                // if we didn't eat the fruit, then we get rid of the last case
                let last_point = snake.pop_back().unwrap();
                draw_terrain_box(last_point.x, last_point.y);
            }
            display::wait_for_vblank();
            draw_snake_front(new_point.x, new_point.y, direction, original);
            //draw_box(new_point.x, new_point.y, LIGHT_GREEN);
            last_move = timing::millis();
        }
    }
    draw_centered_string(
        " GAME OVER ! \0",
        SCREEN_HEIGHT / 3 - LARGE_CHAR_HEIGHT,
        true,
        &ColorConfig {
            text: Color::BLACK,
            bckgrd: BCKD_GRAY,
            alt: Color::RED,
        },
        true,
    );
    snake_pause(points, true)
}

fn snake_pause(points: u16, death: bool) -> u8 {
    let mut string_points: String<15> = String::from(" Points : ");
    string_points
        .push_str(String::<15>::from(points).as_str())
        .unwrap();
    string_points.push_str(" \0").unwrap();
    draw_centered_string(&string_points, 5, true, &COLOR_CONFIG, false);
    let action = selection_menu(
        &COLOR_CONFIG,
        &MenuConfig {
            choices: if death {
                &["Play again\0", "Menu\0", "Exit\0"]
            } else {
                &["Resume\0", "Menu\0", "Exit\0"]
            },
            rect_margins: (20, 12),
            dimensions: (SCREEN_WIDTH * 7 / 15, LARGE_CHAR_HEIGHT * 7),
            offset: (0, if death { 50 } else { 0 }),
            back_key_return: if death { 2 } else { 1 },
        },
        false,
    );
    if action != 0 {
        fading(500);
    }
    return action;
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
        if (i.x == p.x) && (i.y == p.y) {
            return true;
        }
    }
    return false;
}

/// Check if a given Point is in a Vec (the walls)
fn is_in_walls(p: &Point, walls: &Vec<Point, MAX_ARRAY_SIZE>) -> bool {
    for i in walls {
        if (i.x == p.x) && (i.y == p.y) {
            return true;
        }
    }
    return false;
}

/// Give a random Point, corresponding to a position on the game grid
fn get_random_point() -> Point {
    unsafe {
        let x = randint(0, (MAX_WIDTH - 1) as u32) as u16;
        let y = randint(0, (MAX_HEIGHT - 1) as u32) as u16;
        return Point::new(x, y);
    }
}

/// Return a Point from a Point and a Direction, None if the Point is non-valid
fn get_point_from_dir(
    p: &Point,
    d: &Direction,
    snake: &Deque<Point, MAX_ARRAY_SIZE>,
    walls: &Vec<Point, MAX_ARRAY_SIZE>,
    wrapping: bool,
) -> Option<Point> {
    let mut new_point = Point::new(p.x, p.y);
    match d {
        // Big boom here ! x and y can not be < 0 !! (or too big for that matter)
        Direction::UP => {
            if new_point.y >= 1 {
                new_point.y -= 1
            } else if wrapping {
                unsafe { new_point.y = MAX_HEIGHT - 1 }
            } else {
                return None;
            }
        }
        Direction::DOWN => unsafe {
            if new_point.y < MAX_HEIGHT - 1 {
                new_point.y += 1
            } else if wrapping {
                new_point.y = 0
            } else {
                return None;
            }
        },
        Direction::LEFT => {
            if new_point.x >= 1 {
                new_point.x -= 1
            } else if wrapping {
                unsafe { new_point.x = MAX_WIDTH - 1 }
            } else {
                return None;
            }
        }
        Direction::RIGHT => unsafe {
            if new_point.x < MAX_WIDTH - 1 {
                new_point.x += 1
            } else if wrapping {
                new_point.x = 0
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








