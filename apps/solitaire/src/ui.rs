use crate::{
    eadk::{
        display::{draw_string, push_rect_uniform},
        Color, Point, Rect,
    },
    game::Card,
};

const NICE_GREEN: Color = Color::from_rgb888(0, 82, 62);
const GRAY: Color = Color::from_rgb888(80, 80, 80);

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

pub fn draw_card(
    card: Card,
    outline: Color,
    outline_size: u16,
    clear: bool,
    abs_pos: Option<Point>,
) {
    let pos;
    if abs_pos.is_some() {
        pos = abs_pos.unwrap();
    } else {
        pos = get_abs_pos(card);
    }
    push_rect_uniform(
        Rect {
            x: pos.x,
            y: pos.y,
            width: 35,
            height: 52,
        },
        if !clear { outline } else { NICE_GREEN },
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
                Point::new(pos.x + 3, pos.y + 3),
                false,
                Color::BLACK,
                Color::WHITE,
            );
        }
    }
}

fn get_abs_pos(card: Card) {}
