use heapless::Vec;
use numworks_utils::utils::randint;

use crate::game_p4::{check, place_coin_nodraw, MAX_HEIGHT_SIZE, MAX_WIDTH_SIZE};

/* fn find_best_move(table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>, player: u8) -> u8 {
    let ending_moves = find_ending_moves(table, player);
    if !ending_moves.is_empty() {
        return ending_moves[randint(0, ending_moves.len() as u32 - 1) as usize];
    } else {
        return minimax(table, player, true);
    }
    return 0;
} */

fn minimax(
    table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
    player: u8,
    maximizing: bool,
    depth: u8,
) -> u8 {
    if depth == 0 {
        return 0; // TODO
    }
    if maximizing {
        let value: i16 = -3000;
        let valid_moves = get_valid_moves(table, player);
        for col in valid_moves {
            let mut copy = table.clone();
            place_coin_nodraw(col as u16, player, &mut copy);
            // TODO : continue lol
        }
    }
    return 0;
}

fn get_valid_moves(
    table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
    player: u8,
) -> Vec<u8, MAX_WIDTH_SIZE> {
    let mut result_vec = Vec::<u8, MAX_WIDTH_SIZE>::new();
    for i in 0..MAX_WIDTH_SIZE {
        if !table[i].is_full() {
            let _ = result_vec.push(i as u8);
        }
    }
    return result_vec;
}

fn find_ending_moves(
    table: &Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE>,
    player: u8,
) -> Vec<u8, MAX_WIDTH_SIZE> {
    let mut return_vec: Vec<u8, MAX_WIDTH_SIZE> = Vec::new();
    let valid_moves = get_valid_moves(table, player);
    for i in 0..MAX_WIDTH_SIZE {
        if valid_moves.contains(&(i as u8)) {
            let mut copied_table: Vec<Vec<u8, MAX_HEIGHT_SIZE>, MAX_WIDTH_SIZE> = table.clone();
            place_coin_nodraw(i as u16, player, &mut copied_table);
            if check(&copied_table).is_some() {
                let _ = return_vec.push(i as u8);
            }
        }
    }
    return return_vec;
}
