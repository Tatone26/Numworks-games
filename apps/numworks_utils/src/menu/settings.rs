use heapless::{String, Vec};

use crate::{
    eadk::{
        display::{self, push_rect_uniform, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, timing, Point, Rect,
    },
    graphical::{draw_centered_string, draw_string_cfg, fill_screen, ColorConfig},
    storage::{read_file, write_data, MAX_STORAGE_VALUES},
    utils::{
        get_string_pixel_size, string_from_u32, wait_for_no_keydown, LARGE_CHAR_HEIGHT,
        SMALL_CHAR_HEIGHT,
    },
    widgets::numberinput::numberinput,
};

use super::{REPETITION_SPEED, SPACE_BETWEEN_LINES};

// The maximum number of different values an option can take.
pub const MAX_SETTINGS_VALUES: usize = MAX_STORAGE_VALUES;

/// Where in the x coordinate will the names of the [option][Setting] be placed
const XPOS_NAMES: u16 = 30;
/// Where in the x coordinate will the values ([option][Setting].value.1) be placed
const XPOS_VALUES: u16 = 170;

/// An Option of T type, with COUNT possible values
/// Old values are represented as u32.
#[derive(Debug)]
pub struct Setting {
    pub name: &'static str,
    pub choice: usize, // the selected position in the arrays
    pub values: Vec<u32, MAX_SETTINGS_VALUES>,
    pub texts: Vec<&'static str, MAX_SETTINGS_VALUES>,
    pub user_modifiable: bool,
    pub fixed_values: bool,
    // If true : will iterate through the values defined in the vec. If false : will use a number-choosing widget with values[1] and values[2] as the boundaries. (TODO)
    // false will probably mostly be used to store data like high scores.
}

impl Setting {
    /// Set the value to the next one, 0 if needed
    fn increment_value(&mut self) {
        if self.choice == self.values.len() - 1 {
            self.choice = 0
        } else {
            self.choice += 1
        }
    }

    fn decrement_value(&mut self) {
        if self.choice == 0 {
            self.choice = self.values.len() - 1
        } else {
            self.choice -= 1
        }
    }

    pub fn set_value(&mut self, value: u32) {
        self.values[if self.fixed_values { 0 } else { self.choice }] = value;
    }

    #[inline(always)]
    fn get_text(&self) -> &'static str {
        self.texts[self.choice]
    }

    pub fn get_setting_value(&self) -> u32 {
        self.values[self.choice]
    }
}

pub(super) fn set_values_from_file(list: &mut [&mut Setting], filename: &str) {
    let v = read_file(filename);
    if v.is_empty() {
        return;
    }
    for (x, y) in list.iter_mut().zip(v.iter()) {
        if x.fixed_values {
            x.choice = *y as usize;
        } else {
            x.choice = 0;
            x.set_value(*y);
        }
    }
}

pub fn write_values_to_file(list: &[&mut Setting], filename: &str) {
    for (i, v) in list.iter().enumerate() {
        if v.fixed_values {
            write_data(filename, Some(i as u32), v.choice as u32);
        } else {
            write_data(filename, Some(i as u32), v.values[0]);
        }
    }
}

const MAX_ITEMS_PER_PAGE: u16 = 5;

/// Formats current/total pages
fn format_page_counter(current_page: u16, total_pages: u16) -> String<16> {
    use core::fmt::Write;
    let mut str = String::<16>::new();
    let _ = write!(str, "{}/{}\0", current_page + 1, total_pages);
    str
}

/// Computes the top Y coordinate for the settings list on the current page.
#[inline]
fn calculate_first_y(items_in_page: u16) -> u16 {
    match (SCREEN_HEIGHT + LARGE_CHAR_HEIGHT)
        .checked_sub((LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * items_in_page)
    {
        None | Some(0) => 0,
        x_ @ Some(1u16..=u16::MAX) => x_.unwrap() / 2,
    }
}

/// Computes the Y coordinate of an item given its slot index within the page.
#[inline]
fn item_y(first_y: u16, slot: u16) -> u16 {
    first_y + (LARGE_CHAR_HEIGHT + SPACE_BETWEEN_LINES) * slot
}

/// Returns the number of items displayed on a given page.
#[inline]
fn items_on_page(page: u16, total_items: u16) -> u16 {
    let start = page * MAX_ITEMS_PER_PAGE;
    if start >= total_items {
        0
    } else {
        (total_items - start).min(MAX_ITEMS_PER_PAGE)
    }
}

/// Draws a full settings page and returns the calculated `first_y`.
fn draw_settings_page(
    visible: &[&mut Setting],
    page: u16,
    pages: u16,
    cursor_pos: u16,
    cfg: &ColorConfig,
) -> u16 {
    let count = items_on_page(page, visible.len() as u16);
    let first_y = calculate_first_y(count);

    // visual things
    display::wait_for_vblank();
    fill_screen(cfg.bckgrd);
    draw_centered_string("SETTINGS\0", 5u16, true, cfg, false);
    let back_text = "Menu : <Back>  \0";
    draw_string_cfg(
        back_text,
        Point::new(
            SCREEN_WIDTH - get_string_pixel_size(back_text, false) - 5,
            SCREEN_HEIGHT - SMALL_CHAR_HEIGHT - 5,
        ),
        false,
        cfg,
        false,
    );

    // Page counter: print "x/x" in the bottom-left corner only when there are multiple pages
    if pages > 1 {
        let page_str = format_page_counter(page, pages);
        draw_string_cfg(
            &page_str,
            Point::new(10, SCREEN_HEIGHT - SMALL_CHAR_HEIGHT - 5),
            false,
            cfg,
            false,
        );
    }

    // Page indicator: show '^' if there is a previous page
    if page > 0 {
        draw_centered_string("^\0", first_y - LARGE_CHAR_HEIGHT, true, cfg, false);
    }

    // printing the options
    let start = (page * MAX_ITEMS_PER_PAGE) as usize;
    for (i, item) in visible[start..start + count as usize].iter().enumerate() {
        let y_pos = item_y(first_y, i as u16);
        let large: bool = get_string_pixel_size(item.name, true) + XPOS_NAMES
            < XPOS_VALUES - LARGE_CHAR_HEIGHT * 2;
        display::draw_string(
            item.name,
            Point::new(
                XPOS_NAMES,
                y_pos
                    + if !large {
                        (LARGE_CHAR_HEIGHT - SMALL_CHAR_HEIGHT) / 2
                    } else {
                        0
                    },
            ),
            large,
            cfg.text,
            cfg.bckgrd,
        );
        draw_setting_selection(item, y_pos, i as u16 == cursor_pos, cfg);
    }

    // Page indicator: show 'v' if there is a next page
    if page + 1 < pages {
        let last_y = item_y(first_y, count - 1);
        draw_centered_string("v\0", last_y + LARGE_CHAR_HEIGHT + 5, true, cfg, false);
    }

    first_y
}

/// Create a fully fonctional settings menu, which changes directly the [options][Setting] values. (no settings return)
/// For now, only allows a limited number of settings because making multiple pages is too complicated and I don't have time for that
pub(crate) fn settings(
    list: &mut [&mut Setting],
    cfg: &ColorConfig,
    filename: &str,
    godmode: bool,
) {
    // Collect active mutable references so we don't repeatedly filter in hot loops
    {
        let mut visible: heapless::Vec<&mut Setting, MAX_SETTINGS_VALUES> = list
            .iter_mut()
            .filter_map(|s| {
                if godmode || s.user_modifiable {
                    Some(&mut **s)
                } else {
                    None
                }
            })
            .collect();

        let items_number = visible.len() as u16;
        if items_number == 0 {
            return;
        }
        let pages: u16 = (items_number + MAX_ITEMS_PER_PAGE - 1) / MAX_ITEMS_PER_PAGE; // 1 -> 1 ; 5 -> 1 ; 6 -> 2 : good !

        let mut current_page: u16 = 0;
        let mut cursor_pos: u16 = 0;
        let mut first_y = draw_settings_page(&visible, current_page, pages, cursor_pos, cfg);

        setting_selection(
            &mut visible,
            cfg,
            &mut first_y,
            items_number,
            pages,
            &mut current_page,
            &mut cursor_pos,
        );
    }

    write_values_to_file(list, filename);
}

/// Takes care of all the difficult stuff, like moving the cursor and modifying the text.
///
/// Similar to [super::selection] but redone to account for more things (and not account for horizontal versions)
fn setting_selection(
    visible: &mut [&mut Setting],
    cfg: &ColorConfig,
    first_y: &mut u16,
    items_number: u16,
    pages: u16,
    current_page: &mut u16,
    cursor_pos: &mut u16,
) {
    wait_for_no_keydown();

    let mut last_action: u64 = timing::millis();
    let mut last_action_key: u32 = key::ALPHA;

    loop {
        let items_in_page = items_on_page(*current_page, items_number);
        let keyboard_scan = keyboard::scan();

        if keyboard_scan.key_down(key::BACK) {
            break;
        } else if (keyboard_scan.key_down(key::UP) || keyboard_scan.key_down(key::DOWN))
            && (timing::millis() >= (last_action + REPETITION_SPEED as u64))
        {
            display::wait_for_vblank();

            // moving cursor
            if keyboard_scan.key_down(key::UP) {
                if *cursor_pos > 0 {
                    // Un-highlight previous item
                    let old_global_idx =
                        (*current_page * MAX_ITEMS_PER_PAGE + *cursor_pos) as usize;
                    draw_setting_selection(
                        &visible[old_global_idx],
                        item_y(*first_y, *cursor_pos),
                        false,
                        cfg,
                    );

                    *cursor_pos -= 1;

                    // Highlight newly selected item
                    let new_global_idx =
                        (*current_page * MAX_ITEMS_PER_PAGE + *cursor_pos) as usize;
                    draw_setting_selection(
                        &visible[new_global_idx],
                        item_y(*first_y, *cursor_pos),
                        true,
                        cfg,
                    );
                } else if *current_page > 0 {
                    *current_page -= 1;
                    *cursor_pos = items_on_page(*current_page, items_number) - 1;
                    *first_y = draw_settings_page(visible, *current_page, pages, *cursor_pos, cfg);
                }
                // If on page 0 and cursor_pos == 0, do nothing (no wrap around)
                last_action_key = key::UP;
            } else if keyboard_scan.key_down(key::DOWN) {
                if *cursor_pos < items_in_page - 1 {
                    // Un-highlight previous item
                    let old_global_idx =
                        (*current_page * MAX_ITEMS_PER_PAGE + *cursor_pos) as usize;
                    draw_setting_selection(
                        &visible[old_global_idx],
                        item_y(*first_y, *cursor_pos),
                        false,
                        cfg,
                    );

                    *cursor_pos += 1;

                    // Highlight newly selected item
                    let new_global_idx =
                        (*current_page * MAX_ITEMS_PER_PAGE + *cursor_pos) as usize;
                    draw_setting_selection(
                        &visible[new_global_idx],
                        item_y(*first_y, *cursor_pos),
                        true,
                        cfg,
                    );
                } else if *current_page + 1 < pages {
                    *current_page += 1;
                    *cursor_pos = 0;
                    *first_y = draw_settings_page(visible, *current_page, pages, *cursor_pos, cfg);
                }
                // If on the last page and on the last item, do nothing (no wrap around)
                last_action_key = key::DOWN;
            }

            last_action = timing::millis();
        } else if (keyboard_scan.key_down(key::OK)
            || keyboard_scan.key_down(key::RIGHT)
            || keyboard_scan.key_down(key::LEFT))
            && (timing::millis() >= (last_action + REPETITION_SPEED as u64))
        {
            let current_y = item_y(*first_y, *cursor_pos);
            display::wait_for_vblank();

            push_rect_uniform(
                // remove last text
                Rect {
                    x: XPOS_VALUES,
                    y: current_y,
                    width: SCREEN_WIDTH - XPOS_VALUES,
                    height: LARGE_CHAR_HEIGHT + 2, // I got some problems with characters going under the line (like g)
                },
                cfg.bckgrd,
            );

            let global_idx = (*current_page * MAX_ITEMS_PER_PAGE + *cursor_pos) as usize;
            let selection: &mut Setting = &mut visible[global_idx];

            if !selection.fixed_values {
                if keyboard_scan.key_down(key::OK) {
                    let v = numberinput(
                        selection.values[0],
                        selection.values[1],
                        selection.values[2],
                        true,
                        Point {
                            x: XPOS_VALUES,
                            y: current_y,
                        },
                        cfg,
                    );
                    selection.set_value(v);
                    last_action_key = key::OK;
                } else if keyboard_scan.key_down(key::RIGHT) {
                    // increment one step
                    if selection.values[0] < selection.values[2] {
                        selection.set_value(selection.values[0] + 1);
                    }
                    last_action_key = key::RIGHT;
                } else if keyboard_scan.key_down(key::LEFT) {
                    if selection.values[0] > selection.values[1] {
                        selection.set_value(selection.values[0] - 1);
                    }
                    last_action_key = key::LEFT;
                }
            } else if keyboard_scan.key_down(key::OK) {
                selection.increment_value();
                last_action_key = key::OK;
            } else if keyboard_scan.key_down(key::RIGHT) {
                selection.increment_value();
                last_action_key = key::RIGHT;
            } else {
                selection.decrement_value();
                last_action_key = key::LEFT;
            }

            draw_setting_selection(selection, current_y, true, cfg);
            last_action = timing::millis();
        } else if !keyboard_scan.key_down(last_action_key) {
            // if we let go of the key, then we can use a key just after (even the same one)
            last_action = 0;
        }
    }
}

/// Draws the line corresponding to the given setting value
fn draw_setting_selection(set: &Setting, ypos: u16, selected: bool, cfg: &ColorConfig) {
    let text = if set.fixed_values {
        set.get_text()
    } else {
        &string_from_u32(set.get_setting_value())
    };
    let large: bool = get_string_pixel_size(text, true) < SCREEN_WIDTH - XPOS_VALUES;
    draw_string_cfg(
        text,
        Point::new(
            XPOS_VALUES,
            ypos + if !large {
                (LARGE_CHAR_HEIGHT - SMALL_CHAR_HEIGHT) / 2
            } else {
                0
            },
        ),
        large,
        cfg,
        selected,
    );
    push_rect_uniform(
        Rect {
            x: XPOS_VALUES - 15,
            y: ypos + LARGE_CHAR_HEIGHT / 2,
            width: 10,
            height: 2,
        },
        if selected { cfg.alt } else { cfg.bckgrd },
    );
}
