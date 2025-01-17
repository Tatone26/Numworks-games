use crate::{
    game_p4::{
        place_coin_nodraw, remove_coin_nodraw, Table, DEFAULT_TABLE_VALUE, MAX_HEIGHT_SIZE,
        MAX_PLAYERS, MAX_WIDTH_SIZE,
    },
    ui_p4::{clear_thinking_ai, draw_thinking_ai},
};
use heapless::FnvIndexMap;
use numworks_utils::{
    eadk::{display::push_rect_uniform, Color},
    graphical::ColorConfig,
    utils::randint,
};

// it is always best to look first at the middle then the sides. We should gain quite a bit of speed doing that.
// (and the AI should be more fun to play against)
const LOOKING_ORDER: [u8; MAX_WIDTH_SIZE] = [3, 4, 2, 5, 1, 6, 0, 7];

// The max depth the AI should be looking at. A perfect AI has a depth of WIDTH * HEIGHT but for obvious reasons, we have to limit it.
// 8 seems to be pretty good as it is the minimum to not fall into obvious traps
// and is definitely enough to challenge a real human.
const DEPTH: u8 = 8;
// Totally arbitrary : number of times two random values in the looking order should be swapped. Used to add a little bit of randomisation in the AI.
const SHUFFLES: u8 = 6;

const NB_BYTES: usize = 2 * (MAX_WIDTH_SIZE * MAX_HEIGHT_SIZE) / 8; // number of bytes necessary to represent a grid (2 bits per cell)
type Representation = [u8; NB_BYTES];
type HashMap = FnvIndexMap<Representation, i16, 4096>;

fn hash(table: &Table) -> Representation {
    let mut res: Representation = [0; NB_BYTES];

    for (x, column) in table.iter().enumerate() {
        for (y, v) in column.iter().enumerate() {
            let pos = x * MAX_HEIGHT_SIZE + y;
            let byte = pos / 4; // which byte to modify
            let offset = (pos % 4) * 2; // where in the byte it will be stored
            res[byte] |= (v & 0b11) << offset;
        }
    }

    res
}

struct SearchState {
    player: u8,
    alpha: i16,
    beta: i16,
    depth: u8,
}

/// Return the best move, but it may not be the best, it depends on depth of the minimax algorithm
pub fn find_best_move(
    table: &Table,
    player: u8,
    nb_players: u8,
    bounds: i16,
    c: &ColorConfig,
) -> u8 {
    // find the best move
    let mut frame = 0;

    let mut copy = table.clone();

    let mut hash_map = HashMap::new();

    let search_state = SearchState {
        alpha: -bounds,
        beta: bounds,
        depth: DEPTH,
        player,
    };

    let (col, _) = minimax(
        &mut copy,
        player,
        nb_players,
        &search_state,
        &mut frame,
        &mut hash_map,
        c,
    );

    push_rect_uniform(
        numworks_utils::eadk::Rect {
            x: 0,
            y: 0,
            width: 50,
            height: 100,
        },
        Color::WHITE,
    );

    clear_thinking_ai(c);
    if col.is_none() {
        if get_valid_moves(table, nb_players)[3] {
            return 3; // default move = center !
        } else {
            panic!("Sorry, the AI didn't find any move. Sad.");
        }
    }
    col.unwrap()
}

/// returns (column_number, evaluation)
/// Possible optimisation : do not copy every time, but use mutable and remove coin after placing it.
fn minimax(
    table: &mut Table,
    original_player: u8,
    nb_players: u8,
    search_state: &SearchState,
    frame: &mut u16,
    hash_map: &mut HashMap,
    c: &ColorConfig,
) -> (Option<u8>, i16) {
    // necessary because the calculator is not very fast
    if search_state.depth == 0 {
        return (None, evaluate_table(table, original_player, nb_players));
    }
    let maximizing = (search_state.player % nb_players) == original_player;

    if *frame % 256 == 0 {
        draw_thinking_ai(*frame / 256, c); // needs to be slowed down
    }
    *frame += 1;

    // if this tree branch gave an enemy a victory, stop it.
    if let Some(x) = find_winning_moves(table, search_state.player, nb_players) {
        place_coin_nodraw(x, search_state.player, table);
        let res = (Some(x), evaluate_table(table, original_player, nb_players));
        remove_coin_nodraw(x, search_state.player, table);
        return res;
    };
    // initialisation
    let mut value = if maximizing { i16::MIN } else { i16::MAX };
    let mut best_move: Option<u8> = None;

    let mut next_search_state = SearchState {
        alpha: search_state.alpha,
        beta: search_state.beta,
        player: (search_state.player + 1) % nb_players,
        depth: search_state.depth - 1,
    };

    // get valid moves
    let valid_moves = get_valid_moves(table, nb_players);
    if !valid_moves.iter().any(|b| *b) {
        // the game is a draw
        return (None, 0);
    }

    let mut moves = LOOKING_ORDER;
    shuffle_moves(&mut moves, SHUFFLES);

    // look at all possible moves, in the good order, and do recursion
    for &choice in moves.iter().filter(|p| valid_moves[**p as usize]) {
        // creating fake table (next node)
        place_coin_nodraw(choice, search_state.player, table);

        let h = hash(table);

        let down_eval = if let Some(x) = hash_map.get(&h) {
            *x
        } else {
            // Get the evaluation of the child node
            let down_eval = minimax(
                table,
                original_player,
                nb_players,
                &next_search_state,
                frame,
                hash_map,
                c,
            )
            .1;

            if hash_map.insert(h, down_eval).is_err() {
                if let Some((rep, x)) = hash_map
                    .iter()
                    .max_by_key(|(_, b)| b.abs())
                    // Removing the state that is closest to end because those are really fast to compute thanks to min-max.
                    // this choice could easily be perfected, I guess. But it is already cutting a bit of computing !
                    .map(|(a, b)| (*a, *b))
                {
                    if down_eval.abs() < x.abs() {
                        hash_map.remove(&rep);
                        hash_map.insert(h, down_eval).unwrap();
                    }
                }
            }

            down_eval
        };

        if !remove_coin_nodraw(choice, search_state.player, table) {
            panic!("Error removing coin !");
        }
        // go back up the tree
        if maximizing {
            // if we found a better option that we had
            if down_eval > value {
                value = down_eval;
                best_move = Some(choice);
            }
            // upgrade alpha
            if next_search_state.alpha < value {
                next_search_state.alpha = value;
            }
        } else {
            if down_eval < value {
                value = down_eval;
                best_move = Some(choice);
            }
            // downgrade beta
            if next_search_state.beta > value {
                next_search_state.beta = value;
            }
        }
        // if score window is empty, then we won't find anything better.
        if next_search_state.alpha >= next_search_state.beta {
            break;
        }
    }
    (best_move, value)
}

/// Return all valid moves (columns not full)
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

/// Gives a value for a given table : bonuses for "good" placing (3 in a row, 4 in a row etc) and removes from the value the enemies evaluation
fn evaluate_table(table: &Table, player: u8, nb_players: u8) -> i16 {
    let mut value: i16 = 0;
    // find the number of aligned coins and add it to the value (what is important is the change between game states -> blocked alignment are not important and can be counted)
    for (i, weight) in EVALUATION_WEIGHTS.iter().enumerate().skip(1) {
        value = value.saturating_add(
            weight.saturating_mul(
                look_for_aligned_coins(
                    table,
                    player,
                    i as u8 + 1,
                    MAX_WIDTH_SIZE - MAX_PLAYERS + (nb_players as usize),
                )
                .0 as i16,
            ),
        );
    }
    // do the same for enemy
    for d in (0..nb_players).filter(|&a| a != player) {
        for (i, weight) in ENNEMY_EVALUATION_WEIGHTS.iter().enumerate().skip(1) {
            value = value.saturating_sub(
                weight.saturating_mul(
                    look_for_aligned_coins(
                        table,
                        d,
                        i as u8 + 1,
                        MAX_WIDTH_SIZE - MAX_PLAYERS + (nb_players as usize),
                    )
                    .0 as i16,
                ),
            );
        }
    }
    value
}

/// Returns the number of times x coins (of this player) have been seen aligned, and chere is this alignment
/// Could probably be refactored and sped up ?
pub fn look_for_aligned_coins(
    table: &Table,
    player: u8,
    number_of_coins: u8,
    max_width: usize,
) -> (u8, (u16, u16), (u16, u16)) {
    let mut result: u8 = 0;
    let mut first_pos: (u16, u16) = (0, 0);
    let mut last_pos: (u16, u16) = (0, 0);

    // we start by checking the columns
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
    // then we look on the lines
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

    // Testing all up-right diagonals starting on x = 0
    for start_y in 0..MAX_HEIGHT_SIZE {
        let mut count = 0;
        for (x, y) in (0..MAX_WIDTH_SIZE)
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
    // Testing all up-right diagonals starting on x = 0 (skipping the first one)
    for start_x in 1..MAX_WIDTH_SIZE {
        let mut count = 0;
        for (x, y) in (0..MAX_HEIGHT_SIZE)
            .map(|t| (start_x + t, t))
            .take_while(|(r, _)| *r < MAX_WIDTH_SIZE)
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

    // Testing all down-right diagonals starting on x = 0
    for start_y in 0..MAX_HEIGHT_SIZE {
        let mut count = 0;
        for (x, y) in (0..MAX_WIDTH_SIZE)
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
    // Testing all down-right diagonals starting on x = 0 (skipping the first one)
    for start_x in 1..MAX_WIDTH_SIZE {
        let mut count = 0;
        for (x, y) in (0..MAX_HEIGHT_SIZE)
            .map(|t| (start_x + t, MAX_HEIGHT_SIZE - 1 - t))
            .take_while(|(r, _)| *r < MAX_WIDTH_SIZE)
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
    for (col, valid) in valid_moves.iter().enumerate() {
        if *valid {
            place_coin_nodraw(col as u8, player, table);
            let res = look_for_aligned_coins(
                table,
                player,
                4,
                MAX_WIDTH_SIZE - MAX_PLAYERS + nb_players as usize,
            )
            .0;
            remove_coin_nodraw(col as u8, player, table);
            if res > 0 {
                return Some(col as u8);
            }
        }
    }
    None
}

/// Shuffles two random moves n times. Just so the AI is not doing the same thing every time...
fn shuffle_moves(moves: &mut [u8; MAX_WIDTH_SIZE], n: u8) {
    for _ in 0..n {
        let random_one = randint(0, MAX_WIDTH_SIZE as u32);
        let random_two = randint(0, MAX_WIDTH_SIZE as u32);
        moves.swap(random_one as usize, random_two as usize);
    }
}
