use heapless::Vec;
use numworks_utils::{
    eadk::{display::SCREEN_WIDTH, key, keyboard, timing, Color, State},
    graphical::{draw_centered_string, fading, ColorConfig},
    menu::{
        self, pause_menu,
        settings::{Setting, SettingType},
        start_menu, MenuConfig,
    },
    utils::{wait_for_no_keydown, LARGE_CHAR_HEIGHT},
};

use crate::{
    cards::{create_deck, is_card_placable_on_other_card, Card},
    table::{check_for_win, turn_stock, Table},
    ui_solitaire::{
        draw_foundations_stack, draw_selection, draw_stock, draw_table, draw_tableau_stack,
        menu_vis_addon, BACKGROUND_COLOR,
    },
};

// This dictates the principal colors that will be used
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::WHITE,
    bckgrd: BACKGROUND_COLOR,
    alt: Color::RED,
};

pub const REPLAY_MENU_CONFIG: MenuConfig = MenuConfig {
    choices: &["Resume\0", "Menu\0", "Exit\0"],
    rect_margins: (20, 10),
    dimensions: (SCREEN_WIDTH * 2 / 5, LARGE_CHAR_HEIGHT * 7),
    offset: (0, 50),
    back_key_return: 0,
};

/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut Setting; 1] = [&mut Setting {
        name: "Difficulty\0",
        value: 2,
        possible_values: {
            let mut v = Vec::new();
            unsafe { v.push_unchecked((SettingType::Int(1), "Easy\0")) };
            unsafe { v.push_unchecked((SettingType::Int(2), "Normal\0")) };
            unsafe { v.push_unchecked((SettingType::Int(3), "HARD\0")) };
            v
        },
    }];
    loop {
        let start = start_menu(
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
/// Pretty horrifying, but I have done my best.
pub fn game(difficulty: u16) -> u8 {
    {
        // Table creation
        let mut table = Table::empty();
        table.fill_table(&create_deck());
        draw_table(&table);

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
                move_cursor(&mut cursor_pos, &keyboard_state);
                if old_pos != cursor_pos {
                    draw_selection(
                        &old_pos,
                        true,
                        Some(old_pos) == selection,
                        &table,
                        if Some(old_pos) == selection {
                            selection_size
                        } else {
                            1
                        },
                    );
                    draw_selection(&cursor_pos, false, Some(cursor_pos) == selection, &table, 1);
                }

                if keyboard_state.key_down(key::LEFT) {
                    last_action_key = key::LEFT;
                } else if keyboard_state.key_down(key::RIGHT) {
                    last_action_key = key::RIGHT;
                } else if keyboard_state.key_down(key::UP) {
                    last_action_key = key::UP;
                } else if keyboard_state.key_down(key::DOWN) {
                    last_action_key = key::DOWN;
                }
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::EXE)
                && timing::millis() >= (last_action + REPETITION_SPEED as u64)
            {
                // Turn stock
                turn_stock(&mut table, &mut selection, &cursor_pos, difficulty);

                last_action_key = key::EXE;
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::SHIFT)
                && timing::millis() >= (last_action + REPETITION_SPEED as u64)
            {
                // increasing selection size
                increase_selection_size(&table, &cursor_pos, &selection, &mut selection_size);

                last_action_key = key::SHIFT;
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::ALPHA)
                && timing::millis() >= (last_action + REPETITION_SPEED as u64)
            {
                // decreasing selection size
                decrease_selection_size(&table, &cursor_pos, &selection, &mut selection_size);

                last_action_key = key::ALPHA;
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::OK)
                && timing::millis() >= (last_action + REPETITION_SPEED as u64)
            {
                // When "OK"
                if let CursorPos::Stock(1) = cursor_pos {
                    // turnstock...
                    turn_stock(&mut table, &mut selection, &cursor_pos, difficulty);
                } else {
                    // ... or move card
                    move_card(&mut table, &mut selection, &mut selection_size, &cursor_pos);
                    if check_for_win(&table) {
                        auto_win(&mut table);
                        timing::msleep(1000);
                        let action = menu::selection(&COLOR_CONFIG, &REPLAY_MENU_CONFIG, false);
                        return action;
                    }
                }

                last_action_key = key::OK;
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::TOOLBOX)
                && timing::millis() >= (last_action + REPETITION_SPEED as u64)
            {
                // automove !
                if auto_move(&mut table, &cursor_pos, &mut selection, &mut selection_size)
                    && check_for_win(&table)
                {
                    auto_win(&mut table);
                    timing::msleep(1000);
                    let action = menu::selection(&COLOR_CONFIG, &REPLAY_MENU_CONFIG, false);
                    return action;
                }

                last_action_key = key::TOOLBOX;
                last_action = timing::millis();
            } else if keyboard_state.key_down(key::BACK) {
                // undo selection
                if selection.is_some() {
                    draw_selection(&selection.unwrap(), true, false, &table, selection_size);
                    if selection.unwrap() == cursor_pos {
                        draw_selection(&cursor_pos, false, false, &table, 1);
                    }
                    selection = None;
                    selection_size = 1;
                }

                last_action_key = key::BACK;
                last_action = timing::millis();
            } else if !keyboard_state.key_down(last_action_key) {
                last_action = timing::millis() - REPETITION_SPEED as u64;
            }
            // Pause menu
            if keyboard_state.key_down(key::BACKSPACE) {
                let action = pause_menu(&COLOR_CONFIG, 4);
                if action == 0 {
                    draw_table(&table);
                    draw_selection(
                        &cursor_pos,
                        false,
                        selection.is_some() && selection.unwrap() == cursor_pos,
                        &table,
                        1,
                    );
                    if let Some(select) = selection {
                        draw_selection(&select, false, true, &table, selection_size);
                    }
                    last_action = timing::millis();
                } else {
                    fading(500);
                    return action;
                }
            }
        }
    }
}

#[inline]
/// Finds a possible place for the card (on the foundations only) and auto-moves it.
/// Can be improved by making it check on the Tableau too.
fn auto_move(
    table: &mut Table,
    cursor_pos: &CursorPos,
    selection: &mut Option<CursorPos>,
    selection_size: &mut u8,
) -> bool {
    // Auto-move to fondations piles.
    for i in 0..4 {
        if is_move_possible(table, &CursorPos::Fondations(i), cursor_pos, selection_size) {
            move_card(
                table,
                &mut Some(*cursor_pos),
                selection_size,
                &CursorPos::Fondations(i),
            );
            if Some(CursorPos::Fondations(i)) == *selection {
                *selection = None;
            }
            draw_selection(&CursorPos::Fondations(i), true, false, table, 1);
            draw_selection(cursor_pos, false, false, table, 1);
            return true;
        }
    }
    false
}

#[inline]
fn increase_selection_size(
    table: &Table,
    cursor_pos: &CursorPos,
    selection: &Option<CursorPos>,
    selection_size: &mut u8,
) {
    // Increase selection size
    if let CursorPos::Tableau(b) = cursor_pos {
        if Some(*cursor_pos) == *selection {
            let size = table.tableau[*b as usize].len();
            if *selection_size < size as u8
                && table.tableau[*b as usize]
                    .get(size - *selection_size as usize - 1)
                    .unwrap()
                    .visible
                && is_card_placable_on_other_card(
                    // increase selection only if the next card is part of a valid stack
                    table.tableau[*b as usize]
                        .get(size - *selection_size as usize)
                        .unwrap(),
                    table.tableau[*b as usize]
                        .get(size - *selection_size as usize - 1)
                        .unwrap(),
                )
            {
                *selection_size += 1;
                draw_selection(cursor_pos, false, true, table, *selection_size);
            }
        }
    }
}

#[inline]
fn decrease_selection_size(
    table: &Table,
    cursor_pos: &CursorPos,
    selection: &Option<CursorPos>,
    selection_size: &mut u8,
) {
    if let CursorPos::Tableau(_) = cursor_pos {
        if *selection == Some(*cursor_pos) && *selection_size > 1 {
            draw_selection(cursor_pos, true, false, table, *selection_size);
            *selection_size -= 1;
            draw_selection(cursor_pos, false, true, table, *selection_size);
        }
    }
}

#[inline]
fn new_cursor_pos_value(cursor_pos: &CursorPos, new_value: u8) -> CursorPos {
    match cursor_pos {
        CursorPos::Tableau(_) => CursorPos::Tableau(new_value),
        CursorPos::Fondations(_) => CursorPos::Fondations(new_value),
        CursorPos::Stock(_) => CursorPos::Stock(new_value),
    }
}

/// Moves the cursor where it needs to go. Does not do any drawing.
fn move_cursor(cursor_pos: &mut CursorPos, keyboard_state: &State) {
    let value = u8::from_value(cursor_pos);
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

/// Despite its name, it takes care of everything happening when clicking on "OK"
fn move_card(
    table: &mut Table,
    selection: &mut Option<CursorPos>,
    selection_size: &mut u8,
    cursor_pos: &CursorPos,
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
        draw_selection(cursor_pos, false, selection.is_some(), table, 1);
    } else if *selection == Some(*cursor_pos) {
        // If same place, then unselect
        if *selection_size > 1 {
            draw_selection(&selection.unwrap(), true, false, table, *selection_size);
        }
        draw_selection(cursor_pos, false, false, table, 1);
        *selection = None;
        *selection_size = 1;
    } else if is_move_possible(table, cursor_pos, &selection.unwrap(), selection_size) {
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
        match selected_pos {
            CursorPos::Tableau(i) => {
                if !table.tableau[i as usize].is_empty() {
                    table.tableau[i as usize].last_mut().unwrap().visible = true;
                }
                draw_tableau_stack(&table.tableau[i as usize], i as u16);
            }
            CursorPos::Fondations(i) => {
                draw_foundations_stack(&table.fondations[i as usize], i as u16)
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
                draw_foundations_stack(&table.fondations[*i as usize], *i as u16);
            },
            _ => (), // Should NOT be possible.
        }
        draw_selection(cursor_pos, false, false, table, 1);
        *selection = None;
        *selection_size = 1;
    }
}

/// Checks to see if a given move is possible.
fn is_move_possible(
    table: &mut Table,
    cursor_pos: &CursorPos,
    selection: &CursorPos,
    selection_size: &u8,
) -> bool {
    let card_to_move = {
        match selection {
            CursorPos::Tableau(i) => table.tableau[*i as usize].get(
                table.tableau[*i as usize]
                    .len()
                    .saturating_sub(*selection_size as usize),
            ),
            CursorPos::Fondations(i) => table.fondations[*i as usize].last(),
            CursorPos::Stock(_) => {
                if table.stock_iter > 0 {
                    table.stock.get(table.stock_iter - 1)
                } else {
                    None
                }
            }
        }
    };
    if let Some(card_to_move_) = card_to_move {
        match cursor_pos {
            CursorPos::Tableau(i) => {
                let base_card = table.tableau[*i as usize].last();
                if let Some(b_c) = base_card {
                    is_card_placable_on_other_card(card_to_move_, b_c)
                } else {
                    card_to_move_.number == 12
                }
            }
            CursorPos::Fondations(i) => {
                if table.fondations[*i as usize].is_empty() {
                    card_to_move_.number == 0
                } else {
                    (table.fondations[*i as usize].last().unwrap().suit == card_to_move_.suit)
                        && (table.fondations[*i as usize].last().unwrap().number
                            == card_to_move_.number - 1)
                }
            }
            CursorPos::Stock(_) => false,
        }
    } else {
        false
    }
}

/// Auto fills the foundations at the end.
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
                    if let Some(c_t_m) = card_to_move {
                        let _ = fond.push(c_t_m);
                    }
                    draw_tableau_stack(v, e as u16);
                    draw_foundations_stack(fond, i as u16);
                    timing::msleep(150);
                    break 'test;
                }
            }
        }
    }
    draw_centered_string(" BRAVO ! \0", 75, true, &COLOR_CONFIG, false);
}
