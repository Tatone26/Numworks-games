use crate::eadk::display::{draw_string, push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::eadk::{display, key, keyboard, timing, Color, Point, Rect};
use crate::game::BOOL_OPTIONS_NUMBER;
use crate::utils::{
    draw_centered_string, fading, fill_screen, get_centered_text_left_coordo, LARGE_CHAR_HEIGHT,
    SMALL_CHAR_HEIGHT,
};

/// Used to symbolise the cursor position
#[derive(Debug)]
enum CursorPos {
    START,
    OPTIONS,
    EXIT,
}

/// An Option of T type, with COUNT possible values
#[derive(Debug)]
pub struct MyOption<T, const COUNT: usize> {
    pub name: &'static str,
    pub value: usize,
    pub possible_values: [(T, &'static str); COUNT],
}

impl<T, const COUNT: usize> MyOption<T, COUNT> {
    /// Set the value to the next one, 0 if needed
    pub fn increment_value(&mut self) {
        if self.value == COUNT -1{
            self.value = 0
        }else{
            self.value += 1
        }
    }
    /// Returns the value the option is currently set to
    pub fn get_value(&self) -> &(T, &'static str){
        return &self.possible_values[self.value]
    }
}

/// The position of [START_TXT]
const START_POS: u16 = 120;
/// The position of [OPTIONS_TXT]
const OPTIONS_POS: u16 = 160;
/// The position of [EXIT_TXT]
const EXIT_POS: u16 = 200;

/// The Start option text
const START_TXT: &'static str = "Start\0";
/// The Options option text
const OPTIONS_TXT: &'static str = "Options\0";
/// The Exit option text
const EXIT_TXT: &'static str = "Exit\0";

/// The duration of every fadings in the menus
const FADING_TIME: u16 = 750;

/// Creates a fully fonctional start menu, with [Options][MyOption] !
pub fn menu(
    title: &str,
    opt: &mut [&mut MyOption<bool, 2>; BOOL_OPTIONS_NUMBER],
    text_color: Color,
    background_color: Color,
    selection_color: Color,
) -> u8 {
    fill_screen(background_color);
    draw_centered_string(title, 20u16, true, text_color, background_color);
    draw_selection_string(&CursorPos::START, selection_color, background_color, true);
    draw_selection_string(&CursorPos::OPTIONS, text_color, background_color, false);
    draw_selection_string(&CursorPos::EXIT, text_color, background_color, false);
    let mut cursor_pos: CursorPos = CursorPos::START;
    timing::msleep(300);
    loop {
        let keyboard_state = keyboard::scan();
        if keyboard_state.key_down(key::OK) {
            match &cursor_pos {
                CursorPos::START => {
                    // Fading !
                    fading(FADING_TIME as u32);
                    return 1;
                }
                CursorPos::OPTIONS => {
                    _ = options(opt, text_color, background_color, selection_color);
                    return menu(title, opt, text_color, background_color, selection_color);
                }
                CursorPos::EXIT => {
                    fading(FADING_TIME as u32);
                    return 0;
                }
            }
        } else if keyboard_state.key_down(key::DOWN) | keyboard_state.key_down(key::UP) {
            draw_selection_string(&cursor_pos, text_color, background_color, false);
            if keyboard_state.key_down(key::DOWN) {
                match &cursor_pos {
                    CursorPos::START => cursor_pos = CursorPos::OPTIONS,
                    CursorPos::OPTIONS => cursor_pos = CursorPos::EXIT,
                    CursorPos::EXIT => cursor_pos = CursorPos::START,
                }
            } else if keyboard_state.key_down(key::UP) {
                match &cursor_pos {
                    CursorPos::START => cursor_pos = CursorPos::EXIT,
                    CursorPos::OPTIONS => cursor_pos = CursorPos::START,
                    CursorPos::EXIT => cursor_pos = CursorPos::OPTIONS,
                }
            }
            draw_selection_string(&cursor_pos, selection_color, background_color, true);
            display::wait_for_vblank();
            timing::msleep(200);
        } else if keyboard_state.key_down(key::BACK) {
            return 0;
        }
    }
}

/// Draws the line corresponding to the [CursorPos]
fn draw_selection_string(
    cursor_pos: &CursorPos,
    text_color: Color,
    background_color: Color,
    selected: bool,
) {
    let text: &str;
    let y_pos: u16;
    match cursor_pos {
        CursorPos::START => {
            text = START_TXT;
            y_pos = START_POS;
        }
        CursorPos::OPTIONS => {
            text = OPTIONS_TXT;
            y_pos = OPTIONS_POS;
        }
        CursorPos::EXIT => {
            text = EXIT_TXT;
            y_pos = EXIT_POS;
        }
    }
    draw_centered_string(text, y_pos, true, text_color, background_color);
    push_rect_uniform(
        Rect {
            x: get_centered_text_left_coordo(text, true) - 15,
            y: y_pos + LARGE_CHAR_HEIGHT / 2,
            width: 10,
            height: 2,
        },
        if selected {
            text_color
        } else {
            background_color
        },
    );
}

/// For the [options] menu, the space between each line.
const SPACE_BETWEEN_LINES: u16 = 14;
/// Where in the x coordinate will the names of the [option][MyOption] be placed
const XPOS_NAMES: u16 = 30;
/// Where in the x coordinate will the values ([option][MyOption].value.1) be placed
const XPOS_VALUES: u16 = 170;

/// Create a fully fonctional option menu, which changes directly the [options][MyOption] values. (no option return)
fn options(
    list: &mut [&mut MyOption<bool, 2>; BOOL_OPTIONS_NUMBER],
    text_color: Color,
    background_color: Color,
    selection_color: Color,
) -> u8 {
    fill_screen(background_color);
    draw_centered_string("OPTIONS\0", 20u16, true, text_color, background_color);
    // Only taking care of boolean options for now.
    let first_y: u16;
    let items_number: u16 = list.iter().count() as u16;
    match (SCREEN_HEIGHT - (SCREEN_HEIGHT - LARGE_CHAR_HEIGHT) / 2)
        .checked_sub(((LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) / 2) * items_number)
    {
        None | Some(0) => first_y = 0,
        x_ @ Some(1u16..=u16::MAX) => first_y = x_.unwrap(),
    }
    for item in list.iter().enumerate() {
        let (i, x) = item;
        let y_pos: u16 = first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * (i as u16);
        display::draw_string(
            x.name,
            Point::new(
                XPOS_NAMES,
                y_pos
                    + if x.name.len() > 12 {
                        (LARGE_CHAR_HEIGHT - SMALL_CHAR_HEIGHT) / 2
                    } else {
                        0
                    },
            ),
            x.name.len() < 12,
            text_color,
            background_color,
        );
        draw_options_selection(
            x.get_value().1,
            y_pos,
            if i == 0 { true } else { false },
            selection_color,
            background_color,
            text_color,
        )
    }
    let mut cursor_pos: u16 = 0;
    display::wait_for_vblank();
    timing::msleep(200);
    loop {
        let keyboard_scan = keyboard::scan();
        if keyboard_scan.key_down(key::BACK) {
            break;
        } else if keyboard_scan.key_down(key::UP) | keyboard_scan.key_down(key::DOWN) {
            let current_selection: &MyOption<bool, 2> = &list[cursor_pos as usize];
            draw_options_selection(
                current_selection.get_value().1,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                false,
                selection_color,
                background_color,
                text_color,
            );
            if keyboard_scan.key_down(key::UP) {
                if cursor_pos > 0 {
                    cursor_pos -= 1;
                } else {
                    cursor_pos = (list.len() as u16) - 1;
                }
            } else if keyboard_scan.key_down(key::DOWN) {
                if cursor_pos < list.len() as u16 - 1 {
                    cursor_pos += 1;
                } else {
                    cursor_pos = 0;
                }
            }
            let new_selection: &MyOption<bool, 2> = &list[cursor_pos as usize];
            draw_options_selection(
                new_selection.get_value().1,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                true,
                selection_color,
                background_color,
                text_color,
            );
            display::wait_for_vblank();
            timing::msleep(200);
        } else if keyboard_scan.key_down(key::OK) {
            let selection: &mut MyOption<bool, 2> = list[cursor_pos as usize];
            selection.increment_value();
            draw_options_selection(
                selection.get_value().1,
                first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * cursor_pos,
                true,
                selection_color,
                background_color,
                text_color,
            );
            display::wait_for_vblank();
            timing::msleep(200);
        }
    }
    return 0;
}

/// Draws the line corresponding to the given option value
fn draw_options_selection(
    text: &str,
    ypos: u16,
    selected: bool,
    selection_color: Color,
    background_color: Color,
    text_color: Color,
) {
    draw_string(
        text,
        Point::new(XPOS_VALUES, ypos),
        true,
        if selected {
            selection_color
        } else {
            text_color
        },
        background_color,
    );
    push_rect_uniform(
        Rect {
            x: XPOS_VALUES - 15,
            y: ypos + LARGE_CHAR_HEIGHT / 2,
            width: 10,
            height: 2,
        },
        if selected {
            selection_color
        } else {
            background_color
        },
    );
}

/// The text of the Menu option
const MENU_TXT: &'static str = "Menu\0";
/// How big is the rectangle on the top & bottom, *2 for left and right.
const PAUSE_RECT_SIZE: u16 = 15;

/// Creates a fully foncional pause menu
pub fn pause_menu(text_color: Color, background_color: Color, selection_color: Color, resume_txt: &'static str, menu_offset: (u16, u16)) -> u8 {
    // Le curseur utilise toujours CursorPos bien qu'on ne puisse pas sélectionner "options" ; il est possible que cela soit utile un jour.
    let mut cursor_pos: CursorPos = CursorPos::START;
    let rect_x: u16 = get_centered_text_left_coordo(resume_txt, true);
    display::push_rect_uniform(
        Rect {
            // Can get dirty here !! x and y need to be unsigned, so if < 0 then big boom
            x: rect_x - PAUSE_RECT_SIZE * 2  + menu_offset.0,
            y: (SCREEN_HEIGHT / 2) - (LARGE_CHAR_HEIGHT/2)*3 - SPACE_BETWEEN_LINES - PAUSE_RECT_SIZE  + menu_offset.1,
            width: SCREEN_WIDTH - (rect_x - PAUSE_RECT_SIZE * 2) * 2,
            height: (LARGE_CHAR_HEIGHT*3 + SPACE_BETWEEN_LINES*2 + (PAUSE_RECT_SIZE*2)),
        },
        background_color,
    );
    draw_pause_selection_string(
        &cursor_pos,
        text_color,
        background_color,
        true,
        selection_color,
        resume_txt,
        menu_offset
    );
    draw_pause_selection_string(
        &CursorPos::OPTIONS,
        text_color,
        background_color,
        false,
        selection_color,
        resume_txt,
        menu_offset
    );
    draw_pause_selection_string(
        &CursorPos::EXIT,
        text_color,
        background_color,
        false,
        selection_color,
        resume_txt, 
        menu_offset
    );
    display::wait_for_vblank();
    timing::msleep(200);
    loop {
        let keyboard_state = keyboard::scan();
        if keyboard_state.key_down(key::DOWN) | keyboard_state.key_down(key::UP) {
            draw_pause_selection_string(
                &cursor_pos,
                text_color,
                background_color,
                false,
                selection_color,
                resume_txt, 
                menu_offset
            );
            if keyboard_state.key_down(key::DOWN) {
                match &cursor_pos {
                    CursorPos::START => cursor_pos = CursorPos::OPTIONS,
                    CursorPos::EXIT => cursor_pos = CursorPos::START,
                    CursorPos::OPTIONS => cursor_pos = CursorPos::EXIT,
                }
            } else if keyboard_state.key_down(key::UP) {
                match &cursor_pos {
                    CursorPos::START => cursor_pos = CursorPos::EXIT,
                    CursorPos::OPTIONS => cursor_pos = CursorPos::START,
                    CursorPos::EXIT => cursor_pos = CursorPos::OPTIONS,
                }
            }
            draw_pause_selection_string(
                &cursor_pos,
                text_color,
                background_color,
                true,
                selection_color,
                resume_txt, 
                menu_offset
            );
            display::wait_for_vblank();
            timing::msleep(200);
        } else if keyboard_state.key_down(key::OK) {
            match &cursor_pos {
                CursorPos::START => return 1, // Resume
                CursorPos::OPTIONS => return 2, // Quit to menu
                CursorPos::EXIT => return 0, // Quit the game
            }
        } else if keyboard_state.key_down(key::BACK) {
            return 1;
        }
    }
}

/// Draws the line corresponding to the given [CursorPos] (for pause menu)
fn draw_pause_selection_string(
    cursor_pos: &CursorPos,
    text_color: Color,
    background_color: Color,
    selected: bool,
    selection_color: Color,
    resume_txt: &'static str,
    menu_offset: (u16, u16)
) {
    let text: &str;
    let posy: u16;
    match cursor_pos {
        CursorPos::START => {
            text = resume_txt;
            posy = (SCREEN_HEIGHT / 2) - (LARGE_CHAR_HEIGHT/2)*3 - SPACE_BETWEEN_LINES + menu_offset.1;
        }
        CursorPos::OPTIONS => {
            text = MENU_TXT;
            posy = (SCREEN_HEIGHT / 2) - (LARGE_CHAR_HEIGHT/2) + menu_offset.1;
        }
        CursorPos::EXIT => {
            text = EXIT_TXT;
            posy = (SCREEN_HEIGHT / 2) + (LARGE_CHAR_HEIGHT/2) + SPACE_BETWEEN_LINES + menu_offset.1;
        }
    }
    draw_string(
        text,
        Point::new(get_centered_text_left_coordo(text, true) + menu_offset.0, posy),
        true,
        if selected {
            selection_color
        } else {
            text_color
        },
        background_color,
    );
    push_rect_uniform(
        Rect {
            x: get_centered_text_left_coordo(text, true) - 15  + menu_offset.0,
            y: posy + LARGE_CHAR_HEIGHT / 2,
            width: 10,
            height: 2,
        },
        if selected {
            selection_color
        } else {
            background_color
        },
    );
}
