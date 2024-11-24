use core::u8;

use heapless::Vec;
use numworks_utils::{
    graphical::{draw_centered_string, ColorConfig},
    menu::{
        self,
        settings::{Setting, SettingType},
        start_menu,
    },
};

use crate::{
    eadk::{
        display::{wait_for_vblank, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, timing, Color,
    },
    ia_p4::{find_best_move, look_for_aligned_coins},
    menu::MenuConfig,
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
            value: 0,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((SettingType::Bool(false), "No\0")) };
                unsafe { v.push_unchecked((SettingType::Bool(true), "Yes\0")) };
                v
            },
        },
        &mut Setting {
            name: "Players\0",
            value: 0,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((SettingType::Int(2), "2\0")) };
                unsafe { v.push_unchecked((SettingType::Int(3), "3\0")) };
                v
            },
        },
        &mut Setting {
            name: "Dark mode\0",
            value: 1,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((SettingType::Bool(true), "Yes\0")) };
                unsafe { v.push_unchecked((SettingType::Bool(false), "No\0")) };
                v
            },
        },
        &mut Setting {
            name: "IA Strength\0",
            value: 1,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((SettingType::Int(4), "Weak\0")) };
                unsafe { v.push_unchecked((SettingType::Int(6), "Normal\0")) };
                unsafe { v.push_unchecked((SettingType::Int(8), "Strong\0")) };
                v
            },
        },
    ];
    loop {
        let start = start_menu(
            "PUISSANCE 4\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./data/p4_controls.txt"),
        ); // The menu does everything itself !
        if start == 0 {
            unsafe {
                PLAYERS = opt[1].get_param_value::<u16>() as u8; // vis_addon update
            }
            loop {
                let color_config: ColorConfig = if opt[2].get_param_value::<bool>() {
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
                    opt[1].get_param_value::<u16>() as u8,
                    opt[0].get_param_value::<bool>(),
                    opt[3].get_param_value::<u16>() as u8,
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

/// The entire game is here.
pub fn game(nb_players: u8, solo: bool, ia_strength: u8, c: &ColorConfig) -> u8 {
    let mut table: Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE> = Vec::new();
    for _ in 0..MAX_WIDTH_SIZE {
        let mut new_vec = Vec::<u8, MAX_HEIGHT_SIZE>::new();
        for _ in 0..MAX_HEIGHT_SIZE {
            new_vec.push(u8::MAX).unwrap();
        }
        table.push(new_vec).unwrap();
    }
    let mut players_pos: [u16; 3] = [3, 3, 3];
    draw_grid(nb_players, c);
    let mut count_turns: u8 = 0;
    'gameloop: loop {
        for (player, pos) in players_pos.iter_mut().enumerate().take(nb_players as usize) {
            if player == 0 || !solo {
                *pos = selection(*pos, player as u16, nb_players, c);
                while !table.get(*pos as usize).unwrap().last().eq(&Some(&u8::MAX)) {
                    *pos = selection(*pos, player as u16, nb_players, c);
                }
                place_coin(*pos as u8, player as u8, &mut table, c);
            } else {
                let choice: u16 = find_best_move(
                    &table,
                    player as u8,
                    nb_players,
                    if count_turns < ia_strength {
                        // speed up the start of the game !
                        count_turns
                    } else {
                        ia_strength
                    },
                )
                .into();
                *pos = choice;
                place_coin(choice as u8, player as u8, &mut table, c);
            }
            let check = check(&table, nb_players);
            if let Some(align) = check {
                victory(align, c);
                break 'gameloop;
            }
            if table_is_full(&table, nb_players) {
                draw_centered_string("Egalite !\0", 10, true, c, false);
                break 'gameloop;
            }
            count_turns += 1;
        }
    }
    let menu_config = MenuConfig {
        choices: &["Replay\0", "Menu\0", "Exit\0"],
        rect_margins: (20, 0),
        dimensions: (SCREEN_WIDTH, LARGE_CHAR_HEIGHT),
        offset: (0, SCREEN_HEIGHT / 2 - LARGE_CHAR_HEIGHT),
        back_key_return: 1,
    };
    menu::selection(c, &menu_config, true)
}

/// Return True if the table is full (the game has ended in a tie)
fn table_is_full(table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>, players: u8) -> bool {
    let range_x = { 0..table.len() - (MAX_PLAYERS - players as usize) };
    for i in range_x {
        for j in table.get(i).unwrap() {
            if *j == u8::MAX {
                return false;
            }
        }
    }
    true
}

/// Places a coin in a Table, without drawing it to the screen. Useful to perform checks on table copies.
pub fn place_coin_nodraw(
    x: u8,
    number: u8,
    table: &mut Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
) -> u16 {
    let vec_x = table.get_mut(x as usize).unwrap();
    let mut y = 0;
    for i in vec_x {
        if *i == u8::MAX {
            *i = number;
            break;
        } else {
            y += 1
        }
    }
    y
}

/// Places a coin in the table and raws it to the screen.
fn place_coin(
    x: u8,
    number: u8,
    table: &mut Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
    c: &ColorConfig,
) {
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
