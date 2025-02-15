use heapless::Vec;
use numworks_utils::{
    graphical::{draw_centered_string, ColorConfig},
    menu::{self, settings::Setting, start_menu, MenuConfig},
    utils::randint,
};

use crate::{
    eadk::{
        display::{wait_for_vblank, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, timing, Color,
    },
    ia_p4::{find_best_move, look_for_aligned_coins},
    ui_p4::{clear_selection_coin, draw_coin, draw_grid, draw_selection_coin, victory},
    utils::{wait_for_no_keydown, LARGE_CHAR_HEIGHT},
};

// This dictates the principal colors that will be used
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: Color::WHITE,
    alt: Color::from_rgb888(90, 90, 255),
};

// This static is used only for the visual addon of the menu.
static mut PLAYERS: u8 = 2;
pub const MAX_PLAYERS: usize = 3;

fn vis_addon() {
    unsafe {
        draw_selection_coin(2, 0, &COLOR_CONFIG, 25);
        draw_selection_coin(3, 1, &COLOR_CONFIG, 25);
        draw_selection_coin(4, if PLAYERS == 3 { 2 } else { 0 }, &COLOR_CONFIG, 25);
    }
}
/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut Setting; 4] = [
        &mut Setting {
            name: "Solo Game\0",
            choice: 0,
            values: Vec::from_slice(&[0, 1]).unwrap(),
            texts: Vec::from_slice(&["No\0", "Yes\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "Players\0",
            choice: 0,
            values: Vec::from_slice(&[2, 3]).unwrap(),
            texts: Vec::from_slice(&["2\0", "3\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "Dark mode\0",
            choice: 1,
            values: Vec::from_slice(&[1, 0]).unwrap(),
            texts: Vec::from_slice(&["Yes\0", "No\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        &mut Setting {
            name: "IA Strength\0",
            choice: 1,
            values: Vec::from_slice(&[5, 18, i16::MAX as u32]).unwrap(), // This values dictate the "strength" of the AI, stopping as soon as it finds a move better than that.
            texts: Vec::from_slice(&["Weak\0", "Normal\0", "Strong\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
    ];
    loop {
        let start = start_menu(
            "CONNECT 4\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./data/p4_controls.txt"),
            "connectfour",
        ); // The menu does everything itself !
        if start == 0 {
            unsafe {
                PLAYERS = opt[1].get_setting_value() as u8; // vis_addon update
            }
            loop {
                let color_config: ColorConfig = if opt[2].get_setting_value() != 0 {
                    ColorConfig {
                        text: COLOR_CONFIG.bckgrd,
                        bckgrd: COLOR_CONFIG.text,
                        alt: COLOR_CONFIG.alt,
                    }
                } else {
                    COLOR_CONFIG
                };
                // a loop where the game is played again and again, which means it should be 100% contained after the menu
                let action = game(
                    opt[1].get_setting_value() as u8,
                    opt[0].get_setting_value() != 0,
                    opt[3].get_setting_value() as i16,
                    &color_config,
                ); // calling the game based on the parameters is better
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

pub const DEFAULT_TABLE_VALUE: u8 = u8::MAX;

pub type Table = Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>;

/// The entire game is here.
pub fn game(nb_players: u8, solo: bool, ia_strength: i16, c: &ColorConfig) -> u8 {
    let mut table: Table = Vec::new();
    for _ in 0..MAX_WIDTH_SIZE {
        let mut new_vec = Vec::<u8, MAX_HEIGHT_SIZE>::new();
        for _ in 0..MAX_HEIGHT_SIZE {
            new_vec.push(DEFAULT_TABLE_VALUE).unwrap();
        }
        table.push(new_vec).unwrap();
    }
    let mut players_pos: [u16; 3] = [3, 3, 3];
    let start_player = randint(0, nb_players as u32) as usize;

    draw_grid(nb_players, c);

    let mut turn_count = 0;
    'gameloop: loop {
        for (p, pos) in players_pos.iter_mut().enumerate().take(nb_players as usize) {
            let player = (p + start_player) % nb_players as usize;
            if player == 0 || !solo {
                *pos = selection(*pos, player as u16, nb_players, c);
                while !table
                    .get(*pos as usize)
                    .unwrap()
                    .last()
                    .eq(&Some(&DEFAULT_TABLE_VALUE))
                {
                    *pos = selection(*pos, player as u16, nb_players, c);
                }
                place_coin(*pos as u8, player as u8, &mut table, c);
            } else {
                let choice: u16 = find_best_move(
                    &table,
                    player as u8,
                    nb_players,
                    if turn_count < 1 + 2 * (nb_players - 1) {
                        // a way of randomizing the start of the game : the AI will play (almost) randomly for the first one to two moves !
                        ia_strength / 5
                    } else {
                        ia_strength
                    },
                    c,
                ) as u16;
                *pos = choice;
                place_coin(choice as u8, player as u8, &mut table, c);
            }
            let check = check(&table, nb_players);
            if let Some(align) = check {
                victory(align, c);
                break 'gameloop;
            }
            if table_is_full(&table, nb_players) {
                draw_centered_string("It's a draw!\0", 10, true, c, false);
                break 'gameloop;
            }
            turn_count += 1;
        }
    }
    let menu_config = MenuConfig {
        choices: &["Replay\0", "Menu\0", "Exit\0"],
        rect_margins: (20, 0),
        dimensions: (SCREEN_WIDTH, LARGE_CHAR_HEIGHT),
        offset: (0, (SCREEN_HEIGHT / 2 - LARGE_CHAR_HEIGHT) as i16),
        back_key_return: 1,
    };
    menu::selection(c, &menu_config, true)
}

/// Return True if the table is full (the game has ended in a tie)
fn table_is_full(table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>, players: u8) -> bool {
    let range_x = { 0..table.len() - (MAX_PLAYERS - players as usize) };
    for i in range_x {
        for j in table.get(i).unwrap() {
            if *j == DEFAULT_TABLE_VALUE {
                return false;
            }
        }
    }
    true
}

/// Places a coin in a Table, without drawing it to the screen. Useful to perform checks on table.
pub fn place_coin_nodraw(x: u8, number: u8, table: &mut Table) -> u16 {
    let vec_x = table.get_mut(x as usize).unwrap();
    let mut y = 0;
    for i in vec_x {
        if *i == DEFAULT_TABLE_VALUE {
            *i = number;
            break;
        } else {
            y += 1
        }
    }
    //draw_coin(x as u16, y, number as u16, &COLOR_CONFIG, false);
    y
}

/// used for AI, to perform some tests on table without copying it.
pub fn remove_coin_nodraw(x: u8, number: u8, table: &mut Table) -> bool {
    let vec_x = table.get_mut(x as usize).unwrap();
    for (_y, i) in vec_x.iter_mut().enumerate().rev() {
        if *i != DEFAULT_TABLE_VALUE {
            if *i == number {
                *i = DEFAULT_TABLE_VALUE;
                //clear_coin(x as u16, y as u16, &COLOR_CONFIG);
                return true;
            }
            break;
        }
    }
    false
}

/// Places a coin in the table and raws it to the screen.
fn place_coin(x: u8, number: u8, table: &mut Table, c: &ColorConfig) {
    let y = place_coin_nodraw(x, number, table);
    draw_coin(x as u16, y, number as u16, c, false);
}

/// The speed at which the cursor moves when keeping the button pressed.
const REPETITION_SPEED: u64 = 250;

/// Returns a manually selected position, takes care of the selection drawing too.
fn selection(initial_pos: u16, current_player: u16, nb_players: u8, c: &ColorConfig) -> u16 {
    let mut pos = initial_pos;
    wait_for_no_keydown();
    let mut last_action: u64 = timing::millis();
    let mut last_action_key: u32 = key::ALPHA;
    draw_selection_coin(initial_pos, current_player, c, 0);
    loop {
        let keyboard_state = keyboard::scan();
        if (keyboard_state.key_down(key::LEFT) || keyboard_state.key_down(key::RIGHT))
            && (timing::millis() >= last_action + REPETITION_SPEED)
        {
            let old_pos = pos;
            if keyboard_state.key_down(key::LEFT) {
                last_action_key = key::LEFT;
                pos = pos.saturating_sub(1);
            } else if keyboard_state.key_down(key::RIGHT) {
                last_action_key = key::RIGHT;
                // saturing add, with max being this thing
                if pos < { MAX_WIDTH_SIZE as u16 - 1 - (MAX_PLAYERS as u16) + (nb_players as u16) }
                {
                    pos += 1;
                }
            }
            if old_pos != pos {
                clear_selection_coin(old_pos, c);
                draw_selection_coin(pos, current_player, c, 0);
            }
            last_action = timing::millis();
        } else if keyboard_state.key_down(key::OK) || keyboard_state.key_down(key::DOWN) {
            wait_for_no_keydown();
            wait_for_vblank();
            clear_selection_coin(pos, c);
            break;
        } else if !keyboard_state.key_down(last_action_key) {
            last_action = timing::millis() - REPETITION_SPEED;
        }
    }
    pos
}

/// Represents an alignements from .1 to .2, size .0
pub struct Alignment(pub u8, pub (u16, u16), pub (u16, u16));

/// Returns None if there is no winner, Some((t, (x1, y1), (x2,y2))) with t the winner, and (x1, y1) & (x2, y2) the two extremities of the winning line.
/// Pretty computing intensive function, but on a small table like those, it's okay (we don't care about speed anyway)
pub fn check(
    table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
    nb_players: u8,
) -> Option<Alignment> {
    for i in 0..nb_players {
        let (count, mut first, mut last) = look_for_aligned_coins(
            table,
            i,
            4,
            MAX_WIDTH_SIZE - MAX_PLAYERS + (nb_players as usize),
        );
        if count >= 1 {
            // checks for more than 4 aligned coins !
            for p in 5..(if MAX_WIDTH_SIZE > MAX_HEIGHT_SIZE {
                MAX_WIDTH_SIZE
            } else {
                MAX_HEIGHT_SIZE
            }) {
                let (count_more, first_more, last_more) = look_for_aligned_coins(
                    table,
                    i,
                    p as u8,
                    MAX_WIDTH_SIZE - MAX_PLAYERS + (nb_players as usize),
                );
                if count_more >= 1 {
                    first = first_more;
                    last = last_more;
                } else {
                    break;
                }
            }

            return Some(Alignment(i, first, last));
        }
    }
    None
}
