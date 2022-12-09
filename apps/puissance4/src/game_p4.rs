use heapless::Vec;

use crate::{
    eadk::{
        display::{self, push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, timing, Color, Rect,
    },
    menu::{menu, selection_menu, MenuConfig, MyOption},
    ui_p4::{clear_selection_coin, draw_coin, draw_grid, draw_selection_coin, victory},
    utils::{wait_for_no_keydown, ColorConfig, LARGE_CHAR_HEIGHT},
};

/// The number of Boolean Options used. Public so menu() can use it.
pub const BOOL_OPTIONS_NUMBER: usize = 1;

// This dictates the principal colors that will be used
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: Color::WHITE,
    alt: Color::RED,
};

static mut EXEMPLE: bool = false;

fn vis_addon() {
    push_rect_uniform(
        Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        },
        Color::BLACK,
    );
}
/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut MyOption<bool, 2>; BOOL_OPTIONS_NUMBER] = [&mut MyOption {
        name: "Option !\0",
        value: 0,
        possible_values: [(true, "True\0"), (false, "False\0")],
    }];
    loop {
        let start = menu("PUISSANCE 4\0", &mut opt, &COLOR_CONFIG, vis_addon); // The menu does everything itself !
        if start == 1 {
            unsafe {
                EXEMPLE = opt[0].get_value().0; // You could use mutable statics, but it is not very good
            }
            loop {
                // a loop where the game is played again and again, which means it should be 100% contained after the menu
                let action = game(opt[0].get_value().0); // calling the game based on the parameters is better
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

/// The entire game is here.
pub fn game(_exemple: bool) -> u8 {
    let mut table: Vec<Vec<u8, 6>, 7> = Vec::new();
    for _ in 0..7 {
        let mut new_vec = Vec::<u8, 6>::new();
        for _ in 0..6 {
            new_vec.push(0).unwrap();
        }
        table.push(new_vec).unwrap();
    }
    let mut blue_pos: u16 = 3;
    let mut red_pos: u16 = 3;
    draw_grid();
    loop {
        blue_pos = selection(blue_pos, Color::BLUE);
        while !table.get(blue_pos as usize).unwrap().last().eq(&Some(&0)) {
            blue_pos = selection(blue_pos, Color::BLUE);
        }
        place_coin(blue_pos, 1, &mut table);
        let check_1 = check(&table);
        if check_1.is_some() {
            victory(check_1);
            break; // TODO : Afficher ligne, vainqueur, message etc...
        }
        red_pos = selection(red_pos, Color::RED);
        while !table.get(red_pos as usize).unwrap().last().eq(&Some(&0)) {
            red_pos = selection(red_pos, Color::RED);
        }
        place_coin(red_pos, 2, &mut table);
        let check_2 = check(&table);
        if check_2.is_some() | table_is_full(&table) {
            victory(check_2);
            break;
        }
    }
    let menu_config = MenuConfig {
        first_choice: "Replay\0",
        second_choice: "Menu\0",
        null_choice: "Exit\0",
        rect_margins: (0, 0),
        dimensions: (SCREEN_WIDTH, LARGE_CHAR_HEIGHT),
        offset: (0, SCREEN_HEIGHT / 2 - LARGE_CHAR_HEIGHT),
        back_key_return: 2,
    };
    return selection_menu(&COLOR_CONFIG, &menu_config, true);
}



fn table_is_full(table: &Vec<Vec<u8, 6>, 7>) -> bool {
    for i in table {
        for j in i {
            if *j == 0 {
                return false;
            }
        }
    }
    return true;
}

fn place_coin(x: u16, number: u8, table: &mut Vec<Vec<u8, 6>, 7>) {
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
    draw_coin(
        x,
        y,
        if number == 2 {
            Color::RED
        } else if number == 1 {
            Color::BLUE
        } else {
            Color::WHITE
        },
    );
}

const REPETITION_SPEED: u64 = 250;

fn selection(initial_pos: u16, color: Color) -> u16 {
    let mut pos = initial_pos;
    wait_for_no_keydown();
    let mut last_action: u64 = timing::millis();
    let mut last_action_key: u32 = key::ALPHA;
    draw_selection_coin(initial_pos, color);
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
                if pos < 6 {
                    pos += 1;
                }
            }
            if old_pos != pos {
                clear_selection_coin(old_pos);
                draw_selection_coin(pos, color);
                display::wait_for_vblank();
            }
            last_action = timing::millis();
        } else if keyboard_state.key_down(key::OK) | keyboard_state.key_down(key::DOWN) {
            wait_for_no_keydown();
            clear_selection_coin(pos);
            break;
        } else if !keyboard_state.key_down(last_action_key) {
            last_action = timing::millis() - REPETITION_SPEED;
        }
    }
    return pos;
}

fn check(table: &Vec<Vec<u8, 6>, 7>) -> Option<(u8, (u16, u16), (u16, u16))> {
    for x in 0..4 {
        let x_vec = table.get(x).unwrap();
        if x_vec.len() == 0 {
            panic!()
        }
        for y in 0..6 {
            let t = table[x][y];
            if t != 0 && t == table[x + 1][y] && t == table[x + 2][y] && t == table[x + 3][y] {
                return Some((t, (x as u16, y as u16), (x as u16 + 3, y as u16)));
            }
        }
        for y in 0..3 {
            let t = table[x][y];
            if t != 0
                && t == table[x + 1][y + 1]
                && t == table[x + 2][y + 2]
                && t == table[x + 3][y + 3]
            {
                return Some((t, (x as u16, y as u16), (x as u16 + 3, y as u16 + 3)));
            }
        }
        for y in 3..6 {
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
    for x in 0..7 {
        for y in 0..3 {
            let t = table[x][y];
            if t != 0 && t == table[x][y + 1] && t == table[x][y + 2] && t == table[x][y + 3] {
                return Some((t, (x as u16, y as u16), (x as u16, y as u16 + 3)));
            }
        }
    }

    return None;
}
