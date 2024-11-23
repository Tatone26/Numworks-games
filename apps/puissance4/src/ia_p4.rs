use core::{i16, option::Iter};

use crate::game_p4::{
    check, place_coin_nodraw, start, MAX_HEIGHT_SIZE, MAX_PLAYERS, MAX_WIDTH_SIZE,
};
use heapless::Vec;
use numworks_utils::utils::randint;

/// Return the best move, but it may not be the best, it depends on depth of the minimax algorithm
pub fn find_best_move(
    table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
    player: u8,
    nb_players: u8,
    depth: u8,
) -> u8 {
    let (col, _) = minimax(table, player, player, nb_players, i16::MIN, i16::MAX, depth);
    if col.is_none() {
        panic!("the IA couldn't find a move !");
    }
    col.unwrap()
}

/// returns (column_number, evaluation)
fn minimax(
    table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
    original_player: u8,
    player: u8,
    nb_players: u8,
    alpha: i16,
    beta: i16,
    depth: u8,
) -> (Option<u8>, i16) {
    if depth == 0 {
        // base case of recursion
        return (None, evaluate_table(table, original_player, nb_players));
    }
    let maximizing = (player % nb_players) == original_player;
    let mut value = if maximizing { i16::MIN } else { i16::MAX };
    let mut alpha = alpha;
    let mut beta = beta;

    let mut best_move: u8 = 0;
    let valid_moves = get_valid_moves(table);
    // full table = terminal node
    if valid_moves.is_empty() {
        // full table, can't play anymore
        return (None, evaluate_table(table, original_player, nb_players));
    }
    // end of game = terminal node
    let check = check(table, nb_players);
    if let Some(x) = check {
        if x.0 == player {
            return (None, if maximizing { i16::MAX } else { i16::MIN });
        }
    }
    // Randomizing which moves it checks first
    let mut moves_done: Vec<u8, MAX_WIDTH_SIZE> = Vec::new();
    while valid_moves.iter().any(|a| !moves_done.contains(a)) {
        let choice = randint(0, MAX_WIDTH_SIZE as u32);
        if let Some(&col) = valid_moves.get(choice as usize) {
            // loop if already tested
            if moves_done.contains(&col) {
                continue;
            }

            // creating fake table (next node)
            let mut copy = table.clone();
            place_coin_nodraw(col, player, &mut copy);

            // Get the evaluation of the child node
            // every player tries to minimize the gain of the AI (which will translate to trying to maximize their own)
            // while the AI tries to maximize it
            let (_, down_eval) = minimax(
                &copy,
                original_player,
                (player + 1) % nb_players,
                nb_players,
                alpha,
                beta,
                depth - 1,
            );
            if maximizing {
                if down_eval > value {
                    // maximizing
                    value = down_eval;
                    best_move = col;
                }
                if alpha < value {
                    alpha = value;
                }
                if alpha >= beta {
                    break;
                }
            } else {
                if value == i16::MIN || down_eval < value {
                    // minimizing (adversaries turn)
                    value = down_eval;
                    best_move = col;
                }
                if beta > value {
                    beta = value;
                }
                if beta <= alpha {
                    break;
                }
            }
            moves_done.push(col).unwrap();
        }
    }
    (Some(best_move), value)
}

/// Return all valid moves (columns not full)
fn get_valid_moves(
    table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
) -> Vec<u8, MAX_WIDTH_SIZE> {
    let mut result_vec = Vec::<u8, MAX_WIDTH_SIZE>::new();
    for i in 0..MAX_WIDTH_SIZE {
        if table.get(i).unwrap().iter().any(|a| *a == u8::MAX) {
            let _ = result_vec.push(i as u8);
        }
    }
    result_vec
}

/// Weights, need to be fixed manually or find stuff online
const EVALUATION_WEIGHTS: [i16; 4] = [0, 50, 100, 5000];
const ENNEMY_EVALUATION_WEIGHTS: [i16; 4] = [0, 10, 70, 5000];

/// Gives a value for a given table : bonuses for "good" placing (3 in a row, 4 in a row etc) and removes from the value the enemies evaluation
fn evaluate_table(
    table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
    player: u8,
    nb_players: u8,
) -> i16 {
    let mut value: i16 = 0;
    // find the number of aligned coins and add it to the value (what is important is the change between game states -> blocked alignment are not important and can be counted)
    for (i, weight) in EVALUATION_WEIGHTS.iter().enumerate().skip(1) {
        value = value.saturating_add(
            weight
                * look_for_aligned_coins(
                    table,
                    player,
                    i as u8 + 1,
                    MAX_WIDTH_SIZE - MAX_PLAYERS + (nb_players as usize),
                )
                .0 as i16,
        );
    }
    // end : check the table for the other players. You don't want to help them.
    for d in (0..nb_players).filter(|&a| a != player) {
        for (i, weight) in ENNEMY_EVALUATION_WEIGHTS.iter().enumerate().skip(1) {
            value = value.saturating_sub(
                weight
                    * look_for_aligned_coins(
                        table,
                        d,
                        i as u8 + 1,
                        MAX_WIDTH_SIZE - MAX_PLAYERS + (nb_players as usize),
                    )
                    .0 as i16,
            );
        }
    }
    value
}

/// Returns the number of times x coins (of this player) have been seen aligned, and chere is this alignment
/// It is pretty stupid on diagonals, but since we are using the same everytime so errors should compensate, right ?
/// need to redo diagonals checks later
pub fn look_for_aligned_coins(
    table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
    player: u8,
    number_of_coins: u8,
    max_width: usize,
) -> (u8, (u16, u16), (u16, u16)) {
    let mut result: u8 = 0;
    let mut first_pos: (u16, u16) = (0, 0);
    let mut last_pos: (u16, u16) = (0, 0);

    // we start by checking the columns
    for x in 0..max_width {
        let column = table.get(x).unwrap();
        let mut count = 0;
        for y in 0..MAX_HEIGHT_SIZE {
            if *column.get(y).unwrap() == player {
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
            if *table.get(x).unwrap().get(y).unwrap() == player {
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

    let up_right_x = (0..MAX_HEIGHT_SIZE)
        .map(|start_y| (0..(MAX_HEIGHT_SIZE - start_y)).map(move |t| (t, start_y + t)));
    let up_right_y = (1..MAX_WIDTH_SIZE)
        .map(|start_x| (0..(MAX_WIDTH_SIZE - start_x)).map(move |t| (start_x + t, t)));
    let down_right_x =
        (0..MAX_HEIGHT_SIZE).map(|start_y| (0..(start_y + 1)).map(move |t| (t, start_y - t)));
    let down_right_y = (1..MAX_WIDTH_SIZE).map(|start_x| {
        (0..(MAX_WIDTH_SIZE - start_x)).map(move |t| (start_x + t, MAX_HEIGHT_SIZE - 1 - t))
    });
    let diag_test: &[&dyn Iterator<Item = dyn Iterator<Item = (usize, usize)>>] =
        [&up_right_x, &up_right_y, &down_right_x, &down_right_y];

    // Testing all down-right diagonals starting on x = 0 (skipping the first one)
    for start_x in 1..MAX_WIDTH_SIZE {
        let mut count = 0;
        for (x, y) in (0..MAX_HEIGHT_SIZE)
            .map(|t| (start_x + t, MAX_HEIGHT_SIZE - 1 - t))
            .take_while(|(r, _)| *r < MAX_WIDTH_SIZE)
        {
            if *table.get(x).unwrap().get(y).unwrap() == player {
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
