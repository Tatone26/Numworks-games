use heapless::Vec;

use crate::{
    eadk::{display::push_rect_uniform, Color, Rect, random},
    menu::{menu, pause_menu, MyOption},
    ui::{draw_stack, draw_card, get_abs_pos_card, NICE_GREEN, draw_table},
    utils::{randint, ColorConfig, fill_screen},
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

pub struct Table {
    pub deck: Stack,
    pub final_stacks: [Stack; 4],
    pub stacks: [Stack; 7],
    pub turned_deck_index: u8
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
            return &Card {
                number: None,
                suit: 0,
                shown: false,
            };
        }
    }

    pub fn is_empty(&self) -> bool {
        return self.stack.is_empty();
    }

    pub fn length(&self) -> u8 {
        return self.stack.len() as u8;
    }

    pub fn get_card_index(&self, card: &Card) -> u16 {
        for (i, k) in self.get_all_cards().iter().enumerate() {
            if k.suit == card.suit && k.number == card.number {
                return i as u16;
            }
        }
        if card.number.is_none(){
            return 0
        }
        panic!()
    }

    pub fn get_card_from_index(&self, index: u8) -> &Card {
        let card = self.stack.get(index as usize);
        if card.is_some(){
            return card.unwrap();
        }else{
            return &Card{number : None, suit : 0, shown : false}
        }
    }

    pub fn remove_last_card(&mut self) {
        self.stack.pop().unwrap();
    }

    pub fn turn_last_card(&mut self) {
        self.stack.last_mut().unwrap().shown = true
    }
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
            for j in 0..i+1 {
                random_table.stacks[i].add_card_on_top(*mixed_cards.last().unwrap());
                mixed_cards.pop();
            }
            random_table.stacks[i].turn_last_card();
        }
        for k in mixed_cards {
            random_table.deck.add_card_on_top(k);
        }
        random_table.turned_deck_index = random_table.deck.length() - 2;
        return random_table;
    }

    pub fn get_stack_from_pos(&self, cursor: u8) -> &Stack {
        if 6 <= cursor && cursor <= 12 {
            return &self.stacks[(cursor - 6) as usize];
        } else if cursor <= 3 {
            return &self.final_stacks[cursor as usize];
        } else if cursor == 5 || cursor == 6 {
            return &self.deck;
        } else {
            panic!()
        }
    }

    pub fn get_last_three_visible_deck_cards(&self) -> Vec<&Card, 3> {
        let mut res = Vec::<&Card, 3>::new();
        for k in self.turned_deck_index..self.deck.length(){
            res.push(self.deck.get_card_from_index(k));
        }   
        return res
    }

}

/// The entire game is here.
pub fn game(exemple: bool) -> u8 {
    fill_screen(NICE_GREEN);
    let mut table = Table::new_table();
    draw_table(&table);
    loop{}
    return pause_menu(&COLOR_CONFIG, 100);
}
