use crate::game_p4::{check, place_coin_nodraw, MAX_HEIGHT_SIZE, MAX_PLAYERS, MAX_WIDTH_SIZE};
use heapless::Vec;
use numworks_utils::utils::randint;

/// Return the best move, but it may not be the best, it depends on depth of the minimax algorithm
pub fn find_best_move(
    table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
    player: u8,
    nb_players: u8,
    depth: u8,
) -> u8 {
    // if you can win, do
    let win = find_winning_moves(table, player, nb_players);
    if let Some(x) = win {
        return x;
    }
    // block one of the next player winning moves
    if let Some(p) = find_winning_moves(table, (player + 1) % nb_players, nb_players) {
        return p;
    }
    // find the best move otherwise
    let (col, _) = minimax(table, player, player, nb_players, i16::MIN, i16::MAX, depth);
    if col.is_none() {
        if get_valid_moves(table)[3] {
            return 3; // default starting move !
        } else {
            panic!("Stupid AI ! can't find a move when there are some !");
        }
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

    let mut best_move: Option<u8> = None;
    let mut valid_moves = get_valid_moves(table);
    for d in valid_moves
        .iter_mut()
        .take(MAX_WIDTH_SIZE)
        .skip(MAX_WIDTH_SIZE - MAX_PLAYERS + nb_players as usize)
    {
        *d = false;
    }
    // full table = terminal node
    if valid_moves.is_empty() {
        // full table, can't play anymore
        return (None, evaluate_table(table, original_player, nb_players));
    }
    // end of game = terminal node
    let check = check(table, nb_players);
    if check.is_some() {
        return (
            None,
            if maximizing {
                i16::MIN + 1 // safety net to make sure it does register those moves
            } else {
                i16::MAX - 1
            },
        );
    }
    // Randomizing which moves it checks first
    let mut moves_done: [bool; MAX_WIDTH_SIZE] = [false; MAX_WIDTH_SIZE];
    while valid_moves
        .iter()
        .enumerate()
        .any(|(i, a)| *a && !moves_done[i])
    {
        let choice = randint(
            0,
            (MAX_WIDTH_SIZE as u32) - MAX_PLAYERS as u32 + nb_players as u32,
        );
        if valid_moves[choice as usize] {
            // loop if already tested
            if moves_done[choice as usize] {
                continue;
            }

            // creating fake table (next node)
            let mut copy = table.clone();
            place_coin_nodraw(choice as u8, player, &mut copy);

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
                    best_move = Some(choice as u8);
                }
                if alpha < value {
                    alpha = value;
                }
                if alpha >= beta {
                    break;
                }
            } else {
                if down_eval < value {
                    // minimizing (adversaries turn)
                    value = down_eval;
                    best_move = Some(choice as u8);
                }
                if beta > value {
                    beta = value;
                }
                if beta <= alpha {
                    break;
                }
            }
            moves_done[choice as usize] = true;
        }
    }
    (best_move, value)
}

/// Return all valid moves (columns not full)
fn get_valid_moves(
    table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
) -> [bool; MAX_WIDTH_SIZE] {
    let mut result_vec = [false; MAX_WIDTH_SIZE];
    for (i, d) in result_vec.iter_mut().enumerate() {
        if table.get(i).unwrap().iter().any(|a| *a == u8::MAX) {
            *d = true;
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
/// Could probably be refactored and sped up ?
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

    // Testing all up-right diagonals starting on x = 0
    for start_y in 0..MAX_HEIGHT_SIZE {
        let mut count = 0;
        for (x, y) in (0..MAX_WIDTH_SIZE)
            .map(|t| (t, start_y + t))
            .take_while(|(_, r)| *r < MAX_HEIGHT_SIZE)
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
            if *table.get(x).unwrap().get(y).unwrap() == player {
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

fn find_winning_moves(
    table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
    player: u8,
    nb_players: u8,
) -> Option<u8> {
    let valid_moves = get_valid_moves(table);
    for (col, valid) in valid_moves
        .iter()
        .enumerate()
        .take(MAX_WIDTH_SIZE - MAX_PLAYERS + nb_players as usize)
    {
        let mut copy = table.clone();
        if *valid {
            place_coin_nodraw(col as u8, player, &mut copy);
            let res = look_for_aligned_coins(
                &copy,
                player,
                4,
                MAX_WIDTH_SIZE - MAX_PLAYERS + nb_players as usize,
            )
            .0;
            if res > 0 {
                return Some(col as u8);
            }
        }
    }
    None
}
