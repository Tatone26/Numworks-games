use heapless::Vec;
use numworks_utils::{
    eadk::{
        key, keyboard,
        timing::{self, msleep},
    },
    utils::{randint, wait_for_no_keydown},
};

use crate::{
    eadk::{display::push_rect_uniform, Color, Rect},
    menu::{menu, MyOption, OptionType},
    ui_solitaire::{draw_selection, draw_stock, ui_test},
    utils::{fill_screen, ColorConfig},
};

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
            "SNAKE 2.0\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./model_controls.txt"),
        );
        // The menu does everything itself !
        if start == 0 {
            unsafe {
                EXEMPLE = opt[0].get_param_value(); // You could use mutable statics, but it is not very good
            }
            loop {
                // a loop where the game is played again and again, which means it should be 100% contained after the menu
                let action = game(opt[0].get_param_value()); // calling the game based on the parameters is better
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

/// The number associated with each values is the position in the tileset.
#[derive(Debug, Copy, Clone)]
pub enum Suit {
    Club = 2,
    Diamond = 3,
    Heart = 1,
    Spade = 0,
}

#[derive(Debug, Copy, Clone)]
pub struct Card {
    pub suit: Suit,
    pub number: u8,
    pub visible: bool,
}

pub struct Table {
    pub tableau: [Vec<Card, 52>; 7],
    pub fondations: [Vec<Card, 13>; 4],
    pub stock: Vec<Card, 52>,
    pub stock_iter: usize,
}

impl Table {
    pub fn empty() -> Self {
        let table = Table {
            tableau: [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],
            fondations: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            stock: Vec::new(),
            stock_iter: 0,
        };
        return table;
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CursorPos {
    Tableau(u8),
    Fondations(u8),
    Stock(u8),
}

pub trait FromValue {
    fn from_value(value: &CursorPos) -> Self;
}

impl FromValue for u8 {
    fn from_value(value: &CursorPos) -> Self {
        match value {
            CursorPos::Tableau(b) => *b,
            CursorPos::Fondations(b) => *b,
            CursorPos::Stock(b) => *b,
        }
    }
}

fn new_cursor_pos_value(cursor_pos: CursorPos, new_value: u8) -> CursorPos {
    match cursor_pos {
        CursorPos::Tableau(_) => return CursorPos::Tableau(new_value),
        CursorPos::Fondations(_) => return CursorPos::Fondations(new_value),
        CursorPos::Stock(_) => return CursorPos::Stock(new_value),
    }
}

const REPETITION_SPEED: u16 = 200;

/// The entire game is here.
pub fn game(_exemple: bool) -> u8 {
    {
        let mut table = create_table();
        ui_test(&table);
        let mut cursor_pos = CursorPos::Tableau(3);
        draw_selection(cursor_pos, true, &table);
        wait_for_no_keydown();
        let mut last_action: u64 = timing::millis();
        let mut last_action_key: u32 = key::ALPHA;
        loop {
            let keyboard_state = keyboard::scan();
            if (keyboard_state.key_down(key::DOWN)
                || keyboard_state.key_down(key::UP)
                || keyboard_state.key_down(key::LEFT)
                || keyboard_state.key_down(key::RIGHT))
                && (timing::millis() >= (last_action + REPETITION_SPEED as u64))
            {
                // Move the cursor accordingly, draw selections etc
                let old_pos = cursor_pos;
                let value = u8::from_value(&cursor_pos);
                if keyboard_state.key_down(key::LEFT) {
                    last_action_key = key::LEFT;
                    if value > 0 {
                        cursor_pos = new_cursor_pos_value(cursor_pos, value - 1);
                    } else if let CursorPos::Stock(_) = cursor_pos {
                        cursor_pos = CursorPos::Fondations(3);
                    }
                } else if keyboard_state.key_down(key::RIGHT) {
                    last_action_key = key::RIGHT;
                    match cursor_pos {
                        CursorPos::Tableau(_) => {
                            if value < 6 {
                                cursor_pos = new_cursor_pos_value(cursor_pos, value + 1)
                            }
                        }
                        CursorPos::Fondations(_) => {
                            if value < 3 {
                                cursor_pos = new_cursor_pos_value(cursor_pos, value + 1)
                            } else {
                                cursor_pos = CursorPos::Stock(0)
                            }
                        }
                        CursorPos::Stock(_) => {
                            if value < 1 {
                                cursor_pos = new_cursor_pos_value(cursor_pos, value + 1)
                            }
                        }
                    }
                } else if keyboard_state.key_down(key::UP) {
                    if let CursorPos::Tableau(_) = cursor_pos {
                        if value <= 3 {
                            cursor_pos = CursorPos::Fondations(value);
                        } else if value == 6 {
                            cursor_pos = CursorPos::Stock(1);
                        } else {
                            cursor_pos = CursorPos::Stock(0);
                        }
                    }
                } else if keyboard_state.key_down(key::DOWN) {
                    match cursor_pos {
                        CursorPos::Tableau(_) => (),
                        CursorPos::Fondations(_) => cursor_pos = CursorPos::Tableau(value),
                        CursorPos::Stock(_) => {
                            if value == 0 {
                                cursor_pos = CursorPos::Tableau(4);
                            } else {
                                cursor_pos = CursorPos::Tableau(6);
                            }
                        }
                    }
                }
                if old_pos != cursor_pos {
                    draw_selection(old_pos, false, &table);
                    draw_selection(cursor_pos, true, &table);
                }
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::EXE)
                && timing::millis() >= (last_action + REPETITION_SPEED as u64)
            {
                if table.stock_iter >= table.stock.len() {
                    table.stock_iter = 0;
                } else {
                    table.stock_iter += 1;
                }
                draw_stock(&table.stock, table.stock_iter);
                last_action_key = key::EXE;
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::OK) {
                todo!();
            } else if !keyboard_state.key_down(last_action_key) {
                last_action = timing::millis() - REPETITION_SPEED as u64;
            }
        }
        return 1;
    }
}

/// Shuffles in-place the deck, with Fisher-Yates algorithm.
fn shuffle_deck(deck: &mut Vec<Card, 52>) {
    for i in (1..52).rev() {
        let choice = randint(0, i);
        deck.swap(choice as usize, i as usize);
    }
}

fn create_table() -> Table {
    let mut deck = Vec::<Card, 52>::new();
    let suit_list = [Suit::Club, Suit::Diamond, Suit::Heart, Suit::Spade];
    for i in 0..4 {
        for e in 0..13 {
            let test = deck.push(Card {
                suit: suit_list[i],
                number: e,
                visible: false,
            });
            if test.is_err() {
                fill_screen(Color::RED);
                loop {}
            }
        }
    }
    shuffle_deck(&mut deck);
    let mut table = Table::empty();
    for i in 0..7 {
        for _ in 0..(i + 1) {
            let toadd = deck.pop();
            match toadd {
                Some(_) => unsafe { table.tableau[i].push_unchecked(toadd.unwrap()) },
                None => panic!(),
            };
        }
        table.tableau[i].last_mut().unwrap().visible = true;
    }
    table.stock = deck;
    return table;
}
