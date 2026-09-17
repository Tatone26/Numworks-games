use heapless::Vec;
use numworks_utils::storage::{
    save::{GameSave, SaveField, MAX_SAVE_FIELDS},
    MAX_STORAGE_VALUES,
};

use crate::{
    cards::{Card, Suit},
    game_solitaire::CursorPos,
    table::Table,
};

const WORDS_COUNT: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SolitaireSave {
    pub raw: [u32; WORDS_COUNT],
}

impl GameSave for SolitaireSave {
    #[inline(always)]
    fn to_words(&self) -> Vec<u32, MAX_STORAGE_VALUES> {
        let mut v = Vec::new();
        for &word in &self.raw {
            let _ = v.push(word);
        }
        v
    }

    #[inline(always)]
    fn from_words(words: &[u32]) -> Option<Self> {
        if words.len() >= WORDS_COUNT {
            let mut raw = [0u32; WORDS_COUNT];
            raw.copy_from_slice(&words[..WORDS_COUNT]);
            Some(Self { raw })
        } else {
            None
        }
    }

    fn describe_fields(&self) -> Vec<SaveField, MAX_SAVE_FIELDS> {
        let mut fields = Vec::new();
        let difficulty = ((self.raw[15] >> 10) & 0x03) as u32;
        let _ = fields.push(SaveField {
            name: "Difficulty\0",
            min: 1,
            max: 3,
            value: difficulty,
        });
        fields
    }

    fn update_from_fields(&mut self, fields: &[SaveField]) {
        if !fields.is_empty() {
            let diff = (fields[0].value.clamp(1, 3) as u32) & 0x03;
            self.raw[15] = (self.raw[15] & !(0x03 << 10)) | (diff << 10);
        }
    }
}

// Clean 8-bit packing per card:
// bits 0..1: suit (0..=3)
// bits 2..5: number (0..=12)
// bit 6:     visible (0 or 1)
#[inline(always)]
fn pack_card(card: &Card) -> u32 {
    (card.suit as u32 & 0x03)
        | (((card.number as u32) & 0x0F) << 2)
        | (((card.visible as u32) & 0x01) << 6)
}

#[inline(always)]
fn unpack_card(val: u32) -> Card {
    let suit = match val & 0x03 {
        0 => Suit::Spade,
        1 => Suit::Heart,
        2 => Suit::Club,
        _ => Suit::Diamond,
    };
    Card {
        suit,
        number: ((val >> 2) & 0x0F) as u8,
        visible: ((val >> 6) & 0x01) != 0,
    }
}

impl Table {
    pub fn to_save(
        &self,
        selection: Option<CursorPos>,
        selection_size: u8,
        difficulty: u16,
    ) -> SolitaireSave {
        let mut raw = [0u32; WORDS_COUNT];

        // 1. Pack 52 cards into raw[0..13] (4 cards per 32-bit word, 8 bits each)
        let mut card_idx = 0;
        let mut write_card = |c: &Card, raw: &mut [u32; WORDS_COUNT]| {
            if card_idx < 52 {
                let word_slot = card_idx / 4;
                let shift = (card_idx % 4) * 8;
                raw[word_slot] |= (pack_card(c) & 0xFF) << shift;
                card_idx += 1;
            }
        };

        for col in &self.tableau {
            for c in col {
                write_card(c, &mut raw);
            }
        }
        for pile in &self.fondations {
            for c in pile {
                write_card(c, &mut raw);
            }
        }
        for c in &self.stock {
            write_card(c, &mut raw);
        }

        // 2. raw[13]: Tableau lengths (7 × 4 bits = 28 bits)
        for (i, col) in self.tableau.iter().enumerate() {
            raw[13] |= ((col.len() as u32) & 0x0F) << (i * 4);
        }

        // 3. raw[14]: Foundations lengths (4 × 4 bits = 16 bits), stock_len (8 bits), stock_iter (8 bits)
        for (i, pile) in self.fondations.iter().enumerate() {
            raw[14] |= ((pile.len() as u32) & 0x0F) << (i * 4);
        }
        raw[14] |= ((self.stock.len() as u32) & 0xFF) << 16;
        raw[14] |= ((self.stock_iter as u32) & 0xFF) << 24;

        // 4. raw[15]: selection tag (2 bits), pos (3 bits), size (5 bits), difficulty (2 bits)
        let (sel_tag, sel_val) = match selection {
            None => (0u32, 0u32),
            Some(CursorPos::Tableau(v)) => (1, v as u32),
            Some(CursorPos::Fondations(v)) => (2, v as u32),
            Some(CursorPos::Stock(v)) => (3, v as u32),
        };
        raw[15] = (sel_tag & 0x03)
            | ((sel_val & 0x07) << 2)
            | (((selection_size as u32) & 0x1F) << 5)
            | (((difficulty as u32) & 0x03) << 10);

        SolitaireSave { raw }
    }

    pub fn from_save(save: &SolitaireSave) -> (Self, Option<CursorPos>, u8, u16) {
        let mut table = Table::empty();

        // 1. Extract metadata from raw[15]
        let meta = save.raw[15];
        let sel_tag = meta & 0x03;
        let sel_val = ((meta >> 2) & 0x07) as u8;
        let selection = match sel_tag {
            1 => Some(CursorPos::Tableau(sel_val)),
            2 => Some(CursorPos::Fondations(sel_val)),
            3 => Some(CursorPos::Stock(sel_val)),
            _ => None,
        };
        let selection_size = ((meta >> 5) & 0x1F) as u8;
        let difficulty = ((meta >> 10) & 0x03) as u16;

        // 2. Unpack cards sequentially (4 cards per word, 8 bits each)
        let mut card_idx = 0;
        let mut read_card = |raw: &[u32; WORDS_COUNT]| -> Card {
            let word_slot = card_idx / 4;
            let shift = (card_idx % 4) * 8;
            let val = (raw[word_slot] >> shift) & 0xFF;
            card_idx += 1;
            unpack_card(val)
        };

        // Populate Tableaux
        for i in 0..7 {
            let count = ((save.raw[13] >> (i * 4)) & 0x0F) as usize;
            for _ in 0..count {
                let _ = table.tableau[i].push(read_card(&save.raw));
            }
        }

        // Populate Foundations
        for i in 0..4 {
            let count = ((save.raw[14] >> (i * 4)) & 0x0F) as usize;
            for _ in 0..count {
                let _ = table.fondations[i].push(read_card(&save.raw));
            }
        }

        // Populate Stock
        let stock_len = ((save.raw[14] >> 16) & 0xFF) as usize;
        for _ in 0..stock_len {
            let _ = table.stock.push(read_card(&save.raw));
        }
        table.stock_iter = ((save.raw[14] >> 24) & 0xFF) as usize;

        (table, selection, selection_size, difficulty)
    }
}
