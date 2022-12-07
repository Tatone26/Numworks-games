use crate::{
    eadk::{
        display::{draw_string, push_rect_uniform},
        Color, Point, Rect,
    },
    game::{Card, Stack, Table, NONE_CARD},
    utils::fill_screen,
};

pub const NICE_GREEN: Color = Color::from_rgb888(0, 82, 62);
pub const GRAY: Color = Color::from_rgb888(80, 80, 80);

const SUIT_COLORS: [Color; 4] = [
    Color::RED,
    Color::from_rgb888(220, 120, 0),
    Color::BLUE,
    Color::from_rgb888(120, 0, 220),
];
const LETTERS: [&'static str; 13] = [
    "1\0", "2\0", "3\0", "4\0", "5\0", "6\0", "7\0", "8\0", "9\0", "10\0", "J\0", "Q\0", "K\0",
];
const SELECT_COLORS: [Color; 2] = [
    Color::from_rgb888(18, 213, 7),
    Color::from_rgb888(220, 100, 0),
];

pub fn draw_card(card: &Card, outline: Color, outline_size: u16, clear: bool, pos: Point) {
    push_rect_uniform(
        Rect {
            x: pos.x,
            y: pos.y,
            width: 35,
            height: 52,
        },
        if !clear { outline } else { Color::GREEN },
    );
    if !clear {
        push_rect_uniform(
            Rect {
                x: pos.x + outline_size,
                y: pos.y + outline_size,
                width: 35 - 2 * outline_size,
                height: 52 - 2 * outline_size,
            },
            if card.number.is_none() {
                NICE_GREEN
            } else if card.shown {
                Color::WHITE
            } else {
                GRAY
            },
        );
        if card.shown & (card.number.is_some()) {
            push_rect_uniform(
                Rect {
                    x: pos.x + 17,
                    y: pos.y + 3,
                    width: 15,
                    height: 15,
                },
                SUIT_COLORS[card.suit as usize],
            );
            push_rect_uniform(
                Rect {
                    x: pos.x + 3,
                    y: pos.y + 34,
                    width: 15,
                    height: 15,
                },
                SUIT_COLORS[card.suit as usize],
            );
            draw_string(
                LETTERS[(card.number.unwrap() - 1) as usize],
                Point::new(pos.x + 4, pos.y + 3),
                true,
                Color::BLACK,
                Color::WHITE,
            );
        }
    }
}

/// Draws any card, not selected and from a normal stack.
pub fn draw_normal_card(card: &Card, stack: &Stack, pos: u8) {
    draw_card(
        card,
        Color::BLACK,
        1,
        false,
        get_abs_pos_card(stack, pos, stack.get_card_index(card)),
    );
}

pub fn get_abs_pos_card(stack: &Stack, pos: u8, index: u16) -> Point {
    let position: Point = get_abs_pos_stack(pos);
    if pos == 4 {
        return position; // TODO
    } else if 6 <= pos {
        if stack.len() > 9 {
            return Point::new(
                position.x,
                position.y + 15 * (stack.len() as u16 - index),
            );
        } else {
            return Point::new(
                position.x,
                position.y + 10 * (stack.len() as u16 - index),
            );
        }
    } else {
        return position;
    }
}

pub fn draw_selection(stack: &Stack, pos: u8, size: u8, selected: bool) {
    for i in 0..size {
        let k = stack.get_card_from_index(i);
        draw_selection_card(
            k,
            selected,
            get_abs_pos_card(stack, pos, stack.get_card_index(k)),
        );
    }
}

pub fn draw_selection_card(card: &Card, selected: bool, abs_pos: Point) {
    draw_card(
        card,
        if selected {
            SELECT_COLORS[1]
        } else {
            SELECT_COLORS[0]
        },
        3,
        false,
        abs_pos,
    );
}

pub fn clear_selection(stack: &Stack, pos: u8, size: u8) {
    for i in 0..size {
        let k = stack.get_card_from_index(i);
        draw_normal_card(k, stack, pos);
    }
}

pub fn draw_stack(stack: &Stack, pos: u8) {
    for k in stack.get_all_cards().iter().rev() {
        draw_normal_card(k, stack, pos);
    }
}

pub fn get_abs_pos_stack(pos: u8) -> Point {
    if 6 <= pos && pos <= 12 {
        return Point::new(15 + 42 * (pos as u16 - 6), 70);
    } else if pos <= 3 {
        return Point::new(20 + (pos as u16) * 38, 10);
    } else if pos == 4 {
        return Point::new(223, 10);
    } else if pos == 5 {
        return Point::new(275, 10);
    } else {
        panic!()
    }
}

pub fn draw_table(table: &Table) {
    fill_screen(NICE_GREEN);
    for (i, s) in table.final_stacks.iter().enumerate() {
        let card = s.get_last_card();
        draw_normal_card(card, s, i as u8);
        push_rect_uniform(
            Rect {
                x: 22 + 38 * (i as u16),
                y: 5,
                width: 31,
                height: 3,
            },
            SUIT_COLORS[i],
        );
    }
    for (i, s) in table.stacks.iter().enumerate() {
        draw_stack(&s, (i + 6) as u8)
    }
    let last_three_visible_cards = table.get_last_three_visible_deck_cards();
    for k in last_three_visible_cards {
        draw_normal_card(k, &table.deck, 4);
    }
    let deck_cards = table.deck.get_all_cards();
    let deck_card = deck_cards.get((table.turned_deck_index - 1) as usize);
    if deck_card.is_some() {
        draw_normal_card(deck_card.unwrap(), &table.deck, 5);
    } else {
        draw_card(&NONE_CARD, Color::BLACK, 1, false, get_abs_pos_stack(5));
    }
}
