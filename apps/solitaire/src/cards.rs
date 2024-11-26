use heapless::Vec;
use numworks_utils::utils::randint;

use crate::table::Stack;

// The number associated with each values is the position in the tileset.
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

pub const NAMES_LIST: [&str; 13] = [
    "A\0", "2\0", "3\0", "4\0", "5\0", "6\0", "7\0", "8\0", "9\0", "10\0", "J\0", "Q\0", "K\0",
];

pub const TOTAL_CARDS: usize = 52;

pub fn is_card_placable_on_other_card(card_to_move: &Card, base_card: &Card) -> bool {
    if base_card.number == card_to_move.number + 1 {
        match base_card.suit {
            Suit::Club | Suit::Spade => {
                card_to_move.suit == Suit::Diamond || card_to_move.suit == Suit::Heart
            }
            Suit::Diamond | Suit::Heart => {
                card_to_move.suit == Suit::Spade || card_to_move.suit == Suit::Club
            }
        }
    } else {
        false
    }
}

/// Creates a brand new shuffled deck
pub fn create_deck() -> Stack {
    let mut deck = Vec::<Card, TOTAL_CARDS>::new();
    let suit_list = [Suit::Club, Suit::Diamond, Suit::Heart, Suit::Spade];
    for s in &suit_list {
        for e in 0..13 {
            let test = deck.push(Card {
                suit: *s,
                number: e,
                visible: false,
            });
            if test.is_err() {
                panic!("Couldn't put cards in the table !");
            }
        }
    }
    shuffle_deck(&mut deck);
    deck
}

/// Shuffles in-place the deck, with Fisher-Yates algorithm.
fn shuffle_deck(deck: &mut Stack) {
    for i in (1..(TOTAL_CARDS) as u32).rev() {
        let choice = randint(0, i);
        deck.swap(choice as usize, i as usize);
    }
}
