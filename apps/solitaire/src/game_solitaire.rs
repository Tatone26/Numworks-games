use heapless::Vec;
use numworks_utils::{
    eadk::{key, keyboard, timing, State},
    menu::{pause_menu, selection_menu},
    utils::{draw_centered_string, randint, wait_for_no_keydown},
};

use crate::{
    eadk::Color,
    menu::{menu, MyOption, OptionType},
    ui_solitaire::{
        draw_fondations_stack, draw_selection, draw_stock, draw_tableau_stack,
        menu_vis_addon, ui_setup, BACKGROUND_COLOR, REPLAY_MENU_CONFIG,
    },
    utils::{fill_screen, ColorConfig},
};

// This dictates the principal colors that will be used
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::WHITE,
    bckgrd: BACKGROUND_COLOR,
    alt: Color::RED,
};

/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut MyOption; 2] = [
        &mut MyOption {
            name: "Move anywhere ! - DEBUG\0",
            value: 1,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((OptionType::Bool(true), "Yes\0")) };
                unsafe { v.push_unchecked((OptionType::Bool(false), "No\0")) };
                v
            },
        },
        &mut MyOption {
            name: "Difficulty\0",
            value: 0,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((OptionType::Int(1), "Easy\0")) };
                unsafe { v.push_unchecked((OptionType::Int(2), "Normal\0")) };
                unsafe { v.push_unchecked((OptionType::Int(3), "HARD\0")) };
                v
            },
        },
    ];
    loop {
        let start = menu(
            "SOLITAIRE\0",
            &mut opt,
            &COLOR_CONFIG,
            menu_vis_addon,
            include_str!("./data/model_controls.txt"),
        );
        // The menu does everything itself !
        if start == 0 {
            loop {
                // a loop where the game is played again and again, which means it should be 100% contained after the menu
                let action = game(opt[1].get_param_value(), opt[0].get_param_value()); // calling the game based on the parameters is better
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
pub fn game(difficulty: u16, move_anywhere: bool) -> u8 {
    {
        let mut table = create_table();
        ui_setup(&table);
        let mut cursor_pos = CursorPos::Tableau(3);
        draw_selection(&cursor_pos, false, false, &table, 1);
        wait_for_no_keydown();
        let mut last_action: u64 = timing::millis();
        let mut last_action_key: u32 = key::ALPHA;
        let mut selection: Option<CursorPos> = None;
        let mut selection_size: u8 = 1;
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
                        if selection.is_some() && old_pos == selection.unwrap() {
                            selection_size
                        } else {
                            1
                        },
                    );
                    draw_selection(
                        &cursor_pos,
                        false,
                        selection.is_some() && cursor_pos == selection.unwrap(),
                        &table,
                        1,
                    );
                }
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::EXE)
                && timing::millis() >= (last_action + REPETITION_SPEED as u64)
            {
                turn_stock(&mut table, &mut selection, &cursor_pos, difficulty);
                last_action_key = key::EXE;
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::SHIFT)
                && timing::millis() >= (last_action + REPETITION_SPEED as u64)
            {
                if let CursorPos::Tableau(b) = cursor_pos {
                    if selection.is_some() && selection.unwrap() == cursor_pos {
                        let size = table.tableau[b as usize].len();
                        if selection_size < size as u8 {
                            if table.tableau[b as usize]
                                .get(size - selection_size as usize - 1)
                                .unwrap()
                                .visible
                                && is_card_placable_on_other_card(
                                    table.tableau[b as usize]
                                        .get(size - selection_size as usize)
                                        .unwrap(),
                                    table.tableau[b as usize]
                                        .get(size - selection_size as usize - 1)
                                        .unwrap(),
                                )
                            {
                                selection_size += 1;
                                draw_selection(&cursor_pos, false, true, &table, selection_size);
                            }
                        }
                    }
                }
                last_action_key = key::SHIFT;
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::ALPHA)
                && timing::millis() >= (last_action + REPETITION_SPEED as u64)
            {
                if let CursorPos::Tableau(_) = cursor_pos {
                    if selection.is_some() && selection.unwrap() == cursor_pos {
                        if selection_size > 1 {
                            draw_selection(&cursor_pos, true, false, &table, selection_size);
                            selection_size -= 1;
                            draw_selection(&cursor_pos, false, true, &table, selection_size);
                        }
                    }
                }
                last_action_key = key::ALPHA;
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::OK)
                && timing::millis() >= (last_action + REPETITION_SPEED as u64)
            {
                if let CursorPos::Stock(1) = cursor_pos {
                    turn_stock(&mut table, &mut selection, &cursor_pos, difficulty);
                } else {
                    move_card(
                        &mut table,
                        &mut selection,
                        &mut selection_size,
                        &cursor_pos,
                        move_anywhere,
                    );
                    if check_for_win(&table) {
                        auto_win(&mut table);
                        timing::msleep(1000);
                        let action = selection_menu(&COLOR_CONFIG, &REPLAY_MENU_CONFIG, false);
                        return action;
                    }
                }
                last_action_key = key::OK;
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::BACK) {
                if selection.is_some() {
                    draw_selection(&selection.unwrap(), true, false, &table, selection_size);
                    if selection.unwrap() == cursor_pos {
                        draw_selection(&cursor_pos, false, false, &table, 1);
                    }
                    selection = None;
                    selection_size = 1;
                }
            } else if !keyboard_state.key_down(last_action_key) {
                last_action = timing::millis() - REPETITION_SPEED as u64;
            }
            if keyboard_state.key_down(key::BACKSPACE) {
                let action = pause_menu(&COLOR_CONFIG, 0);
                if action == 0 {
                    ui_setup(&table);
                    draw_selection(
                        &cursor_pos,
                        false,
                        selection.is_some() && selection.unwrap() == cursor_pos,
                        &table,
                        1,
                    );
                    if selection.is_some() {
                        draw_selection(&selection.unwrap(), false, true, &table, selection_size);
                    }
                    last_action = timing::millis();
                } else {
                    return action;
                }
            }
        }
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

fn turn_stock(
    table: &mut Table,
    selection: &mut Option<CursorPos>,
    cursor_pos: &CursorPos,
    difficulty: u16,
) {
    if table.stock_iter >= table.stock.len() {
        table.stock_iter = 0;
    } else {
        table.stock_iter += difficulty as usize;
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
        draw_selection(cursor_pos, false, false, &table, 1);
    }
}

/// God help me, this is the fonction for what happens when click on OK. Yep.
fn move_card(
    table: &mut Table,
    selection: &mut Option<CursorPos>,
    selection_size: &mut u8,
    cursor_pos: &CursorPos,
    move_anywhere: bool,
) {
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
        *selection_size = 1;
        draw_selection(cursor_pos, false, selection.is_some(), &table, 1);
    } else {
        if selection.unwrap() == *cursor_pos {
            // If same place, then unselect
            if *selection_size > 1 {
                draw_selection(&selection.unwrap(), true, false, &table, *selection_size);
            }
            draw_selection(&cursor_pos, false, false, &table, 1);
            *selection = None;
            *selection_size = 1;
        } else if let CursorPos::Stock(_) = cursor_pos {
            // if there is a selection but we click on stock ? The other case are taken cared of outside of this function.
        } else if move_anywhere
            || is_move_possible(table, cursor_pos, &selection.unwrap(), &selection_size)
        {
            // Here, we move the card (if possible of course.)
            let selected_pos = selection.unwrap();
            let mut cards_to_move = Vec::<Card, 13>::new();
            for _ in 0..*selection_size {
                let mut new_card = {
                    // We REMOVE the card from its old place, so if there is nothing we cry
                    match selected_pos {
                        CursorPos::Tableau(i) => table.tableau[i as usize].pop(),
                        CursorPos::Fondations(i) => table.fondations[i as usize].pop(),
                        CursorPos::Stock(_) => Some(table.stock.remove(table.stock_iter - 1)),
                    }
                }
                .unwrap();
                new_card.visible = true;
                let _ = cards_to_move.push(new_card);
            }
            // TODO : verifications pour savoir si mouvement possible
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
                    for _ in 0..*selection_size {
                        table.tableau[*i as usize].push_unchecked(cards_to_move.pop().unwrap());
                    }
                    draw_tableau_stack(&table.tableau[*i as usize], *i as u16);
                },
                CursorPos::Fondations(i) => unsafe {
                    table.fondations[*i as usize].push_unchecked(cards_to_move.pop().unwrap());
                    draw_fondations_stack(&table.fondations[*i as usize], *i as u16);
                },
                _ => (), // Should NOT be possible.
            }
            draw_selection(cursor_pos, false, false, &table, 1);
            *selection = None;
            *selection_size = 1;
        }
    }
}

fn is_card_placable_on_other_card(card_to_move: &Card, base_card: &Card) -> bool {
    if base_card.number == card_to_move.number + 1 {
        return {
            match base_card.suit {
                Suit::Club => {
                    card_to_move.suit == Suit::Diamond || card_to_move.suit == Suit::Heart
                }
                Suit::Diamond => {
                    card_to_move.suit == Suit::Spade || card_to_move.suit == Suit::Club
                }
                Suit::Heart => card_to_move.suit == Suit::Spade || card_to_move.suit == Suit::Club,
                Suit::Spade => {
                    card_to_move.suit == Suit::Diamond || card_to_move.suit == Suit::Heart
                }
            }
        };
    }
    return false;
}

fn is_move_possible(
    table: &mut Table,
    cursor_pos: &CursorPos,
    selection: &CursorPos,
    selection_size: &u8,
) -> bool {
    let card_to_move = {
        match selection {
            CursorPos::Tableau(i) => table.tableau[*i as usize]
                .get(table.tableau[*i as usize].len() - *selection_size as usize),
            CursorPos::Fondations(i) => table.fondations[*i as usize].last(),
            CursorPos::Stock(_) => table.stock.get(table.stock_iter - 1),
        }
    }
    .unwrap(); // no check, should not be any problem here... BUT BE WARNED
    match cursor_pos {
        CursorPos::Tableau(i) => {
            let base_card = table.tableau[*i as usize].last();
            if base_card.is_some() {
                return is_card_placable_on_other_card(card_to_move, base_card.unwrap());
            } else {
                return card_to_move.number == 12;
            }
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

fn check_for_win(table: &Table) -> bool {
    if !table.stock.is_empty() {
        return false;
    } else {
        for v in &table.fondations {
            if v.is_empty() {
                return false;
            }
        }
        for v in table.tableau.iter().rev() {
            for c in v {
                if !c.visible {
                    return false;
                }
            }
        }
    }
    return true;
}

fn auto_win(table: &mut Table) {
    while {
        let mut test = false;
        for v in &table.tableau {
            if !v.is_empty() {
                test = true;
                break;
            }
        }
        test
    } {
        'test: for (i, fond) in table.fondations.iter_mut().enumerate() {
            let last_card = fond.last().unwrap(); // Safe because of check_for_win
            let card_to_find = Card {
                suit: last_card.suit,
                number: last_card.number + 1,
                visible: true,
            };
            for (e, v) in table.tableau.iter_mut().enumerate() {
                if v.last().is_some()
                    && v.last().unwrap().suit == card_to_find.suit
                    && v.last().unwrap().number == card_to_find.number
                {
                    let card_to_move = v.pop();
                    if card_to_move.is_some() {
                        let _ = fond.push(card_to_move.unwrap());
                    }
                    draw_tableau_stack(v, e as u16);
                    draw_fondations_stack(fond, i as u16);
                    timing::msleep(100);
                    break 'test;
                }
            }
        }
    }
    draw_centered_string("BRAVO !\0", 50, true, &COLOR_CONFIG, true);
}
