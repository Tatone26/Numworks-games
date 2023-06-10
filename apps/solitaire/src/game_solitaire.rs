use heapless::Vec;
use numworks_utils::{
    eadk::{key, keyboard, timing, State},
    utils::{randint, wait_for_no_keydown},
};

use crate::{
    eadk::{display::push_rect_uniform, Color, Rect},
    menu::{menu, MyOption, OptionType},
    ui_solitaire::{
        draw_fondations_stack, draw_selection, draw_stock, draw_tableau_stack, ui_setup,
    },
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
            "SOLITAIRE\0",
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
#[derive(Debug, Copy, Clone, PartialEq)]
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

const REPETITION_SPEED: u16 = 200;

/// The entire game is here.
pub fn game(_exemple: bool) -> u8 {
    {
        let mut table = create_table();
        ui_setup(&table);
        let mut cursor_pos = CursorPos::Tableau(3);
        draw_selection(&cursor_pos, false, false, &table);
        wait_for_no_keydown();
        let mut last_action: u64 = timing::millis();
        let mut last_action_key: u32 = key::ALPHA;
        let mut selection: Option<CursorPos> = None;
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
                if keyboard_state.key_down(key::LEFT) {
                    last_action_key = key::LEFT;
                } else if keyboard_state.key_down(key::RIGHT) {
                    last_action_key = key::RIGHT;
                } else if keyboard_state.key_down(key::UP) {
                    last_action_key = key::UP;
                } else if keyboard_state.key_down(key::DOWN) {
                    last_action_key = key::DOWN;
                }
                move_cursor(&mut cursor_pos, &keyboard_state);
                if old_pos != cursor_pos {
                    draw_selection(
                        &old_pos,
                        true,
                        selection.is_some() && old_pos == selection.unwrap(),
                        &table,
                    );
                    draw_selection(
                        &cursor_pos,
                        false,
                        selection.is_some() && cursor_pos == selection.unwrap(),
                        &table,
                    );
                }
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::EXE)
                && timing::millis() >= (last_action + REPETITION_SPEED as u64)
            {
                turn_stock(&mut table, &mut selection, &cursor_pos);
                last_action_key = key::EXE;
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::OK)
                && timing::millis() >= (last_action + REPETITION_SPEED as u64)
            {
                if let CursorPos::Stock(1) = cursor_pos {
                    turn_stock(&mut table, &mut selection, &cursor_pos);
                } else {
                    move_card(&mut table, &mut selection, &cursor_pos);
                }
                last_action_key = key::OK;
                last_action = timing::millis();
            } else if !keyboard_state.key_down(last_action_key) {
                last_action = timing::millis() - REPETITION_SPEED as u64;
            }
        }
        return 1;
    }
}

fn new_cursor_pos_value(cursor_pos: &CursorPos, new_value: u8) -> CursorPos {
    match cursor_pos {
        CursorPos::Tableau(_) => return CursorPos::Tableau(new_value),
        CursorPos::Fondations(_) => return CursorPos::Fondations(new_value),
        CursorPos::Stock(_) => return CursorPos::Stock(new_value),
    }
}

fn move_cursor(cursor_pos: &mut CursorPos, keyboard_state: &State) {
    let value = u8::from_value(&cursor_pos);
    if keyboard_state.key_down(key::LEFT) {
        if value > 0 {
            *cursor_pos = new_cursor_pos_value(cursor_pos, value - 1);
        } else if let CursorPos::Stock(_) = cursor_pos {
            // Only possible "strange" case : from stock(0) to fondations(3)
            *cursor_pos = CursorPos::Fondations(3);
        }
    } else if keyboard_state.key_down(key::RIGHT) {
        match cursor_pos {
            CursorPos::Tableau(_) => {
                if value < 6 {
                    *cursor_pos = new_cursor_pos_value(cursor_pos, value + 1)
                }
            }
            CursorPos::Fondations(_) => {
                if value < 3 {
                    *cursor_pos = new_cursor_pos_value(cursor_pos, value + 1)
                } else {
                    *cursor_pos = CursorPos::Stock(0) // Strange case : from fondations(3) to stock(0)
                }
            }
            CursorPos::Stock(_) => {
                if value < 1 {
                    *cursor_pos = new_cursor_pos_value(cursor_pos, value + 1)
                }
            }
        }
    } else if keyboard_state.key_down(key::UP) {
        if let CursorPos::Tableau(_) = cursor_pos {
            if value <= 3 {
                *cursor_pos = CursorPos::Fondations(value);
            } else if value == 6 {
                *cursor_pos = CursorPos::Stock(1);
            } else {
                *cursor_pos = CursorPos::Stock(0);
            }
        }
    } else if keyboard_state.key_down(key::DOWN) {
        match cursor_pos {
            CursorPos::Tableau(_) => (),
            CursorPos::Fondations(_) => *cursor_pos = CursorPos::Tableau(value),
            CursorPos::Stock(_) => {
                if value == 0 {
                    *cursor_pos = CursorPos::Tableau(4);
                } else {
                    *cursor_pos = CursorPos::Tableau(6);
                }
            }
        }
    }
}

fn turn_stock(table: &mut Table, selection: &mut Option<CursorPos>, cursor_pos: &CursorPos) {
    if table.stock_iter >= table.stock.len() {
        table.stock_iter = 0;
    } else {
        table.stock_iter += 1;
    }
    draw_stock(&table.stock, table.stock_iter);
    if selection.is_some() {
        // If the card on top of the shown stock was selected, unselect it.
        if let CursorPos::Stock(_) = selection.unwrap() {
            *selection = None;
        }
    }
    if let CursorPos::Stock(_) = cursor_pos {
        // Do not redraw selection if not necessary. That's the kind of optimisations we want.
        draw_selection(cursor_pos, false, false, &table);
    }
}

/// God help me
fn move_card(table: &mut Table, selection: &mut Option<CursorPos>, cursor_pos: &CursorPos) {
    if selection.is_none() {
        match cursor_pos {
            CursorPos::Tableau(i) => {
                if !table.tableau[*i as usize].is_empty() {
                    *selection = Some(*cursor_pos)
                }
            }
            CursorPos::Fondations(i) => {
                if !table.fondations[*i as usize].is_empty() {
                    *selection = Some(*cursor_pos)
                }
            }
            CursorPos::Stock(i) => {
                if *i == 0 && table.stock_iter > 0 {
                    *selection = Some(*cursor_pos)
                }
            }
        }
        draw_selection(cursor_pos, false, selection.is_some(), &table);
    } else {
        if selection.unwrap() == *cursor_pos {
            // If same place, then unselect
            *selection = None;
            draw_selection(cursor_pos, false, false, &table);
        } else if let CursorPos::Stock(_) = cursor_pos {
            // if there is a selection but we click on stock ? The other case are taken cared of outside of this function.
        } else if is_move_possible(table, cursor_pos, &selection.unwrap()) {
            // Here, we move the card (if possible of course.)
            let selected_pos = selection.unwrap();
            let mut new_card = {
                // We REMOVE the card from its old place, so if there is nothing we cry
                match selected_pos {
                    CursorPos::Tableau(i) => table.tableau[i as usize].pop(),
                    CursorPos::Fondations(i) => table.fondations[i as usize].pop(),
                    CursorPos::Stock(_) => Some(table.stock.remove(table.stock_iter - 1)),
                }
            }
            .unwrap();
            // TODO : verifications pour savoir si mouvement possible
            new_card.visible = true;
            match selected_pos {
                CursorPos::Tableau(i) => {
                    if !table.tableau[i as usize].is_empty() {
                        table.tableau[i as usize].last_mut().unwrap().visible = true;
                    }
                    draw_tableau_stack(&table.tableau[i as usize], i as u16);
                }
                CursorPos::Fondations(i) => {
                    draw_fondations_stack(&table.fondations[i as usize], i as u16)
                }
                CursorPos::Stock(_) => {
                    table.stock_iter -= 1;
                    draw_stock(&table.stock, table.stock_iter)
                }
            }
            match cursor_pos {
                CursorPos::Tableau(i) => unsafe {
                    table.tableau[*i as usize].push_unchecked(new_card);
                    draw_tableau_stack(&table.tableau[*i as usize], *i as u16);
                },
                CursorPos::Fondations(i) => unsafe {
                    table.fondations[*i as usize].push_unchecked(new_card);
                    draw_fondations_stack(&table.fondations[*i as usize], *i as u16);
                },
                _ => (), // Should NOT be possible.
            }
            draw_selection(cursor_pos, false, false, &table);
            *selection = None;
        }
    }
}

fn is_move_possible(table: &Table, cursor_pos: &CursorPos, selection: &CursorPos) -> bool {
    let card_to_move = {
        match selection {
            CursorPos::Tableau(i) => table.tableau[*i as usize].last(),
            CursorPos::Fondations(i) => table.fondations[*i as usize].last(),
            CursorPos::Stock(_) => table.stock.get(table.stock_iter - 1),
        }
    }
    .unwrap(); // no check, should not be any problem here... BUT BE WARNED
    match cursor_pos {
        CursorPos::Tableau(i) => {
            let base_card = table.tableau[*i as usize].last();
            if base_card.is_some() {
                if base_card.unwrap().number == card_to_move.number + 1 {
                    return {
                        match base_card.unwrap().suit {
                            Suit::Club => {
                                card_to_move.suit == Suit::Diamond
                                    || card_to_move.suit == Suit::Heart
                            }
                            Suit::Diamond => {
                                card_to_move.suit == Suit::Spade || card_to_move.suit == Suit::Club
                            }
                            Suit::Heart => {
                                card_to_move.suit == Suit::Spade || card_to_move.suit == Suit::Club
                            }
                            Suit::Spade => {
                                card_to_move.suit == Suit::Diamond
                                    || card_to_move.suit == Suit::Heart
                            }
                        }
                    };
                }
            } else {
                return card_to_move.number == 12;
            }
            return false;
        }
        CursorPos::Fondations(i) => {
            if table.fondations[*i as usize].is_empty() {
                return card_to_move.number == 0;
            } else if (table.fondations[*i as usize].last().unwrap().suit == card_to_move.suit)
                && (table.fondations[*i as usize].last().unwrap().number == card_to_move.number - 1)
            {
                return true;
            } else {
                return false;
            }
        }
        CursorPos::Stock(_) => return false,
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
