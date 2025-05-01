use heapless::String;
use numworks_utils::{
    eadk::{
        display::{wait_for_vblank, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, Color, Point,
    },
    graphical::{draw_centered_string, draw_string_cfg, fading, fill_screen, ColorConfig},
    utils::{string_from_u32, wait_for_no_keydown, CENTER, LARGE_CHAR_HEIGHT, LARGE_CHAR_WIDTH},
};

// This dictates the principal colors that will be used
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: Color::WHITE,
    alt: Color::RED,
};

const NB_GAMES: usize = 5;

const STARTS: [fn(); NB_GAMES] = [
    flappybird::start,
    tetris::start,
    connectfour::start,
    snake::start,
    solitaire::start,
];
const THUMBNAILS: [fn(Point); NB_GAMES] = [
    flappybird::thumbnail,
    tetris::thumbnail,
    connectfour::thumbnail,
    snake::thumbnail,
    solitaire::thumbnail,
];
const NAMES: [fn() -> String<15>; NB_GAMES] = [
    flappybird::get_name,
    tetris::get_name,
    connectfour::get_name,
    snake::get_name,
    solitaire::get_name,
];

pub fn start() {
    fill_screen(COLOR_CONFIG.bckgrd);
    let mut current: usize = 0;
    draw_game(current);
    wait_for_no_keydown();
    loop {
        let scan = keyboard::scan();
        if scan.key_down(key::OK) {
            fading(300);
            STARTS[current]();
            wait_for_vblank();
            draw_game(current);
            wait_for_no_keydown();
        } else if scan.key_down(key::RIGHT) {
            current = (current + 1) % NB_GAMES;
            draw_game(current);
            wait_for_no_keydown();
        } else if scan.key_down(key::LEFT) {
            if current == 0 {
                current = NB_GAMES - 1;
            } else {
                current -= 1;
            }
            draw_game(current);
            wait_for_no_keydown();
        }
    }
}

fn draw_game(i: usize) {
    wait_for_vblank();
    fill_screen(COLOR_CONFIG.bckgrd);
    draw_ui();
    wait_for_vblank();
    THUMBNAILS[i](Point { x: 0, y: 0 });
    let name = NAMES[i]();
    wait_for_vblank();
    draw_centered_string(&name, 2 * SCREEN_HEIGHT / 3, true, &COLOR_CONFIG, true);
    let number = string_from_u32((i + 1) as u32);
    let total = string_from_u32(NB_GAMES as u32);
    draw_string_cfg(
        &number,
        Point {
            x: SCREEN_WIDTH - 60,
            y: SCREEN_HEIGHT - LARGE_CHAR_HEIGHT - 15,
        },
        true,
        &COLOR_CONFIG,
        false,
    );
    draw_string_cfg(
        "/\0",
        Point {
            x: SCREEN_WIDTH - 40,
            y: SCREEN_HEIGHT - LARGE_CHAR_HEIGHT - 15,
        },
        true,
        &COLOR_CONFIG,
        false,
    );
    draw_string_cfg(
        &total,
        Point {
            x: SCREEN_WIDTH - 20,
            y: SCREEN_HEIGHT - LARGE_CHAR_HEIGHT - 15,
        },
        true,
        &COLOR_CONFIG,
        false,
    );
}

fn draw_ui() {
    draw_string_cfg(
        "<\0",
        Point {
            x: 20,
            y: CENTER.y - LARGE_CHAR_HEIGHT / 2,
        },
        true,
        &COLOR_CONFIG,
        false,
    );
    draw_string_cfg(
        ">\0",
        Point {
            x: SCREEN_WIDTH - 20 - LARGE_CHAR_WIDTH,
            y: CENTER.y - LARGE_CHAR_HEIGHT / 2,
        },
        true,
        &COLOR_CONFIG,
        false,
    );
    draw_centered_string(
        "<OK>\0",
        2 * SCREEN_HEIGHT / 3 + LARGE_CHAR_HEIGHT * 2,
        true,
        &COLOR_CONFIG,
        false,
    );
}
