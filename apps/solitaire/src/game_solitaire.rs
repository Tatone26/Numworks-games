use heapless::Vec;

use crate::{
    eadk::{
        display::{self, push_rect_uniform},
        key, keyboard, timing, Color, Rect,
    },
    menu::{menu, pause_menu, MyOption},
    ui_solitaire::{
        clear_selection, draw_last_deck_cards, draw_selection,
        draw_selection_card, draw_table, get_abs_pos_stack, NICE_GREEN,
    },
    utils::{fill_screen, randint, ColorConfig},
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
        let start = menu("SOLITAIRE\0", &mut opt, &COLOR_CONFIG, vis_addon); // The menu does everything itself !
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

#[derive(Clone, Copy, Debug)]
pub struct Card {
    pub number: Option<u16>,
    pub suit: u16,
    pub shown: bool,
}

impl Card {
    pub fn turn(&mut self) {
        self.shown = !self.shown
    }
}

pub const NONE_CARD: Card = Card {
    number: None,
    suit: 0,
    shown: false,
};

pub struct Table {
    pub deck: Stack,
    pub final_stacks: [Stack; 4],
    pub stacks: [Stack; 7],
    pub turned_deck_index: u8,
}

impl Table {
    pub fn new_table() -> Self {
        let mut temp = Vec::<Card, 52>::new();
        for s in 0..4 {
            for n in 0..13 {
                temp.push(Card {
                    number: Some(n + 1),
                    suit: s,
                    shown: false,
                })
                .unwrap();
            }
        }
        let mut mixed_cards = Vec::<Card, 52>::new();
        for _ in 0..52 {
            let rd = randint(0, temp.len() as u32);
            mixed_cards.push(temp[rd as usize]).unwrap();
            temp.swap_remove(rd as usize);
        }
        let mut random_table = Table {
            deck: Stack::new(),
            final_stacks: [Stack::new(), Stack::new(), Stack::new(), Stack::new()],
            stacks: [
                Stack::new(),
                Stack::new(),
                Stack::new(),
                Stack::new(),
                Stack::new(),
                Stack::new(),
                Stack::new(),
            ],
            turned_deck_index: 0,
        };
        for i in 0..7 {
            for _ in 0..i + 1 {
                random_table.stacks[i].add_card_on_top(*mixed_cards.last().unwrap());
                mixed_cards.pop();
            }
            random_table.stacks[i].turn_first_card();
        }
        for k in mixed_cards {
            random_table.deck.add_card_on_top(k);
        }
        random_table.turned_deck_index = random_table.deck.len();
        return random_table;
    }

    pub fn get_stack_from_pos(&self, cursor: u8) -> &Stack {
        if 6 <= cursor && cursor <= 12 {
            return &self.stacks[(cursor - 6) as usize];
        } else if cursor <= 3 {
            return &self.final_stacks[cursor as usize];
        } else if cursor == 4 || cursor == 5 {
            return &self.deck;
        } else {
            panic!()
        }
    }

    pub fn get_last_three_visible_deck_cards(&self) -> Vec<&Card, 3> {
        let mut res = Vec::<&Card, 3>::new();
        for k in self.turned_deck_index..self.deck.len() {
            res.push(self.deck.get_card_from_index(k)).unwrap();
        }
        if res.is_empty() {
            res.push(&NONE_CARD).unwrap();
        }
        return res;
    }

    pub fn turn_last_three_visible_deck_cards(&mut self) {
        for k in self.turned_deck_index..self.deck.len() {
            let card = self.deck.stack.get_mut(k as usize);
            if card.is_some() {
                card.unwrap().turn();
            }
        }
    }
}

pub struct Stack {
    stack: Vec<Card, 52>,
}

impl Stack {
    pub const fn new() -> Self {
        return Stack {
            stack: Vec::<Card, 52>::new(),
        };
    }

    pub fn add_card_on_top(&mut self, card: Card) {
        self.stack.push(card).unwrap();
    }

    pub fn get_all_cards(&self) -> &Vec<Card, 52> {
        return &self.stack;
    }

    pub fn get_last_card(&self) -> &Card {
        let card = self.stack.last();
        if card.is_some() {
            return self.stack.last().unwrap();
        } else {
            return &NONE_CARD;
        }
    }

    pub fn remove_last_card(&mut self) {
        self.stack.pop().unwrap();
    }

    pub fn is_empty(&self) -> bool {
        return self.stack.is_empty();
    }

    pub fn len(&self) -> u8 {
        return self.stack.len() as u8;
    }

    pub fn get_card_index(&self, card: &Card) -> u16 {
        for (i, k) in self.get_all_cards().iter().enumerate() {
            if k.suit == card.suit && k.number == card.number {
                return i as u16;
            }
        }
        if card.number.is_none() {
            return 0;
        }
        panic!()
    }

    pub fn get_card_from_index(&self, index: u8) -> &Card {
        let card = self.stack.get(index as usize);
        if card.is_some() {
            return card.unwrap();
        } else {
            return &NONE_CARD;
        }
    }

    pub fn turn_first_card(&mut self) {
        self.stack.first_mut().unwrap().shown = true
    }
}

fn next_position(pos: u8, direction: u8) -> u8 {
    // direction being 0, 1, 2 or 3 for left, up, down, right (key::LEFT etc)
    if direction == 0 {
        if 6 < pos && pos <= 12 {
            return pos - 1;
        } else if pos == 6 {
            return 12;
        } else if 0 < pos && pos <= 5 {
            return pos - 1;
        } else {
            return pos;
        }
    } else if direction == 3 {
        if 6 <= pos && pos < 12 {
            return pos + 1;
        } else if pos == 12 {
            return 6;
        } else if pos < 5 {
            return pos + 1;
        } else {
            return pos;
        }
    } else if direction == 1 {
        if 6 <= pos && pos <= 9 {
            return pos - 6;
        } else if pos == 10 || pos == 11 {
            return 4;
        } else if pos == 12 {
            return 5;
        } else {
            return pos;
        }
    } else if direction == 2 {
        if pos <= 3 {
            return pos + 6;
        } else if pos == 4 {
            return 10;
        } else if pos == 5 {
            return 12;
        } else {
            return pos;
        }
    } else {
        return pos;
    }
}

const REPETITION_SPEED: u64 = 150;

/// The entire game is here.
pub fn game(exemple: bool) -> u8 {
    fill_screen(NICE_GREEN);
    let mut table = Table::new_table();
    let mut cursor_position: u8 = 9;
    let mut last_click: u64 = timing::millis();
    let mut selection = Vec::<&Card, 52>::new();
    let mut selection_stack: Option<u8> = None;
    draw_table(&table);
    draw_selection(
        table.get_stack_from_pos(cursor_position),
        cursor_position,
        1,
        false,
    );
    loop {
        let keyboard_state = keyboard::scan();
        if last_click + REPETITION_SPEED < timing::millis() {
            let last_pos: u8 = cursor_position;
            if keyboard_state.key_down(key::UP) {
                cursor_position = next_position(cursor_position, key::UP as u8);
            } else if keyboard_state.key_down(key::DOWN) {
                cursor_position = next_position(cursor_position, key::DOWN as u8);
            } else if keyboard_state.key_down(key::LEFT) {
                cursor_position = next_position(cursor_position, key::LEFT as u8);
            } else if keyboard_state.key_down(key::RIGHT) {
                cursor_position = next_position(cursor_position, key::RIGHT as u8);
            } else if keyboard_state.key_down(key::OK) {
                if cursor_position == 5 {
                    if selection_stack.is_some() {
                        clear_selection(
                            table.get_stack_from_pos(selection_stack.unwrap()),
                            5,
                            selection.len() as u8,
                        );
                        selection.clear();
                        selection_stack = None;
                    }
                    table.turned_deck_index += 3;
                    table.turn_last_three_visible_deck_cards();
                    draw_last_deck_cards(&table);
                }
                // PTN NIK SA MAMAN EN PYtHon Y A PAS PLUS FACILE FAIT CHIER
                // faire des copies ? tant pis ? 
                let selection_first = selection.first();
                //let here_first = table.get_stack_from_pos(cursor_position).get_last_card();
                if selection_first.is_some()
                //& (selection_first.unwrap().number != here_first.number
                //    || selection_first.unwrap().suit != here_first.suit)
                {
                    clear_selection(
                        table.get_stack_from_pos(selection_stack.unwrap()),
                        selection_stack.unwrap(),
                        selection.len() as u8 + 1,
                    );
                } else if selection_first.is_none() {
                    //let last = table.get_stack_from_pos(cursor_position).stack.last();
                    //selection.push(if last.is_none() { &NONE_CARD } else {last.unwrap()}).unwrap();
                    selection_stack = Some(cursor_position);
                    draw_selection(
                        table.get_stack_from_pos(selection_stack.unwrap()),
                        cursor_position,
                        1,
                        true,
                    );
                }
            }
            if last_pos != cursor_position {
                if selection_stack.is_none() || last_pos != selection_stack.unwrap() {
                    if last_pos != 4 {
                        clear_selection(
                            table.get_stack_from_pos(last_pos),
                            last_pos,
                            selection.len() as u8 + 1,
                        );
                    } else {
                        draw_last_deck_cards(&table);
                    }
                } else if selection_stack.is_some() && last_pos == selection_stack.unwrap() {
                    if cursor_position != 4 {
                        draw_selection(
                            table.get_stack_from_pos(last_pos),
                            last_pos,
                            selection.len() as u8 + 1,
                            true,
                        );
                    } else {
                        let k = table.get_last_three_visible_deck_cards()[0];
                        draw_selection_card(k, true, get_abs_pos_stack(4));
                    }
                }
                if cursor_position != 4 {
                    draw_selection(
                        table.get_stack_from_pos(cursor_position),
                        cursor_position,
                        selection.len() as u8 + 1,
                        false,
                    );
                } else {
                    let k = table.get_last_three_visible_deck_cards()[0];
                    draw_selection_card(k, false, get_abs_pos_stack(4));
                }
                display::wait_for_vblank();
            }
            if keyboard_state.any_down() {
                last_click = timing::millis();
            }
        }
        if !keyboard_state.any_down() {
            last_click = 0;
        }
    }
    return pause_menu(&COLOR_CONFIG, 100);
}
