use crate::{
    eadk::{
        display::{draw_string, push_rect_uniform},
        Color, Point, Rect,
    },
    game::{Card, Stack, Table},
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
                Point::new(pos.x + 3, pos.y + 2),
                true,
                Color::BLACK,
                Color::WHITE,
            );
        }
    }
}

pub fn get_abs_pos_card(stack: &Stack, pos: u8, index: u16) -> Point {
    let position: Point = get_abs_pos_stack(pos);
    if pos == 4 {
        return position; // TODO
    } else if 6 <= pos {
        if stack.length() > 9 {
            return Point::new(position.x, position.y + 15 * index);
        } else {
            return Point::new(position.x, position.y + 10 * index);
        }
    } else {
        return position;
    }
}

pub fn draw_selection(stack: &Stack, pos: u8, size: u8) {
    let cards = stack.get_all_cards();
    for i in 0..size {
        let k = &cards[i as usize];
        draw_card(
            k,
            SELECT_COLORS[1],
            3,
            false,
            get_abs_pos_card(stack, pos, stack.get_card_index(k)),
        );
    }
}

pub fn clear_selection(stack: &Stack, pos: u8, size: u8) {
    let cards = stack.get_all_cards();
    for i in 0..size {
        let k = &cards[i as usize];
        draw_card(
            k,
            Color::BLACK,
            1,
            false,
            get_abs_pos_card(stack, pos, stack.get_card_index(k)),
        );
    }
}

pub fn draw_stack(stack: &Stack, pos: u8) {
    for k in stack.get_all_cards() {
        draw_card(
            k,
            Color::BLACK,
            1,
            false,
            get_abs_pos_card(stack, pos, stack.get_card_index(k)),
        )
    }
}

fn get_abs_pos_stack(pos: u8) -> Point {
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
        draw_card(
            card,
            Color::BLACK,
            1,
            false,
            get_abs_pos_card(s, i as u8, s.get_card_index(card)),
        );
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
    for i in last_three_visible_cards {
        draw_card(
            i,
            Color::BLACK,
            1,
            false,
            get_abs_pos_card(
                &table.deck,
                4,
                0, // TODO
            ),
        );
    }
    let deck_cards = table.deck.get_all_cards();
    let deck_card = deck_cards.get((table.turned_deck_index - 1) as usize);
    if deck_card.is_some() {
        draw_card(
            deck_card.unwrap(),
            Color::BLACK,
            1,
            false,
            get_abs_pos_stack(5),
        );
    } else {
        draw_card(
            &Card {
                number: None,
                suit: 0,
                shown: false,
            },
            Color::BLACK,
            1,
            false,
            get_abs_pos_stack(5),
        );
    }
}
