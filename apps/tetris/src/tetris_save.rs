use heapless::Vec;
use numworks_utils::storage::{
    save::{GameSave, SaveField, MAX_SAVE_FIELDS},
    MAX_STORAGE_VALUES,
};

use crate::{
    game_tetris::{Grid, PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH},
    tetriminos::{get_initial_tetri, TetriType, Tetrimino},
};

pub const TETRIS_SAVE_WORDS: usize = 25;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TetrisSave {
    pub raw: [u32; TETRIS_SAVE_WORDS],
}

impl GameSave for TetrisSave {
    fn to_words(&self) -> Vec<u32, MAX_STORAGE_VALUES> {
        let mut v = Vec::new();
        for &w in &self.raw {
            let _ = v.push(w);
        }
        v
    }

    fn from_words(words: &[u32]) -> Option<Self> {
        if words.len() >= TETRIS_SAVE_WORDS {
            let mut raw = [0u32; TETRIS_SAVE_WORDS];
            raw.copy_from_slice(&words[..TETRIS_SAVE_WORDS]);
            Some(Self { raw })
        } else {
            None
        }
    }

    fn describe_fields(&self) -> Vec<SaveField, MAX_SAVE_FIELDS> {
        let mut fields = Vec::new();
        let _ = fields.push(SaveField {
            name: "Current Score\0",
            min: 0,
            max: 999999,
            value: self.raw[20],
        });
        let _ = fields.push(SaveField {
            name: "Current Level\0",
            min: 1,
            max: 20,
            value: (self.raw[21] & 0xFFFF),
        });
        fields
    }

    fn update_from_fields(&mut self, fields: &[SaveField]) {
        if fields.len() >= 2 {
            self.raw[20] = fields[0].value;
            self.raw[21] = (self.raw[21] & 0xFFFF_0000) | (fields[1].value & 0xFFFF);
        }
    }
}

pub struct TetrisSaveState {
    pub grid: Grid,
    pub score: u32,
    pub level: u16,
    pub level_lines: u16,
    pub current_piece: TetriType,
    pub next_piece: TetriType,
    pub held_piece: Option<TetriType>,
    pub bag: Vec<Tetrimino, 7>,
}

impl TetrisSave {
    pub fn pack(state: &TetrisSaveState) -> Self {
        let mut raw = [0u32; TETRIS_SAVE_WORDS];

        // 1. Pack Grid: 200 cells into raw[0..20] (10 cells per word, 3 bits each)
        // 0 = empty, 1..=7 = color + 1
        let mut cell_idx = 0;
        for y in 0..PLAYFIELD_HEIGHT as usize {
            for x in 0..PLAYFIELD_WIDTH as usize {
                let code = match state.grid.grid[x][y] {
                    None => 0u32,
                    Some(c) => (c as u32 + 1) & 0x07,
                };
                let word_idx = cell_idx / 10;
                let shift = (cell_idx % 10) * 3;
                raw[word_idx] |= code << shift;
                cell_idx += 1;
            }
        }

        // 2. Score
        raw[20] = state.score;

        // 3. Level & Lines
        raw[21] = (state.level as u32) | ((state.level_lines as u32) << 16);

        // 4. Current, Next, Held pieces (4 bits each in raw[22])
        let cur = state.current_piece as u32 & 0x0F;
        let nxt = state.next_piece as u32 & 0x0F;
        let hld = match state.held_piece {
            None => 0x0F,
            Some(p) => p as u32 & 0x0F,
        };
        raw[22] = cur | (nxt << 4) | (hld << 8);

        // 5. Bag contents in raw[23]
        let mut bag_word = state.bag.len() as u32 & 0x0F;
        for (i, t) in state.bag.iter().enumerate() {
            let p_code = t.tetri as u32 & 0x07;
            bag_word |= p_code << (4 + i * 3);
        }
        raw[23] = bag_word;

        TetrisSave { raw }
    }

    pub fn unpack(&self) -> TetrisSaveState {
        let mut grid = Grid::new();

        // 1. Unpack Grid
        let mut cell_idx = 0;
        for y in 0..PLAYFIELD_HEIGHT as usize {
            for x in 0..PLAYFIELD_WIDTH as usize {
                let word_idx = cell_idx / 10;
                let shift = (cell_idx % 10) * 3;
                let code = (self.raw[word_idx] >> shift) & 0x07;
                if code > 0 {
                    grid.grid[x][y] = Some((code - 1) as u8);
                }
                cell_idx += 1;
            }
        }

        // 2. Score, Level, Lines
        let score = self.raw[20];
        let level = (self.raw[21] & 0xFFFF) as u16;
        let level_lines = ((self.raw[21] >> 16) & 0xFFFF) as u16;

        // 3. Current, Next, Held
        let cur = TetriType::from_u8((self.raw[22] & 0x0F) as u8).unwrap_or(TetriType::T);
        let nxt = TetriType::from_u8(((self.raw[22] >> 4) & 0x0F) as u8).unwrap_or(TetriType::J);
        let hld_code = (self.raw[22] >> 8) & 0x0F;
        let held_piece = if hld_code == 0x0F {
            None
        } else {
            TetriType::from_u8(hld_code as u8)
        };

        // 4. Bag
        let bag_len = (self.raw[23] & 0x0F) as usize;
        let mut bag = Vec::<Tetrimino, 7>::new();
        for i in 0..bag_len.min(7) {
            let p_code = ((self.raw[23] >> (4 + i * 3)) & 0x07) as u8;
            if let Some(t_type) = TetriType::from_u8(p_code) {
                let _ = bag.push(get_initial_tetri(t_type));
            }
        }

        TetrisSaveState {
            grid,
            score,
            level,
            level_lines,
            current_piece: cur,
            next_piece: nxt,
            held_piece,
            bag,
        }
    }
}
