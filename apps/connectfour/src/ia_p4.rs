use crate::{
    game_p4::{
        place_coin_nodraw, remove_coin_nodraw, Table, DEFAULT_TABLE_VALUE, MAX_HEIGHT_SIZE,
        MAX_PLAYERS, MAX_WIDTH_SIZE,
    },
    ui_p4::{clear_thinking_ai, draw_thinking_ai},
};
use numworks_utils::{graphical::ColorConfig, utils::randint};

const LOOKING_ORDER: [u8; MAX_WIDTH_SIZE] = [3, 4, 2, 5, 1, 6, 0, 7];
const DEPTH: u8 = 7;
const WIN_SCORE: i16 = 10_000;

const NB_BYTES: usize = 2 * (MAX_WIDTH_SIZE * MAX_HEIGHT_SIZE) / 8;
type Representation = [u8; NB_BYTES];

#[derive(Clone, Copy, PartialEq, Eq)]
enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy)]
struct TTEntry {
    rep: Representation,
    score: i16,
    depth: u8,
    flag: TTFlag,
    occupied: bool,
}

const TT_SIZE: usize = 1024;

struct TranspositionTable {
    entries: [TTEntry; TT_SIZE],
}

impl TranspositionTable {
    const fn new() -> Self {
        Self {
            entries: [TTEntry {
                rep: [0; NB_BYTES],
                score: 0,
                depth: 0,
                flag: TTFlag::Exact,
                occupied: false,
            }; TT_SIZE],
        }
    }

    #[inline(always)]
    fn index(rep: &Representation) -> usize {
        let mut hash: u32 = 0x811c9dc5;
        for &b in rep.iter() {
            hash ^= b as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
        (hash as usize) % TT_SIZE
    }

    #[inline(always)]
    fn get(&self, rep: &Representation, depth: u8, alpha: i16, beta: i16) -> Option<i16> {
        let idx = Self::index(rep);
        let entry = &self.entries[idx];

        if entry.occupied && entry.rep == *rep && entry.depth >= depth {
            match entry.flag {
                TTFlag::Exact => return Some(entry.score),
                TTFlag::LowerBound if entry.score >= beta => return Some(entry.score),
                TTFlag::UpperBound if entry.score <= alpha => return Some(entry.score),
                _ => {}
            }
        }
        None
    }

    #[inline(always)]
    fn store(&mut self, rep: Representation, score: i16, depth: u8, flag: TTFlag) {
        let idx = Self::index(&rep);
        self.entries[idx] = TTEntry {
            rep,
            score,
            depth,
            flag,
            occupied: true,
        };
    }
}

fn hash(table: &Table) -> Representation {
    let mut res: Representation = [0; NB_BYTES];

    for (x, column) in table.iter().enumerate() {
        for (y, v) in column.iter().enumerate() {
            let pos = x * MAX_HEIGHT_SIZE + y;
            let byte = pos / 4;
            let offset = (pos % 4) * 2;
            res[byte] |= (v & 0b11) << offset;
        }
    }

    res
}

pub fn find_best_move(
    table: &Table,
    player: u8,
    nb_players: u8,
    bounds: i16,
    c: &ColorConfig,
) -> u8 {
    let mut frame = 0;
    let mut copy = table.clone();
    let mut tt = TranspositionTable::new();

    let (col, _) = minimax_root(
        &mut copy, player, nb_players, DEPTH, -bounds, bounds, &mut frame, &mut tt, c,
    );

    clear_thinking_ai(c);

    if let Some(c_move) = col {
        c_move
    } else {
        let valid = get_valid_moves(table, nb_players);
        for &choice in LOOKING_ORDER.iter() {
            if choice < (MAX_WIDTH_SIZE - MAX_PLAYERS + nb_players as usize) as u8
                && valid[choice as usize]
            {
                return choice;
            }
        }
        panic!("Sorry, the AI didn't find any move. Sad.");
    }
}

/// Root search specifically handling random tie-breaking among best evaluated moves
fn minimax_root(
    table: &mut Table,
    player: u8,
    nb_players: u8,
    depth: u8,
    mut alpha: i16,
    beta: i16,
    frame: &mut u16,
    tt: &mut TranspositionTable,
    c: &ColorConfig,
) -> (Option<u8>, i16) {
    if let Some(winning_col) = find_winning_moves(table, player, nb_players) {
        return (Some(winning_col), WIN_SCORE);
    }

    let valid_moves = get_valid_moves(table, nb_players);
    let active_width = MAX_WIDTH_SIZE - MAX_PLAYERS + nb_players as usize;
    let next_player = (player + 1) % nb_players;

    let mut candidate_moves = [0u8; MAX_WIDTH_SIZE];
    let mut candidate_count = 0usize;
    let mut best_value = i16::MIN;

    for &choice in LOOKING_ORDER.iter() {
        let col = choice as usize;
        if col >= active_width || !valid_moves[col] {
            continue;
        }

        place_coin_nodraw(choice, player, table);

        let child_eval = minimax(
            table,
            next_player,
            player,
            nb_players,
            depth - 1,
            alpha,
            beta,
            frame,
            tt,
            c,
        );

        if !remove_coin_nodraw(choice, player, table) {
            panic!("Error removing coin !");
        }

        if child_eval > best_value {
            best_value = child_eval;
            candidate_moves[0] = choice;
            candidate_count = 1;
        } else if child_eval == best_value && candidate_count < MAX_WIDTH_SIZE {
            // Collect moves that share the top score so we don't always pick column 3
            candidate_moves[candidate_count] = choice;
            candidate_count += 1;
        }

        if best_value > alpha {
            alpha = best_value;
        }
    }

    if candidate_count > 0 {
        let selected_idx = randint(0, candidate_count as u32) as usize;
        (Some(candidate_moves[selected_idx]), best_value)
    } else {
        (None, best_value)
    }
}

fn minimax(
    table: &mut Table,
    current_player: u8,
    original_player: u8,
    nb_players: u8,
    depth: u8,
    mut alpha: i16,
    mut beta: i16,
    frame: &mut u16,
    tt: &mut TranspositionTable,
    c: &ColorConfig,
) -> i16 {
    if depth == 0 {
        return evaluate_table(table, original_player, nb_players);
    }

    let maximizing = (current_player % nb_players) == original_player;

    if *frame % 256 == 0 {
        draw_thinking_ai(*frame / 256, c);
    }
    *frame += 1;

    let h = hash(table);
    if let Some(cached_score) = tt.get(&h, depth, alpha, beta) {
        return cached_score;
    }

    if let Some(_) = find_winning_moves(table, current_player, nb_players) {
        return if maximizing {
            WIN_SCORE + depth as i16
        } else {
            -WIN_SCORE - depth as i16
        };
    }

    let valid_moves = get_valid_moves(table, nb_players);
    if !valid_moves.iter().any(|b| *b) {
        return 0;
    }

    let alpha_orig = alpha;
    let mut best_value = if maximizing { i16::MIN } else { i16::MAX };
    let next_player = (current_player + 1) % nb_players;
    let active_width = MAX_WIDTH_SIZE - MAX_PLAYERS + nb_players as usize;

    for &choice in LOOKING_ORDER.iter() {
        let col = choice as usize;
        if col >= active_width || !valid_moves[col] {
            continue;
        }

        place_coin_nodraw(choice, current_player, table);

        let child_eval = minimax(
            table,
            next_player,
            original_player,
            nb_players,
            depth - 1,
            alpha,
            beta,
            frame,
            tt,
            c,
        );

        if !remove_coin_nodraw(choice, current_player, table) {
            panic!("Error removing coin !");
        }

        if maximizing {
            if child_eval > best_value {
                best_value = child_eval;
            }
            if best_value > alpha {
                alpha = best_value;
            }
        } else {
            if child_eval < best_value {
                best_value = child_eval;
            }
            if best_value < beta {
                beta = best_value;
            }
        }

        if alpha >= beta {
            break;
        }
    }

    let flag = if best_value <= alpha_orig {
        TTFlag::UpperBound
    } else if best_value >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };

    tt.store(h, best_value, depth, flag);

    best_value
}

fn get_valid_moves(table: &Table, nb_players: u8) -> [bool; MAX_WIDTH_SIZE] {
    let mut result_vec = [false; MAX_WIDTH_SIZE];
    for (i, d) in result_vec
        .iter_mut()
        .enumerate()
        .take(MAX_WIDTH_SIZE - MAX_PLAYERS + nb_players as usize)
    {
        if table[i].iter().any(|a| *a == DEFAULT_TABLE_VALUE) {
            *d = true;
        }
    }
    result_vec
}

const EVALUATION_WEIGHTS: [i16; 4] = [0, 4, 9, 1000];
const ENNEMY_EVALUATION_WEIGHTS: [i16; 4] = [0, 4, 9, 1000];

fn evaluate_table(table: &Table, player: u8, nb_players: u8) -> i16 {
    let mut value: i16 = 0;
    let max_width = MAX_WIDTH_SIZE - MAX_PLAYERS + (nb_players as usize);

    for (i, weight) in EVALUATION_WEIGHTS.iter().enumerate().skip(1) {
        let (aligned, _, _) = look_for_aligned_coins(table, player, i as u8 + 1, max_width);
        value = value.saturating_add(weight.saturating_mul(aligned as i16));
    }

    for d in (0..nb_players).filter(|&a| a != player) {
        for (i, weight) in ENNEMY_EVALUATION_WEIGHTS.iter().enumerate().skip(1) {
            let (aligned, _, _) = look_for_aligned_coins(table, d, i as u8 + 1, max_width);
            value = value.saturating_sub(weight.saturating_mul(aligned as i16));
        }
    }
    value
}

pub fn look_for_aligned_coins(
    table: &Table,
    player: u8,
    number_of_coins: u8,
    max_width: usize,
) -> (u8, (u16, u16), (u16, u16)) {
    let mut result: u8 = 0;
    let mut first_pos: (u16, u16) = (0, 0);
    let mut last_pos: (u16, u16) = (0, 0);

    // Columns
    for x in 0..max_width {
        let column = &table[x];
        let mut count = 0;
        for y in 0..MAX_HEIGHT_SIZE {
            if column[y] == player {
                count += 1;
            } else {
                count = 0;
            }
            if count == number_of_coins {
                if result == 0 {
                    first_pos = (x as u16, y as u16 + 1 - number_of_coins as u16);
                    last_pos = (x as u16, y as u16);
                }
                result += 1;
                count = 0;
            }
        }
    }

    // Rows
    for y in 0..MAX_HEIGHT_SIZE {
        let mut count = 0;
        for x in 0..max_width {
            if table[x][y] == player {
                count += 1;
            } else {
                count = 0;
            }
            if count == number_of_coins {
                if result == 0 {
                    first_pos = (x as u16 + 1 - number_of_coins as u16, y as u16);
                    last_pos = (x as u16, y as u16);
                }
                result += 1;
                count = 0;
            }
        }
    }

    // Up-right diagonals starting at x = 0
    for start_y in 0..MAX_HEIGHT_SIZE {
        let mut count = 0;
        for (x, y) in (0..max_width)
            .map(|t| (t, start_y + t))
            .take_while(|(_, r)| *r < MAX_HEIGHT_SIZE)
        {
            if table[x][y] == player {
                count += 1;
            } else {
                count = 0;
            }
            if count == number_of_coins {
                if result == 0 {
                    first_pos = (
                        x as u16 + 1 - number_of_coins as u16,
                        y as u16 + 1 - number_of_coins as u16,
                    );
                    last_pos = (x as u16, y as u16);
                }
                result += 1;
                count = 0;
            }
        }
    }

    // Up-right diagonals starting at y = 0
    for start_x in 1..max_width {
        let mut count = 0;
        for (x, y) in (0..MAX_HEIGHT_SIZE)
            .map(|t| (start_x + t, t))
            .take_while(|(r, _)| *r < max_width)
        {
            if table[x][y] == player {
                count += 1;
            } else {
                count = 0;
            }
            if count == number_of_coins {
                if result == 0 {
                    first_pos = (
                        x as u16 + 1 - number_of_coins as u16,
                        y as u16 + 1 - number_of_coins as u16,
                    );
                    last_pos = (x as u16, y as u16);
                }
                result += 1;
                count = 0;
            }
        }
    }

    // Down-right diagonals starting at x = 0
    for start_y in 0..MAX_HEIGHT_SIZE {
        let mut count = 0;
        for (x, y) in (0..max_width)
            .take_while(|t| *t <= start_y)
            .map(|t| (t, start_y - t))
        {
            if table[x][y] == player {
                count += 1;
            } else {
                count = 0;
            }
            if count == number_of_coins {
                if result == 0 {
                    first_pos = (
                        x as u16 + 1 - number_of_coins as u16,
                        y as u16 + number_of_coins as u16 - 1,
                    );
                    last_pos = (x as u16, y as u16);
                }
                result += 1;
                count = 0;
            }
        }
    }

    // Down-right diagonals starting at y = MAX_HEIGHT_SIZE - 1
    for start_x in 1..max_width {
        let mut count = 0;
        for (x, y) in (0..MAX_HEIGHT_SIZE)
            .map(|t| (start_x + t, MAX_HEIGHT_SIZE - 1 - t))
            .take_while(|(r, _)| *r < max_width)
        {
            if table[x][y] == player {
                count += 1;
            } else {
                count = 0;
            }
            if count == number_of_coins {
                if result == 0 {
                    first_pos = (
                        x as u16 + 1 - number_of_coins as u16,
                        y as u16 + number_of_coins as u16 - 1,
                    );
                    last_pos = (x as u16, y as u16);
                }
                result += 1;
                count = 0;
            }
        }
    }

    (result, first_pos, last_pos)
}

fn find_winning_moves(table: &mut Table, player: u8, nb_players: u8) -> Option<u8> {
    let valid_moves = get_valid_moves(table, nb_players);
    let max_width = MAX_WIDTH_SIZE - MAX_PLAYERS + nb_players as usize;

    for (col, valid) in valid_moves.iter().enumerate().take(max_width) {
        if *valid {
            place_coin_nodraw(col as u8, player, table);
            let res = look_for_aligned_coins(table, player, 4, max_width).0;
            remove_coin_nodraw(col as u8, player, table);
            if res > 0 {
                return Some(col as u8);
            }
        }
    }
    None
}
