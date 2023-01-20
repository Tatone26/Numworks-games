use heapless::Vec;

use crate::{
    eadk::{
        display::{self, wait_for_vblank},
        key, keyboard, timing, Color,
    },
    menu::{menu, pause_menu, selection_menu, MenuConfig, MyOption, OptionType},
    tetriminos::{get_initial_tetri, get_random_bag, get_wall_kicks_data, Tetrimino},
    ui_tetris::{
        draw_blank_line, draw_block, draw_held_tetrimino, draw_level, draw_lines_number,
        draw_next_tetrimino, draw_score, draw_stable_ui, draw_tetrimino,
    },
    utils::{draw_centered_string, ColorConfig},
};

// This dictates the principal colors that will be used
pub const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::from_rgb888(251, 251, 219),
    bckgrd: Color::from_rgb888(20, 20, 20),
    alt: Color::RED,
};

pub const BACKGROUND_GRAY: Color = Color::from_rgb888(100, 100, 100);
pub const BACKGROUND_DARK_GRAY: Color = Color::from_rgb888(70, 70, 70);

fn vis_addon() {
    draw_centered_string("VIS ADDON\0", 70, true, &COLOR_CONFIG, true);
}
/// Menu, Options and Game start/*  */
pub fn start() {
    let mut opt: [&mut MyOption; 1] = [&mut MyOption {
        name: "Option !\0",
        value: 0,
        possible_values: {
            let mut v = Vec::new();
            unsafe { v.push_unchecked((OptionType::Bool(true), "True\0")) };
            unsafe { v.push_unchecked((OptionType::Bool(false), "False\0")) };
            v
        },
    }];
    loop {
        let start = menu(
            "TETRIS\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./data/tetris_controls.txt"),
        );
        // The menu does everything itself !
        if start == 0 {
            loop {
                // a loop where the game is played again and again, which means it should be 100% contained after the menu
                let action = game(); // calling the game based on the parameters is better
                if action == 2 {
                    // 2 means quitting
                    return;
                } else if action == 1 {
                    // 1 means back to menu
                    break;
                } // if action == 0 : rejouer
            }
        } else {
            return;
        }
    }
}

/// Represents the game grid, every case being a [Color] or [None]
struct Grid {
    grid: [[Option<u8>; (PLAYFIELD_HEIGHT as usize)]; (PLAYFIELD_WIDTH as usize)],
}

impl Grid {
    // Does the grid initialisation for you
    fn new() -> Self {
        return Self {
            grid: [[None; (PLAYFIELD_HEIGHT as usize)]; (PLAYFIELD_WIDTH as usize)],
        };
    }

    /// Returns the color at the given position, None if the pos is outside the grid
    fn get_color_at(&self, x: i16, y: i16) -> Option<u8> {
        if (x < 0) | (y < 0) {
            return None;
        } else if (x as u16 >= PLAYFIELD_WIDTH) | (y as u16 >= PLAYFIELD_HEIGHT) {
            return None;
        } else {
            return self.grid[x as usize][y as usize];
        }
    }

    /// Set the color at the given position, if the position is in the grid
    fn set_color_at(&mut self, x: i16, y: i16, c: u8) {
        if (x >= 0) & (y >= 0) & (x < PLAYFIELD_WIDTH as i16) & (y < PLAYFIELD_HEIGHT as i16) {
            self.grid[x as usize][y as usize] = Some(c);
            draw_block(x as u16, y as u16, c as u16);
        }
    }

    /// Set the case at the given position to None, if the position is in the grid
    fn remove_color_at(&mut self, x: i16, y: i16) {
        if (x >= 0) & (y >= 0) & (x < PLAYFIELD_WIDTH as i16) & (y < PLAYFIELD_HEIGHT as i16) {
            self.grid[x as usize][y as usize] = None;
            // draw_block(x as u16, y as u16, 7);
        }
    }

    fn remove_line(&mut self, y: i16) {
        if (y >= 0) & (y < PLAYFIELD_HEIGHT as i16) {
            for x in 0..PLAYFIELD_WIDTH {
                self.remove_color_at(x as i16, y);
            }
            // draw_blank_line(y as u16);
        }
    }
}

/// from tetris.wiki/Marathon, given in G (1/s)
static FALL_SPEED_DATA: [f32; 19] = [
    0.01667, 0.021017, 0.026977, 0.035256, 0.04693, 0.06361, 0.0879, 0.1236, 0.1775, 0.2598, 0.388,
    0.59, 0.92, 1.46, 2.36, 3.91, 6.61, 11.43, 20.0,
];

pub const HIGH_SCORE: &'static str = "031050\0"; // Need to be 6 char long !
pub const CASE_SIZE: u16 = 10;
pub const PLAYFIELD_HEIGHT: u16 = 20;
pub const PLAYFIELD_WIDTH: u16 = 10;

const ROTATE_SPEED: u64 = 150;
const DELAYED_AUTO_SHIFT: u64 = 167;
const AUTO_MOVE_SPEED: u64 = 33;
const SOFT_DROP_SPEED: u64 = 33;

const LEFT_KEY: u32 = key::LEFT;
const RIGHT_KEY: u32 = key::RIGHT;
const SOFT_DROP_KEY: u32 = key::DOWN;
const HARD_DROP_KEY: u32 = key::UP;
const PAUSE_KEY: u32 = key::SHIFT;
const RIGHT_ROTATION_KEY: u32 = key::BACK;
const LEFT_ROTATION_KEY: u32 = key::OK;
const HOLD_KEY: u32 = key::BACKSPACE;

const DEATH_MENU: MenuConfig = MenuConfig {
    choices: &["Replay\0", "Menu\0", "Exit\0"],
    rect_margins: (20, 10),
    dimensions: (CASE_SIZE * (PLAYFIELD_WIDTH + 2), CASE_SIZE * 10),
    offset: (0, 60),
    back_key_return: 2,
};
/// The entire game is here.
pub fn game() -> u8 {
    // Is it possible to not have all those variables ? Maybe some struct ?
    let mut last_fall_time: u64 = timing::millis();
    let mut last_move_time: u64 = timing::millis();
    let mut move_button_down: bool = false;
    let mut auto_repeat_on: bool = false;
    let mut last_rotate_time: u64 = timing::millis();
    let mut rotate_button_down: bool = false;
    let mut soft_drop_button_down: bool = false;

    let mut grid: Grid = Grid::new();
    let mut score: u32 = 0;
    let mut level: u16 = 1;
    let mut level_lines: u16 = 0;

    let mut random_bag: Vec<Tetrimino, 7> = get_random_bag();

    let mut current_tetri: Tetrimino = random_bag.swap_remove(0);
    let mut next_tetri: Tetrimino = random_bag.swap_remove(0);

    let mut held_tetri: Option<Tetrimino> = None;
    let mut held_blocked: bool = false;
    let mut held_button_down: bool = false;

    let mut hard_drop_blocked: bool = false;
    let mut soft_drop_blocked: bool = false;

    let mut fall_speed: u64 = (1000.0 / (FALL_SPEED_DATA[level as usize - 1] * 60.0)) as u64;

    draw_stable_ui(level, level_lines, score);

    draw_tetrimino(&current_tetri, false);
    draw_ghost_tetri(&current_tetri, &grid, false);
    draw_next_tetrimino(&next_tetri);

    'gameloop: loop {
        let keyboard_state = keyboard::scan();
        if (!move_button_down | ((last_move_time + AUTO_MOVE_SPEED < timing::millis()) & auto_repeat_on)) // if we touch the button for the first time in this frame, or if we maintained it pressed and some time has passed
            & (keyboard_state.key_down(RIGHT_KEY) | keyboard_state.key_down(LEFT_KEY))
        {
            // MOVE
            let direction: i16;
            if keyboard_state.key_down(LEFT_KEY) {
                direction = -1
            } else {
                direction = 1
            }
            if can_move(&current_tetri, (direction, 0), &grid) {
                draw_tetrimino(&current_tetri, true);
                draw_ghost_tetri(&current_tetri, &grid, true);

                current_tetri.pos.x += direction;

                draw_ghost_tetri(&current_tetri, &grid, false);
                draw_tetrimino(&current_tetri, false);

                last_move_time = timing::millis();
                move_button_down = true;
            }
        } else if (!rotate_button_down | (last_rotate_time + ROTATE_SPEED < timing::millis()))
            & (keyboard_state.key_down(RIGHT_ROTATION_KEY)
                | keyboard_state.key_down(LEFT_ROTATION_KEY))
        {
            // ROTATE
            let new_tetri = can_rotate(
                keyboard_state.key_down(RIGHT_ROTATION_KEY),
                &current_tetri,
                &grid,
            );
            if new_tetri.is_some() {
                draw_tetrimino(&current_tetri, true);
                draw_ghost_tetri(&current_tetri, &grid, true);

                current_tetri = new_tetri.unwrap();

                draw_ghost_tetri(&current_tetri, &grid, false);
                draw_tetrimino(&current_tetri, false);

                last_rotate_time = timing::millis();
                rotate_button_down = true;
            }
        } else if !held_button_down & !held_blocked & keyboard_state.key_down(HOLD_KEY) {
            // HOLD

            let temp = current_tetri.clone();

            if held_tetri.is_some() {
                held_blocked = true;
                held_button_down = true;

                draw_tetrimino(&current_tetri, true);
                draw_ghost_tetri(&current_tetri, &grid, true);

                current_tetri = get_initial_tetri(held_tetri.unwrap().tetri);

                draw_ghost_tetri(&current_tetri, &grid, false);
                draw_tetrimino(&current_tetri, false);

                held_tetri = Some(temp.clone());
                draw_held_tetrimino(&held_tetri.as_ref().unwrap());
            } else {
                held_blocked = true;
                held_button_down = true;

                draw_tetrimino(&current_tetri, true);
                draw_ghost_tetri(&current_tetri, &grid, true);

                if random_bag.len() == 0 {
                    random_bag = get_random_bag();
                }
                current_tetri = next_tetri.clone();
                next_tetri = random_bag.swap_remove(0);

                draw_ghost_tetri(&current_tetri, &grid, false);
                draw_tetrimino(&current_tetri, false);

                draw_next_tetrimino(&next_tetri);
                held_tetri = Some(temp.clone());
                draw_held_tetrimino(&held_tetri.as_ref().unwrap());
            }
        }
        if (last_fall_time
            + if (!soft_drop_button_down & keyboard_state.key_down(SOFT_DROP_KEY))
                | (soft_drop_button_down & !soft_drop_blocked)
            {
                SOFT_DROP_SPEED
            } else {
                fall_speed
            }
            < timing::millis())
            || (!hard_drop_blocked & keyboard_state.key_down(HARD_DROP_KEY))
        {
            if !hard_drop_blocked & keyboard_state.key_down(HARD_DROP_KEY) {
                hard_drop_blocked = true;
                draw_tetrimino(&current_tetri, true);
                if can_move(&current_tetri, (0, 1), &grid) {
                    while can_move(&current_tetri, (0, 1), &grid) {
                        current_tetri.pos.y += 1;
                    }
                }
            } else {
                // Place that somewhere else ? maybe doing the check differently too ?
                if !soft_drop_button_down && keyboard_state.key_down(SOFT_DROP_KEY) {
                    soft_drop_button_down = true;
                }
            }
            // FALL
            let need_to_fall = can_move(&current_tetri, (0, 1), &grid);
            if need_to_fall {
                draw_tetrimino(&current_tetri, true);
                current_tetri.pos.y += 1;
                draw_tetrimino(&current_tetri, false);
            } else {
                let mut death: bool = true;
                for p in current_tetri.get_blocks_grid_pos() {
                    grid.set_color_at(p.x, p.y, current_tetri.color);
                    if death && p.y >= 0 {
                        death = false;
                    }
                }
                if death {
                    draw_centered_string(" GAME OVER \0", 10, true, &COLOR_CONFIG, true);
                    let action = selection_menu(&COLOR_CONFIG, &DEATH_MENU, false);
                    break 'gameloop action;
                }

                let clear_lines_y = get_clear_lines(&current_tetri, &grid);
                if !clear_lines_y.is_empty() {
                    score = add_points(&clear_lines_y, level, score);
                    let temp_level = level;
                    (level_lines, level) =
                        add_lines(clear_lines_y.len() as u16, level, level_lines);
                    if level != temp_level {
                        fall_speed = (1000.0 / (FALL_SPEED_DATA[level as usize - 1] * 60.0)) as u64;
                    }

                    // BRINGS LINES DOWN (display done at the same time for optimisation) -> lol not optimised at all because drawing too much things
                    bring_lines_down(&clear_lines_y, &mut grid);
                }

                // new tetri because last one touched down
                if random_bag.len() == 0 {
                    random_bag = get_random_bag();
                }
                current_tetri = next_tetri.clone();
                next_tetri = random_bag.swap_remove(0);

                draw_ghost_tetri(&current_tetri, &grid, false);
                draw_tetrimino(&current_tetri, false);
                draw_next_tetrimino(&next_tetri);

                held_blocked = false;
                if soft_drop_button_down {
                    soft_drop_blocked = true;
                }
            }
            last_fall_time = timing::millis();
        }

        if move_button_down
            & !(keyboard_state.key_down(LEFT_KEY) | keyboard_state.key_down(RIGHT_KEY))
        {
            move_button_down = false;
            auto_repeat_on = false;
        }
        if move_button_down & (last_move_time + DELAYED_AUTO_SHIFT < timing::millis()) {
            auto_repeat_on = true;
        }
        if rotate_button_down
            & !(keyboard_state.key_down(RIGHT_ROTATION_KEY)
                | keyboard_state.key_down(LEFT_ROTATION_KEY))
        {
            rotate_button_down = false;
        }
        if soft_drop_button_down & !keyboard_state.key_down(SOFT_DROP_KEY) {
            soft_drop_button_down = false;
            soft_drop_blocked = false; 
        }
        if held_button_down & !keyboard_state.key_down(HOLD_KEY) {
            held_button_down = false;
        }
        if hard_drop_blocked & !keyboard_state.key_down(HARD_DROP_KEY) {
            hard_drop_blocked = false;
        }
        if keyboard_state.key_down(PAUSE_KEY) {
            // PAUSE MENU
            let action = pause_menu(&COLOR_CONFIG, 0);
            if action != 0 {
                return action;
            } else {
                wait_for_vblank();
                draw_stable_ui(level, level_lines, score);
                wait_for_vblank();
                for x in 0..PLAYFIELD_WIDTH {
                    for y in 0..PLAYFIELD_HEIGHT {
                        let c = grid.get_color_at(x as i16, y as i16);
                        if c.is_some() {
                            draw_block(x, y, c.unwrap() as u16)
                        }
                    }
                }
                wait_for_vblank();
                draw_tetrimino(&current_tetri, false);
                draw_ghost_tetri(&current_tetri, &grid, false);
                if held_tetri.is_some() {
                    draw_held_tetrimino(&held_tetri.as_ref().unwrap());
                }
                draw_next_tetrimino(&next_tetri);
            }
        }
        display::wait_for_vblank();
        // EST-CE UNE BONNE IDEE ?
    }
}

fn bring_lines_down(clear_lines_y: &Vec<i16, 4>, grid: &mut Grid) {
    // Removes every cleared line
    for i in clear_lines_y {
        grid.remove_line(*i);
        draw_blank_line(*i as u16);
    }
    // Brings lines down
    for (i, e) in clear_lines_y.iter().enumerate() {
        for j in (0..*e + i as i16).rev() {
            for x in 0..PLAYFIELD_WIDTH {
                let last_color = grid.get_color_at(x as i16, j as i16);
                if last_color.is_some() {
                    grid.set_color_at(x as i16, j as i16 + 1, last_color.unwrap());
                }
            }
            grid.remove_line(j as i16);
            draw_blank_line(j as u16);
        }
    }
}

fn draw_ghost_tetri(tetri: &Tetrimino, grid: &Grid, clear: bool) {
    let mut ghost_tetri = tetri.clone();
    ghost_tetri.color = 8;
    if can_move(tetri, (0, 1), grid) {
        while can_move(&ghost_tetri, (0, 1), grid) {
            ghost_tetri.pos.y += 1;
        }
        draw_tetrimino(&ghost_tetri, clear);
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
            for e in clear_lines_y.iter() {
                if (pos.y) < *e {
                    break;
                }
                new_index += 1;
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

    draw_score((sum * (level as u32)) + points);
    return (sum * (level as u32)) + points;
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

fn can_rotate(right: bool, tetri: &Tetrimino, grid: &Grid) -> Option<Tetrimino> {
    let mut rotated_tetri = tetri.clone();
    if right {
        rotated_tetri.rotate_right();
    } else {
        rotated_tetri.rotate_left();
    }
    if can_move(&rotated_tetri, (0, 0), grid) {
        return Some(rotated_tetri);
    } else {
        let kicks = get_wall_kicks_data(tetri, right);
        for k in kicks {
            if can_move(&rotated_tetri, k.clone(), grid) {
                rotated_tetri.pos.x += k.0;
                rotated_tetri.pos.y += k.1;
                return Some(rotated_tetri);
            }
        }
        return None;
    }
}
