use crate::snake_ui::{
    draw_box, draw_fruit, draw_snake, draw_snake_front, draw_terrain, draw_terrain_box, draw_wall,
    BCKD_GRAY, CASE_SIZE, DARK_GREEN,
};

use heapless::{Deque, String, Vec};
use numworks_utils::{
    eadk::{
        display::{self, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, timing, Color, Point,
    },
    graphical::{draw_centered_string, fading, ColorConfig},
    menu::{
        selection,
        settings::{write_values_to_file, Setting},
        start_menu, MenuConfig,
    },
    utils::{randint, string_from_u16, wait_for_no_keydown, CENTER, LARGE_CHAR_HEIGHT},
};

/// Used to get directions, and nothing else !
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// The offset of the grid. Using static because don't have the courage to do an other way...
pub static mut GRID_OFFSET: (u16, u16) = (CASE_SIZE / 2, CASE_SIZE / 2);

/// Max array size : width * height (the grid)
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
    let mut opt: [&mut Setting; 6] = [
        &mut Setting {
            name: "Speed\0",
            choice: 1,
            values: Vec::from_slice(&[3, 2, 1]).unwrap(),
            texts: Vec::from_slice(&["Fast\0", "Normal\0", "Slow\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "Terrain\0",
            choice: 1,
            values: Vec::from_slice(&[1, 2, 3]).unwrap(),
            texts: Vec::from_slice(&["Small\0", "Medium\0", "Immense\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "Walls\0",
            choice: 0,
            values: Vec::from_slice(&[1, 0]).unwrap(),
            texts: Vec::from_slice(&["Yes\0", "No\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "Wrapping\0",
            choice: 1,
            values: Vec::from_slice(&[1, 0]).unwrap(),
            texts: Vec::from_slice(&["Yes\0", "No\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "Sprites\0",
            choice: 0,
            values: Vec::from_slice(&[0, 1]).unwrap(),
            texts: Vec::from_slice(&["Yes\0", "No\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "High Score\0",
            choice: 0,
            values: Vec::from_slice(&[0, 0, u32::MAX]).unwrap(),
            texts: Vec::new(),
            user_modifiable: false,
            fixed_values: false,
        },
    ];
    loop {
        let start = start_menu(
            "SNAKE\0",
            &mut opt,
            &COLOR_CONFIG,
            crate::snake_ui::menu_vis_addon,
            include_str!("./data/snake_controls.txt"),
            "snake",
        );
        if start == 0 {
            let (height, width) = match opt[1].get_setting_value() {
                1 => (11, 16),
                2 => (15, 24),
                _ => (SCREEN_HEIGHT / CASE_SIZE - 1, SCREEN_WIDTH / CASE_SIZE - 1),
            };
            unsafe {
                GRID_OFFSET = (
                    CENTER.x - (width * CASE_SIZE) / 2,
                    CENTER.y - (height * CASE_SIZE) / 2,
                )
            }
            loop {
                let mut high_score = opt[5].get_setting_value();
                let action = game(
                    325 - opt[0].get_setting_value() as u16 * 75,
                    opt[2].get_setting_value() != 0,
                    opt[3].get_setting_value() != 0,
                    opt[4].get_setting_value() != 0,
                    height,
                    width,
                    &mut high_score,
                );
                // high_score saving
                opt[5].set_value(high_score);
                write_values_to_file(&mut opt, "snake");

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
pub fn game(
    speed: u16,
    has_walls: bool,
    wrapping: bool,
    original: bool,
    height: u16,
    width: u16,
    high_score: &mut u32,
) -> u8 {
    let random_point = || get_random_point(width, height);
    // This mutable variables are the heart of everything. Really.
    // Not the prettiest way, but works well enough for a simple game.
    let mut snake: Deque<Point, MAX_ARRAY_SIZE> = Deque::new();
    unsafe {
        snake.push_front_unchecked(Point::new(1, 1));
        snake.push_front_unchecked(Point::new(2, 1));
        snake.push_front_unchecked(Point::new(3, 1));
    }
    let mut last_move: u64 = timing::millis();
    let mut direction: Direction = Direction::Right;
    let mut last_direction: Direction = direction;
    let mut fruit_pos: Point = random_point();
    while is_in_snake(&fruit_pos, &snake) {
        fruit_pos = random_point();
    }
    let mut walls: Vec<Point, MAX_ARRAY_SIZE> = Vec::new();
    let mut score: u16 = 0;

    display::wait_for_vblank();
    draw_terrain(wrapping, width, height);
    draw_snake(&snake, direction, original);
    draw_fruit(fruit_pos.x, fruit_pos.y, original);
    display::wait_for_vblank();
    wait_for_no_keydown();
    timing::msleep(300);
    loop {
        let keyboard_state = keyboard::scan();
        // very efficient scan, register most of the input, do any verification every move only.
        if keyboard_state.key_down(key::UP) {
            direction = Direction::Up;
        } else if keyboard_state.key_down(key::DOWN) {
            direction = Direction::Down;
        } else if keyboard_state.key_down(key::RIGHT) {
            direction = Direction::Right;
        } else if keyboard_state.key_down(key::LEFT) {
            direction = Direction::Left;
        } else if keyboard_state.key_down(key::OK) {
            let action = snake_pause(score, false, high_score);
            if action == 0 {
                display::wait_for_vblank();
                draw_terrain(wrapping, width, height);
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
                front_point = *snake.front().unwrap();
                match get_point_from_dir(
                    &front_point,
                    &direction,
                    &snake,
                    &walls,
                    wrapping,
                    height,
                    width,
                ) {
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
                score += 1;
                let next_direct_pos: Point = // Evite que les murs ou les fruits apparaissent directement devant le joueur
                match get_point_from_dir(&new_point, &direction, &snake, &walls, wrapping, height, width) {
                    x @ Some(_) =>  x.unwrap(),
                    None =>  Point::new(width, height),
                };
                if has_walls && ((score % 2) == 0) {
                    // add walls
                    let mut new_wall = random_point();
                    while is_in_walls(&new_wall, &walls)
                        || is_in_snake(&new_wall, &snake)
                        || ((new_wall.x == next_direct_pos.x) && (new_wall.y == next_direct_pos.y))
                    {
                        new_wall = random_point();
                    }
                    unsafe {
                        walls.push_unchecked(new_wall);
                    }
                    draw_wall(new_wall.x, new_wall.y, original);
                }
                fruit_pos = random_point();
                while is_in_snake(&fruit_pos, &snake)
                    || is_in_walls(&fruit_pos, &walls)
                    || ((fruit_pos.x == next_direct_pos.x) && (fruit_pos.y == next_direct_pos.y))
                {
                    fruit_pos = random_point();
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
        " GAME OVER! \0",
        SCREEN_HEIGHT / 3 - LARGE_CHAR_HEIGHT,
        true,
        &ColorConfig {
            text: Color::BLACK,
            bckgrd: BCKD_GRAY,
            alt: Color::RED,
        },
        true,
    );
    if score > *high_score as u16 {
        draw_centered_string(
            "NEW HIGH SCORE!\0",
            SCREEN_HEIGHT / 3 + 2,
            true,
            &ColorConfig {
                text: Color::BLACK,
                bckgrd: BCKD_GRAY,
                alt: Color::RED,
            },
            true,
        );
        *high_score = score as u32;
    }
    snake_pause(score, true, high_score)
}

fn snake_pause(points: u16, death: bool, high_score: &u32) -> u8 {
    let mut string_points: String<15> = String::new();
    string_points.push_str(" Score : ").unwrap();
    string_points
        .push_str(
            string_from_u16(points)
                .as_str()
                .split_terminator('\0')
                .next()
                .unwrap(),
        )
        .unwrap();
    string_points.push_str(" \0").unwrap();
    draw_centered_string(&string_points, 5, true, &COLOR_CONFIG, false);
    let mut string_high_score: String<20> = String::new();
    string_high_score.push_str(" High Score : ").unwrap();
    string_high_score
        .push_str(
            string_from_u16(*high_score as u16)
                .as_str()
                .split_terminator('\0')
                .next()
                .unwrap(),
        )
        .unwrap();
    string_high_score.push_str(" \0").unwrap();
    draw_centered_string(
        &string_high_score,
        5 + LARGE_CHAR_HEIGHT + 2,
        true,
        &COLOR_CONFIG,
        false,
    );
    let action = selection(
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
    action
}

/// Just checks if the new direction isn't the opposite of the last one (as the snake can't go back on itself)
fn check_direction(direction: Direction, last_direction: Direction) -> Direction {
    match direction {
        Direction::Up => {
            if last_direction == Direction::Down {
                return last_direction;
            }
        }
        Direction::Down => {
            if last_direction == Direction::Up {
                return last_direction;
            }
        }
        Direction::Left => {
            if last_direction == Direction::Right {
                return last_direction;
            }
        }
        Direction::Right => {
            if last_direction == Direction::Left {
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
    false
}

/// Check if a given Point is in a Vec (the walls)
fn is_in_walls(p: &Point, walls: &Vec<Point, MAX_ARRAY_SIZE>) -> bool {
    for i in walls {
        if (i.x == p.x) && (i.y == p.y) {
            return true;
        }
    }
    false
}

/// Give a random Point, corresponding to a position on the game grid
fn get_random_point(width: u16, height: u16) -> Point {
    let x = randint(0, (width - 1) as u32) as u16;
    let y = randint(0, (height - 1) as u32) as u16;
    Point::new(x, y)
}

/// Return a Point from a Point and a Direction, None if the Point is non-valid
fn get_point_from_dir(
    p: &Point,
    d: &Direction,
    snake: &Deque<Point, MAX_ARRAY_SIZE>,
    walls: &Vec<Point, MAX_ARRAY_SIZE>,
    wrapping: bool,
    height: u16,
    width: u16,
) -> Option<Point> {
    let mut new_point = Point::new(p.x, p.y);
    match d {
        // Big boom here ! x and y can not be < 0 !! (or too big for that matter)
        Direction::Up => {
            if new_point.y >= 1 {
                new_point.y -= 1
            } else if wrapping {
                new_point.y = height - 1
            } else {
                return None;
            }
        }
        Direction::Down => {
            if new_point.y < height - 1 {
                new_point.y += 1
            } else if wrapping {
                new_point.y = 0
            } else {
                return None;
            }
        }
        Direction::Left => {
            if new_point.x >= 1 {
                new_point.x -= 1
            } else if wrapping {
                new_point.x = width - 1
            } else {
                return None;
            }
        }
        Direction::Right => {
            if new_point.x < width - 1 {
                new_point.x += 1
            } else if wrapping {
                new_point.x = 0
            } else {
                return None;
            }
        }
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
