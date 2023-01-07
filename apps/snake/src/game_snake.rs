use crate::{
    eadk::{
        self,
        display::{self, push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, timing, Point, Rect,
    },
    menu::{menu, selection_menu, MenuConfig, MyOption},
    utils::{
        draw_centered_string, draw_image_from_tilemap, fading, fill_screen, randint,
        wait_for_no_keydown, ColorConfig, CENTER, LARGE_CHAR_HEIGHT,
    },
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
pub const BOOL_OPTIONS_NUMBER: usize = 4;

const DARK_GRAY: Color = Color::from_rgb888(60, 60, 80);
const GRAY: Color = Color::from_rgb888(175, 175, 175);
const BCKD_GRAY: Color = Color::from_rgb888(200, 200, 200);
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
static mut GRID_OFFSET: (u16, u16) = (CASE_SIZE / 2, CASE_SIZE / 2);
/// No array in this program can exceed this size ; it can be used to set the size.
const MAX_ARRAY_SIZE: usize =
    ((SCREEN_HEIGHT / CASE_SIZE - 1) * SCREEN_WIDTH / CASE_SIZE - 1) as usize;
/// The [ColorConfig] to use for most of the text in menus etc
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: BCKD_GRAY,
    alt: DARK_GREEN,
};

const MENU_FIGURE_Y: u16 = 70;
/// Menu Visual Addon
fn menu_vis_addon() {
    draw_image_from_tilemap(&TILEMAP, CENTER.x - CASE_SIZE, MENU_FIGURE_Y, 10, 10, 2, 30, 0);
    push_rect_uniform(
        Rect {
            x: CENTER.x - CASE_SIZE * 7,
            y: MENU_FIGURE_Y,
            width: CASE_SIZE * 6,
            height: CASE_SIZE * 2,
        },
        DARK_GREEN,
    );
    draw_image_from_tilemap(&TILEMAP, CENTER.x + CASE_SIZE*3, MENU_FIGURE_Y, 10, 10, 2, 0, 0);
}

/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut MyOption<bool, 2>; BOOL_OPTIONS_NUMBER] = [
        &mut MyOption {
            name: "HYPER RAPIDE !\0",
            value: 0,
            possible_values: [(true, "Oui !\0"), (false, "Non !\0")],
        },
        &mut MyOption {
            name: "MINI TERRAIN !\0",
            value: 0,
            possible_values: [(true, "Oui !\0"), (false, "Non !\0")],
        },
        &mut MyOption {
            name: "OBSTACLES !\0",
            value: 0,
            possible_values: [(true, "Oui !\0"), (false, "Non !\0")],
        },
        &mut MyOption {
            name: "Version originale\0",
            value: 0,
            possible_values: [(true, "Oui !\0"), (false, "Non !\0")],
        },
    ];
    loop {
        let start = menu("SNAKE\0", &mut opt, &COLOR_CONFIG, menu_vis_addon);
        if start == 1 {
            if opt[1].get_value().0 {
                unsafe {
                    MAX_HEIGHT = 11;
                    MAX_WIDTH = 16;
                    GRID_OFFSET = (
                        CENTER.x - (MAX_WIDTH * CASE_SIZE) / 2,
                        CENTER.y - (MAX_HEIGHT * CASE_SIZE) / 2,
                    )
                }
            } else {
                unsafe {
                    MAX_HEIGHT = SCREEN_HEIGHT / CASE_SIZE - 1;
                    MAX_WIDTH = SCREEN_WIDTH / CASE_SIZE - 1;
                    GRID_OFFSET = (CASE_SIZE / 2, CASE_SIZE / 2);
                }
            }
            loop {
                let action = game(
                    if opt[0].get_value().0 { 100 } else { 250 },
                    opt[2].get_value().0,
                    opt[3].get_value().0,
                );
                if action == 0 {
                    return;
                } else if action == 2 {
                    break;
                }
            }
        } else {
            return;
        }
    }
}

static TILEMAP: [u8; 1813] = *include_bytes!("./tiles.ppm");

/// The entire game is here.
pub fn game(speed: u16, has_walls: bool, original: bool) -> u8 {
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

    draw_terrain();
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
            if action == 1 {
                draw_terrain();
                draw_snake(&snake, direction, original);
                draw_fruit(fruit_pos.x, fruit_pos.y, original);
                for i in &walls {
                    draw_box(i.x, i.y, Color::BLACK);
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
                if has_walls & ((points % 2) == 0) {
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
                draw_fruit(fruit_pos.x, fruit_pos.y, original);
            } else {
                // if we didn't eat the fruit, then we get rid of the last case
                let last_point = snake.pop_back().unwrap();
                draw_terrain_box(last_point.x, last_point.y);
            }
            draw_snake_front(new_point.x, new_point.y, direction, original);
            //draw_box(new_point.x, new_point.y, LIGHT_GREEN);
            display::wait_for_vblank();
            last_move = timing::millis();
        }
    }
    draw_centered_string(
        " Perdu ! \0",
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
            first_choice: if death { "Play again\0" } else { "Resume\0" },
            second_choice: "Menu\0",
            null_choice: "Exit\0",
            rect_margins: (20, 12),
            dimensions: (SCREEN_WIDTH * 7 / 15, LARGE_CHAR_HEIGHT * 7),
            offset: (0, if death { 50 } else { 0 }),
            back_key_return: if death { 2 } else { 1 },
        },
        false,
    );
    if action != 1 {
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
            if new_point.y < MAX_HEIGHT - 1 {
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
            if new_point.x < MAX_WIDTH - 1 {
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

fn draw_fruit(x: u16, y: u16, original: bool) {
    if original {
        draw_box(x, y, Color::RED);
    } else {
        unsafe {
            draw_image_from_tilemap(
                &TILEMAP,
                x * CASE_SIZE + GRID_OFFSET.0,
                y * CASE_SIZE + GRID_OFFSET.1,
                CASE_SIZE,
                CASE_SIZE,
                1,
                0,
                0,
            );
        }
    }
}
fn draw_snake_front(x:u16, y: u16, direction : Direction, original: bool){
    if !original {
        draw_image_from_tilemap(
            &TILEMAP,
            unsafe { x * CASE_SIZE + GRID_OFFSET.0 },
            unsafe { y * CASE_SIZE + GRID_OFFSET.1 },
            CASE_SIZE,
            CASE_SIZE,
            1,
            CASE_SIZE*{match direction {
                Direction::UP => 1,
                Direction::DOWN => 2,
                Direction::LEFT => 4,
                Direction::RIGHT => 3,
            }},
            0,
        );
    } else {
        draw_box(x, y, LIGHT_GREEN);
    }
}

/// Draws the snake !
fn draw_snake(snake: &Deque<Point, MAX_ARRAY_SIZE>, direction : Direction, original: bool) {
    if original {
        for i in snake {
            if (i.x == snake.front().unwrap().x) & (i.y == snake.front().unwrap().y) {
                draw_snake_front(i.x, i.y, direction, original)
            } else {
                draw_box(i.x, i.y, DARK_GREEN);
            }
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
    fill_screen(DARK_GRAY);
    unsafe {
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
                width: (MAX_WIDTH + 1) * CASE_SIZE,
                height: (MAX_HEIGHT + 1) * CASE_SIZE,
            },
            Color::BLACK,
        );
        push_rect_uniform(
            Rect {
                x: GRID_OFFSET.0,
                y: GRID_OFFSET.1,
                width: MAX_WIDTH * CASE_SIZE,
                height: MAX_HEIGHT * CASE_SIZE,
            },
            LIGHT_GRAY,
        );
        for x in (1..(MAX_WIDTH)).step_by(2) {
            for y in (0..(MAX_HEIGHT)).step_by(2) {
                draw_box(x, y, GRAY)
            }
        }
        for x in (0..MAX_WIDTH).step_by(2) {
            for y in (1..MAX_HEIGHT).step_by(2) {
                draw_box(x, y, GRAY)
            }
        }
    }
    //display::wait_for_vblank();
}
