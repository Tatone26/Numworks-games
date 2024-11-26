use heapless::Vec;

use crate::{
    cards::{Card, TOTAL_CARDS},
    game_solitaire::CursorPos,
    ui_solitaire::{draw_selection, draw_stock},
};

pub type Stack = Vec<Card, TOTAL_CARDS>;

pub struct Table {
    pub tableau: [Stack; 7],
    pub fondations: [Vec<Card, 13>; 4],
    pub stock: Stack,
    pub stock_iter: usize,
}

impl Table {
    /// Creates an empty table
    pub fn empty() -> Self {
        Table {
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
        }
    }

    /// Fills this table with a given deck (with copies of the cards)
    pub fn fill_table(&mut self, deck: &Stack) {
        let mut count = 0;
        for (i, tab) in self.tableau.iter_mut().enumerate() {
            for _ in 0..(i + 1) {
                let toadd = deck.get(count).unwrap();
                tab.push(*toadd).unwrap();
                count += 1;
            }
            tab.last_mut().unwrap().visible = true;
        }
        for c in deck.iter().skip(count) {
            self.stock.push(*c).unwrap();
        }
    }
}

/// Takes a turn of the stock
pub fn turn_stock(
    table: &mut Table,
    selection: &mut Option<CursorPos>,
    cursor_pos: &CursorPos,
    cards_to_turn: u16,
) {
    if table.stock_iter >= table.stock.len() {
        table.stock_iter = 0;
    } else {
        table.stock_iter += cards_to_turn as usize;
        if table.stock_iter > table.stock.len() {
            table.stock_iter = table.stock.len()
        }
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
        draw_selection(cursor_pos, false, false, table, 1);
    }
}

/// Check for win : stock empty, ni empty foundations and all cards visible.
pub fn check_for_win(table: &Table) -> bool {
    if !table.stock.is_empty() {
        false
    } else {
        for v in table.tableau.iter().rev() {
            for c in v {
                if !c.visible {
                    return false;
                }
            }
        }
        for v in &table.fondations {
            if v.is_empty() {
                return false;
            }
        }
        true
    }
}
