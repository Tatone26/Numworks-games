use heapless::Vec;

use crate::{
    eadk::{
        display::{SCREEN_HEIGHT, SCREEN_WIDTH, wait_for_vblank},
        key, keyboard, timing, Color,
    },
    menu::{menu, selection_menu, MenuConfig, MyOption, OptionType},
    ui_p4::{clear_selection_coin, draw_coin, draw_grid, draw_selection_coin, victory},
    utils::{draw_centered_string, wait_for_no_keydown, ColorConfig, LARGE_CHAR_HEIGHT},
};

// This dictates the principal colors that will be used
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: Color::WHITE,
    alt: Color::from_rgb888(90, 90, 255),
};

static mut PLAYERS: u8 = 2;
pub const MAX_PLAYERS: u8 = 3;

fn vis_addon() {
    unsafe {
        draw_selection_coin(2, 0, &COLOR_CONFIG, 25);
        draw_selection_coin(3, 1, &COLOR_CONFIG, 25);
        draw_selection_coin(4, if PLAYERS == 3 { 2 } else { 0 }, &COLOR_CONFIG, 25);
    }
}
/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut MyOption; 2] = [
        &mut MyOption {
            name: "Players\0",
            value: 0,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((OptionType::Int(2), "2\0")) };
                unsafe { v.push_unchecked((OptionType::Int(3), "3\0")) };
                v
            },
        },
        &mut MyOption {
            name: "Dark mode\0",
            value: 1,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((OptionType::Bool(true), "Yes\0")) };
                unsafe { v.push_unchecked((OptionType::Bool(false), "No\0")) };
                v
            },
        },
    ];
    loop {
        let start = menu(
            "PUISSANCE 4\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./data/p4_controls.txt"),
        ); // The menu does everything itself !
        if start == 0 {
            unsafe {
                PLAYERS = opt[0].get_param_value::<u16>() as u8; // You could use mutable statics, but it is not very good
            }
            loop {
                let color_config: ColorConfig;
                if opt[1].get_param_value() {
                    color_config = ColorConfig {
                        text: COLOR_CONFIG.bckgrd,
                        bckgrd: COLOR_CONFIG.text,
                        alt: COLOR_CONFIG.alt,
                    }
                } else {
                    color_config = COLOR_CONFIG
                }
                // a loop where the game is played again and again, which means it should be 100% contained after the menu
                let action = game(opt[0].get_param_value::<u16>() as u8, &color_config); // calling the game based on the parameters is better
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

pub const MAX_WIDTH_SIZE: usize = 8;
pub const MAX_HEIGHT_SIZE: usize = 6;

pub const PLAYERS_COLORS: [Color; 3] = [Color::BLUE, Color::RED, Color::from_rgb888(250, 200, 0)];

/// The entire game is here.
pub fn game(players: u8, c: &ColorConfig) -> u8 {
    let mut table: Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE> = Vec::new();
    for _ in 0..MAX_WIDTH_SIZE {
        let mut new_vec = Vec::<u8, MAX_HEIGHT_SIZE>::new();
        for _ in 0..MAX_HEIGHT_SIZE {
            new_vec.push(0).unwrap();
        }
        table.push(new_vec).unwrap();
    }
    let mut players_pos: [u16; 3] = [3, 3, 3];
    draw_grid(players, c);
    'gameloop: loop {
        for p in 0..players as usize {
            players_pos[p] = selection(players_pos[p], p as u16, players, c);
            while !table
                .get(players_pos[p] as usize)
                .unwrap()
                .last()
                .eq(&Some(&0))
            {
                players_pos[p] = selection(players_pos[p], p as u16, players, c);
            }
            place_coin(players_pos[p], p as u8 + 1, &mut table, c);
            let check = check(&table);
            if check.is_some() {
                victory(check, c);
                break 'gameloop;
            }
            if table_is_full(&table, players) {
                draw_centered_string("Egalite !\0", 10, true, c, false);
                break 'gameloop;
            }
        }
    }
    let menu_config = MenuConfig {
        choices: &["Replay\0", "Menu\0", "Exit\0"],
        rect_margins: (0, 0),
        dimensions: (SCREEN_WIDTH, LARGE_CHAR_HEIGHT),
        offset: (0, SCREEN_HEIGHT / 2 - LARGE_CHAR_HEIGHT),
        back_key_return: 1,
    };
    return selection_menu(c, &menu_config, true);
}

fn table_is_full(table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>, players: u8) -> bool {
    let range_x = { 0..table.len() - (MAX_PLAYERS - players) as usize };
    for i in range_x {
        for j in table.get(i).unwrap() {
            if *j == 0 {
                return false;
            }
        }
    }
    return true;
}

fn place_coin(
    x: u16,
    number: u8,
    table: &mut Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
    c: &ColorConfig,
) {
    let vec_x = table.get_mut(x as usize).unwrap();
    let mut y = 0;
    for i in vec_x {
        if *i == 0 {
            *i = i.checked_add(number).unwrap();
            break;
        } else {
            y += 1
        }
    }
    draw_coin(x, y, number as u16 - 1, c, false);
}

const REPETITION_SPEED: u64 = 250;

fn selection(initial_pos: u16, color: u16, players: u8, c: &ColorConfig) -> u16 {
    let mut pos = initial_pos;
    wait_for_no_keydown();
    let mut last_action: u64 = timing::millis();
    let mut last_action_key: u32 = key::ALPHA;
    draw_selection_coin(initial_pos, color, c, 0);
    loop {
        let keyboard_state = keyboard::scan();
        if (keyboard_state.key_down(key::LEFT) | keyboard_state.key_down(key::RIGHT))
            & (timing::millis() >= last_action + REPETITION_SPEED)
        {
            let old_pos = pos;
            if keyboard_state.key_down(key::LEFT) {
                last_action_key = key::LEFT;
                if pos > 0 {
                    pos -= 1;
                }
            } else if keyboard_state.key_down(key::RIGHT) {
                last_action_key = key::RIGHT;
                if pos < {MAX_WIDTH_SIZE as u16 - 1 - (MAX_PLAYERS - players) as u16} {
                    pos += 1;
                }
            }
            if old_pos != pos {
                clear_selection_coin(old_pos, c);
                draw_selection_coin(pos, color, c, 0);
            }
            last_action = timing::millis();
        } else if keyboard_state.key_down(key::OK) | keyboard_state.key_down(key::DOWN) {
            wait_for_no_keydown();
            wait_for_vblank();
            clear_selection_coin(pos, c);
            break;
        } else if !keyboard_state.key_down(last_action_key) {
            last_action = timing::millis() - REPETITION_SPEED;
        }
    }
    return pos;
}

fn check(
    table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
) -> Option<(u8, (u16, u16), (u16, u16))> {
    for x in 0..MAX_WIDTH_SIZE - 3 {
        let x_vec = table.get(x).unwrap();
        if x_vec.len() == 0 {
            panic!()
        }
        for y in 0..MAX_HEIGHT_SIZE {
            let t = table[x][y];
            if t != 0 && t == table[x + 1][y] && t == table[x + 2][y] && t == table[x + 3][y] {
                return Some((t, (x as u16, y as u16), (x as u16 + 3, y as u16)));
            }
        }
        for y in 0..MAX_HEIGHT_SIZE - 3 {
            let t = table[x][y];
            if t != 0
                && t == table[x + 1][y + 1]
                && t == table[x + 2][y + 2]
                && t == table[x + 3][y + 3]
            {
                return Some((t, (x as u16, y as u16), (x as u16 + 3, y as u16 + 3)));
            }
        }
        for y in MAX_HEIGHT_SIZE - 3..MAX_HEIGHT_SIZE {
            let t = table[x][y];
            if t != 0
                && t == table[x + 1][y - 1]
                && t == table[x + 2][y - 2]
                && t == table[x + 3][y - 3]
            {
                return Some((t, (x as u16, y as u16), (x as u16 + 3, y as u16 - 3)));
            }
        }
    }
    for x in 0..MAX_WIDTH_SIZE {
        for y in 0..MAX_HEIGHT_SIZE - 3 {
            let t = table[x][y];
            if t != 0 && t == table[x][y + 1] && t == table[x][y + 2] && t == table[x][y + 3] {
                return Some((t, (x as u16, y as u16), (x as u16, y as u16 + 3)));
            }
        }
    }

    return None;
}
