use heapless::Vec;

use crate::{
    eadk::{
        display::{self},
        key, keyboard, timing, Color,
    },
    menu::{menu, pause_menu, MyOption},
    tetris::tetriminos::{get_random_bag, Tetrimino},
    tetris::ui_tetris::{
        debug_check, draw_block, draw_held_tetrimino, draw_level, draw_line, draw_lines_number,
        draw_next_tetrimino, draw_score, draw_stable_ui, draw_tetrimino,
    },
    utils::{draw_centered_string, ColorConfig},
};

/// The number of Boolean Options used. Public so menu() can use it.
pub const BOOL_OPTIONS_NUMBER: usize = 1;

// This dictates the principal colors that will be used
pub const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::from_rgb888(251, 251, 219),
    bckgrd: Color::from_rgb888(10, 10, 10),
    alt: Color::RED,
};

pub const BACKGROUND_GRAY: Color = Color::from_rgb888(100, 100, 100);
pub const BACKGROUND_DARK_GRAY: Color = Color::from_rgb888(70, 70, 70);

static mut EXEMPLE: bool = false;

fn vis_addon() {
    draw_centered_string("VIS ADDON\0", 70, true, &COLOR_CONFIG, true);
}
/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut MyOption<bool, 2>; BOOL_OPTIONS_NUMBER] = [&mut MyOption {
        name: "Option !\0",
        value: 0,
        possible_values: [(true, "True\0"), (false, "False\0")],
    }];
    loop {
        let start = menu("TETRIS\0", &mut opt, &COLOR_CONFIG, vis_addon); // The menu does everything itself !
        if start == 1 {
            unsafe {
                EXEMPLE = opt[0].get_value().0; // You could use mutable statics, but it is not very good
            }
            loop {
                // a loop where the game is played again and again, which means it should be 100% contained after the menu
                let action = game(); // calling the game based on the parameters is better
                if action == 0 {
                    // 0 means quitting
                    return;
                } else if action == 2 {
                    // 2 means back to menu
                    break;
                } // if action == 1 : rejouer
            }
        } else {
            return;
        }
    }
}

/// Represents the game grid, every case being a [Color] or [None]
struct Grid {
    grid: [[Option<Color>; (PLAYFIELD_HEIGHT as usize)]; (PLAYFIELD_WIDTH as usize)],
}

impl Grid {
    // Does the grid initialisation for you
    fn new() -> Self {
        return Self {
            grid: [[None; (PLAYFIELD_HEIGHT as usize)]; (PLAYFIELD_WIDTH as usize)],
        };
    }

    /// Returns the color at the given position, None if the pos is outside the grid
    fn get_color_at(&self, x: i16, y: i16) -> Option<Color> {
        if (x < 0) | (y < 0) {
            return None;
        } else if (x as u16 >= PLAYFIELD_WIDTH) | (y as u16 >= PLAYFIELD_HEIGHT) {
            return None;
        } else {
            return self.grid[x as usize][y as usize];
        }
    }

    /// Set the color at the given position, if the position is in the grid
    fn set_color_at(&mut self, x: i16, y: i16, c: Color) {
        if (x >= 0) & (y >= 0) & (x < PLAYFIELD_WIDTH as i16) & (y < PLAYFIELD_HEIGHT as i16) {
            self.grid[x as usize][y as usize] = Some(c);
            draw_block(x as u16, y as u16, c);
        }
    }

    /// Set the case at the given position to None, if the position is in the grid
    fn remove_color_at(&mut self, x: i16, y: i16) {
        if (x >= 0) & (y >= 0) & (x < PLAYFIELD_WIDTH as i16) & (y < PLAYFIELD_HEIGHT as i16) {
            self.grid[x as usize][y as usize] = None;
            draw_block(x as u16, y as u16, COLOR_CONFIG.bckgrd)
        }
    }

    fn remove_line(&mut self, y: i16) {
        if (y >= 0) & (y < PLAYFIELD_HEIGHT as i16) {
            for x in 0..PLAYFIELD_WIDTH {
                self.remove_color_at(x as i16, y);
            }
            draw_line(y as u16, COLOR_CONFIG.bckgrd);
        }
    }
}

pub const HIGH_SCORE: &'static str = "000000\0"; // Need to be 6 char long !
pub const CASE_SIZE: u16 = 10;
pub const PLAYFIELD_HEIGHT: u16 = 20;
pub const PLAYFIELD_WIDTH: u16 = 10;

const ROTATE_SPEED: u64 = 150;
const DELAYED_AUTO_SHIFT: u64 = 167;
const AUTO_MOVE_SPEED: u64 = 33;
const SOFT_DROP_SPEED: u64 = 33;

/// The entire game is here.
pub fn game() -> u8 {
    draw_stable_ui();

    // Is it possible to not have all those variables ? Maybe some struct ?
    let mut fall_speed: u64 = 500; // TODO: decrease with level increase
    let mut last_fall_time: u64 = timing::millis();
    let mut last_move_time: u64 = timing::millis();
    let mut move_button_down: bool = false;
    let mut auto_repeat_on: bool = false;
    let mut last_rotate_time: u64 = timing::millis();
    let mut rotate_button_down: bool = false;
    let mut soft_drop_button_down: bool = false;

    let mut grid: Grid = Grid::new();
    let mut score: u32 = 0;
    let mut level: u16 = 0;
    let mut level_lines: u16 = 0;

    let mut random_bag: Vec<Tetrimino, 7> = get_random_bag();

    let mut actual_tetri: Tetrimino = random_bag.swap_remove(0);
    let mut next_tetri: Tetrimino = random_bag.swap_remove(0);

    let mut held_tetri: Option<Tetrimino> = None;
    let mut held_blocked: bool = false;
    let mut held_button_down: bool = false;

    draw_tetrimino(&actual_tetri, false);
    draw_next_tetrimino(&next_tetri);

    loop {
        let keyboard_state = keyboard::scan();
        if (!move_button_down | ((last_move_time + AUTO_MOVE_SPEED < timing::millis()) & auto_repeat_on)) // if we touch the button for the first time in this frame, or if we maintained it pressed and some time has passed
            & (keyboard_state.key_down(key::LEFT) | keyboard_state.key_down(key::RIGHT))
        {
            // MOVE
            let direction: i16;
            if keyboard_state.key_down(key::LEFT) {
                direction = -1
            } else {
                direction = 1
            }
            if can_move(&actual_tetri, (direction, 0), &grid) {
                draw_tetrimino(&actual_tetri, true);
                actual_tetri.pos.x += direction;
                draw_tetrimino(&actual_tetri, false);
                last_move_time = timing::millis();
                move_button_down = true;
            }
        } else if (!rotate_button_down | (last_rotate_time + ROTATE_SPEED < timing::millis()))
            & keyboard_state.key_down(key::OK)
        {
            // ROTATE
            if can_rotate_left(&actual_tetri, &grid) {
                draw_tetrimino(&actual_tetri, true);
                actual_tetri.rotate_left();
                draw_tetrimino(&actual_tetri, false);
                last_rotate_time = timing::millis();
                rotate_button_down = true;
            }else{
                // TODO : kicks (verification if possible rotation first, then tests with differents kicks)
            }
        } else if !held_button_down & !held_blocked & keyboard_state.key_down(key::BACK) {
            // HOLD
            held_blocked = true;
            held_button_down = true;
            let temp = actual_tetri.clone();
            if held_tetri.is_some() { // Needs to check if position possible
                draw_tetrimino(&actual_tetri, true);
                actual_tetri = held_tetri.unwrap();
                actual_tetri.pos = temp.pos; // Needs to kick !!
                draw_tetrimino(&actual_tetri, false);
            } else {
                draw_tetrimino(&actual_tetri, true);
                if random_bag.len() == 0 {
                    random_bag = get_random_bag();
                }
                actual_tetri = next_tetri.clone();
                next_tetri = random_bag.swap_remove(0);
                draw_tetrimino(&actual_tetri, false);
                draw_next_tetrimino(&next_tetri);
            }
            held_tetri = Some(temp.clone());
            draw_held_tetrimino(&held_tetri.as_ref().unwrap());
        }
        if last_fall_time
            + if (!soft_drop_button_down & keyboard_state.key_down(key::DOWN))
                | (soft_drop_button_down)
            {
                SOFT_DROP_SPEED
            } else {
                fall_speed
            }
            < timing::millis()
        {
            // Place that somewhere else ? maybe doing the check differently too ?
            if !soft_drop_button_down {
                soft_drop_button_down = true;
            }
            // FALL
            let need_to_fall = can_move(&actual_tetri, (0, 1), &grid);
            if need_to_fall {
                draw_tetrimino(&actual_tetri, true);
                actual_tetri.pos.y += 1;
                draw_tetrimino(&actual_tetri, false);
            } else {
                for p in actual_tetri.get_blocks_grid_pos() {
                    grid.set_color_at(p.x, p.y, actual_tetri.color);
                }

                let clear_lines_y = get_clear_lines(&actual_tetri, &grid);
                if !clear_lines_y.is_empty() {
                    score = add_points(&clear_lines_y, level, score);

                    (level_lines, level) =
                        add_lines(clear_lines_y.len() as u16, level, level_lines);

                    // Removes every cleared line
                    for i in &clear_lines_y {
                        grid.remove_line(*i);
                    }

                    // BRINGS LINES DOWN (display done at the same time for optimisation) -> lol not optimised at all because drawing too much things
                    for (i, e) in clear_lines_y.iter().enumerate() {
                        for j in (0..*e + i as i16).rev() {
                            for x in 0..PLAYFIELD_WIDTH {
                                let last_color = grid.get_color_at(x as i16, j as i16);
                                if last_color.is_some() {
                                    grid.set_color_at(x as i16, j as i16 + 1, last_color.unwrap());
                                }
                            }
                            grid.remove_line(j as i16);
                        }
                    }
                }

                // new tetri because last one touched down
                if random_bag.len() == 0 {
                    random_bag = get_random_bag();
                }
                actual_tetri = next_tetri.clone();
                next_tetri = random_bag.swap_remove(0);
                draw_tetrimino(&actual_tetri, false);
                draw_next_tetrimino(&next_tetri);

                held_blocked = false;
            }
            last_fall_time = timing::millis();
        }

        if move_button_down
            & !(keyboard_state.key_down(key::LEFT) | keyboard_state.key_down(key::RIGHT))
        {
            move_button_down = false;
            auto_repeat_on = false;
        }
        if move_button_down & (last_move_time + DELAYED_AUTO_SHIFT < timing::millis()) {
            auto_repeat_on = true;
        }
        if rotate_button_down & !keyboard_state.key_down(key::OK) {
            rotate_button_down = false;
        }
        if soft_drop_button_down & !keyboard_state.key_down(key::DOWN) {
            soft_drop_button_down = false;
        }
        if keyboard_state.key_down(key::BACKSPACE) {
            // PAUSE MENU
            let action = pause_menu(&COLOR_CONFIG, 0);
            if action != 1 {
                return action;
            } else {
                draw_stable_ui();
                // redraws everything that needs it
                return action;
            }
        }
        display::wait_for_vblank();

        if held_button_down & !keyboard_state.key_down(key::BACK) {
            held_button_down = false;
        }
    }
}

/// Check if a line is complete or not.
fn check_line(p: i16, grid: &Grid) -> bool {
    for i in 0..PLAYFIELD_WIDTH {
        if grid.get_color_at(i as i16, p).is_none() {
            return false;
        }
    }
    return true;
}

/// Returns every completed line by the given tetrimino
fn get_clear_lines(tetri: &Tetrimino, grid: &Grid) -> Vec<i16, 4> {
    let mut clear_lines_y = Vec::<i16, 4>::new();
    for pos in tetri.get_blocks_grid_pos() {
        if check_line(pos.y, &grid) & !clear_lines_y.contains(&(pos.y)) {
            // get sorted index
            let mut new_index: usize = 0;
            for (i, e) in clear_lines_y.iter().enumerate() {
                if (pos.y) < *e {
                    new_index = i;
                    break;
                }
            }
            clear_lines_y.insert(new_index, pos.y).unwrap();
        }
    }
    clear_lines_y.reverse();
    return clear_lines_y;
}

/// Returns the number of points gained by clearing the given lines
fn add_points(cleared_lines: &Vec<i16, 4>, level: u16, points: u32) -> u32 {
    let mut sum: u32 = 0;

    if cleared_lines.len() == 1 {
        sum += 40
    } else if cleared_lines.len() == 4 {
        sum += 1200
    } else if cleared_lines.len() == 2 {
        if cleared_lines[0].abs_diff(cleared_lines[1]) == 1 {
            sum += 100
        } else {
            sum += 80
        }
    } else {
        // len == 3
        if (cleared_lines[0].abs_diff(cleared_lines[1]) == 1)
            & (cleared_lines[1].abs_diff(cleared_lines[2]) == 1)
        {
            sum += 300
        } else {
            sum += 140
        }
    }

    draw_score((sum * (level as u32 + 1)) + points);
    return (sum * (level as u32 + 1)) + points;
}

/// Returns true if the tetrimino can go to that direction.
fn can_move(future_tetri: &Tetrimino, direction: (i16, i16), grid: &Grid) -> bool {
    for pos in future_tetri.get_blocks_grid_pos() {
        if (pos.x + direction.0 < 0)
            | (pos.x + direction.0 > PLAYFIELD_WIDTH as i16 - 1)
            | (pos.y + direction.1 > PLAYFIELD_HEIGHT as i16 - 1)
        {
            return false;
        } else if grid
            .get_color_at(pos.x + direction.0, pos.y + direction.1)
            .is_some()
        {
            return false;
        }
    }
    return true;
}

/// Returns (line_number, level)
fn add_lines(number: u16, level: u16, current_lines: u16) -> (u16, u16) {
    let new_number = (current_lines + number) % 10;
    draw_lines_number(new_number);

    if current_lines + number >= 10 {
        draw_level(level + 1);
        return (new_number, level + 1);
    } else {
        return (new_number, level);
    }
}

fn can_rotate_left(tetri: &Tetrimino, grid: &Grid) -> bool {
    let mut rotated_tetri = tetri.clone();
    rotated_tetri.rotate_left();
    return can_move(&rotated_tetri, (0, 0), &grid);
}
