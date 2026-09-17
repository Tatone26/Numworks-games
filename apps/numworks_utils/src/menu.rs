pub mod settings;

use settings::{set_values_from_file, Setting};

use crate::eadk::display::{push_rect_uniform, wait_for_vblank, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::eadk::{display, key, keyboard, timing, Point, Rect};
use crate::graphical::{draw_centered_string, draw_string_cfg, fading, fill_screen, ColorConfig};
use crate::storage::open_file;
use crate::storage::save::{
    delete_save, load_save, write_save, GameSave, SaveField, SaveSlotMode, MAX_SAVE_FIELDS,
};
use crate::storage::MAX_STORAGE_VALUES;
use crate::utils::{
    get_centered_text_x_coordo, get_string_pixel_size, wait_for_no_keydown, CENTER,
    LARGE_CHAR_HEIGHT, SMALL_CHAR_HEIGHT,
};

const FADING_TIME: u32 = 500;
const REPETITION_SPEED: u16 = 200;
const SPACE_BETWEEN_LINES: u16 = LARGE_CHAR_HEIGHT;

pub struct MenuConfig {
    pub choices: &'static [&'static str],
    pub rect_margins: (u16, u16),
    pub dimensions: (u16, u16),
    pub offset: (i16, i16),
    pub back_key_return: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartMenuAction<T> {
    /// Slot index initialized (None for Single mode, Some(slot) for Multi mode).
    NewGame(Option<u8>),
    /// Active save state along with the slot it originated from.
    Continue(T, Option<u8>),
    Exit,
}

const CHOICES_NO_SAVE: &[&str] = &["Play\0", "Settings\0", "How to Play\0", "Exit\0"];
const CHOICES_WITH_SAVE: &[&str] = &[
    "Continue\0",
    "New Game\0",
    "Settings\0",
    "How to Play\0",
    "Exit\0",
];

fn confirm_dialog(prompt: &str, cfg: &ColorConfig) -> bool {
    let dialog_w = 200u16;
    let dialog_h = (LARGE_CHAR_HEIGHT * 3) + 16;
    let dialog_x = (SCREEN_WIDTH - dialog_w) / 2;
    let dialog_y = (SCREEN_HEIGHT - dialog_h) / 2;

    display::wait_for_vblank();

    // Outer border
    push_rect_uniform(
        Rect {
            x: dialog_x,
            y: dialog_y,
            width: dialog_w,
            height: dialog_h,
        },
        cfg.alt,
    );
    // Inner background
    push_rect_uniform(
        Rect {
            x: dialog_x + 2,
            y: dialog_y + 2,
            width: dialog_w - 4,
            height: dialog_h - 4,
        },
        cfg.bckgrd,
    );

    // Prompt text centered near the top of the box
    draw_centered_string(prompt, dialog_y + 10, true, cfg, false);

    // Button row Y offset relative to SCREEN_HEIGHT / 2
    let btn_y_offset =
        (dialog_y + dialog_h - LARGE_CHAR_HEIGHT - 12) as i16 - (SCREEN_HEIGHT / 2) as i16;

    let choice = selection(
        cfg,
        &MenuConfig {
            choices: &["No\0", "Yes\0"],
            rect_margins: (35, 0), // 35px padding on left and right inside the 200px box
            dimensions: (dialog_w - 4, LARGE_CHAR_HEIGHT + 4),
            offset: (0, btn_y_offset),
            back_key_return: 0,
        },
        true,
    );

    choice == 1
}

fn open_settings_with_save_editor<T: GameSave>(
    opt: &mut [&mut Setting],
    cfg: &ColorConfig,
    filename: &str,
    godmode: bool,
) {
    if !godmode {
        settings::settings(opt, cfg, filename, false);
        return;
    }

    if let Some(mut save_state) = load_save::<T>(filename, None) {
        let fields = save_state.describe_fields();
        let mut save_settings: heapless::Vec<Setting, MAX_SAVE_FIELDS> = heapless::Vec::new();

        for f in fields.iter() {
            let mut vals: heapless::Vec<u32, MAX_STORAGE_VALUES> = heapless::Vec::new();
            let _ = vals.push(f.value);
            let _ = vals.push(f.min);
            let _ = vals.push(f.max);

            let _ = save_settings.push(Setting {
                name: f.name,
                choice: 0,
                values: vals,
                texts: heapless::Vec::new(),
                user_modifiable: false,
                fixed_values: false,
            });
        }

        let opt_len = opt.len();
        let mut updated_values: heapless::Vec<u32, MAX_SAVE_FIELDS> = heapless::Vec::new();
        {
            let mut combined: heapless::Vec<&mut Setting, MAX_STORAGE_VALUES> =
                heapless::Vec::new();
            for s in opt.iter_mut() {
                let _ = combined.push(&mut **s);
            }
            for s in save_settings.iter_mut() {
                let _ = combined.push(s);
            }

            settings::settings(&mut combined, cfg, filename, true);

            for s in combined[opt_len..].iter() {
                let _ = updated_values.push(s.get_setting_value());
            }
        }

        let mut updated_fields: heapless::Vec<SaveField, MAX_SAVE_FIELDS> = heapless::Vec::new();
        for (i, f) in fields.iter().enumerate() {
            let _ = updated_fields.push(SaveField {
                name: f.name,
                min: f.min,
                max: f.max,
                value: updated_values[i],
            });
        }

        save_state.update_from_fields(&updated_fields);
        write_save(filename, None, &save_state);
    } else {
        settings::settings(opt, cfg, filename, true);
    }
}

fn start_menu_single_slot<T: GameSave>(
    title: &str,
    opt: &mut [&mut Setting],
    cfg: &ColorConfig,
    vis_addon: fn(),
    controls_text: &str,
    filename: &str,
    check_save: bool,
) -> StartMenuAction<T> {
    open_file(filename);
    set_values_from_file(opt, filename);

    loop {
        let maybe_save: Option<T> = if check_save {
            load_save(filename, None)
        } else {
            None
        };

        wait_for_vblank();
        fill_screen(cfg.bckgrd);
        vis_addon();
        draw_centered_string(title, 20, true, cfg, false);

        let (choices, back_key, dims) = if maybe_save.is_some() {
            (
                CHOICES_WITH_SAVE,
                4u8,
                (SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2 + 16),
            )
        } else {
            (CHOICES_NO_SAVE, 3u8, (SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2))
        };

        let action = selection(
            cfg,
            &MenuConfig {
                choices,
                rect_margins: (10, 10),
                dimensions: dims,
                offset: (0, (SCREEN_HEIGHT / 5) as i16),
                back_key_return: back_key,
            },
            false,
        );

        if let Some(save) = maybe_save {
            match action {
                0 => {
                    fading(FADING_TIME);
                    return StartMenuAction::Continue(save, None);
                }
                1 => {
                    if confirm_dialog("Overwrite save?\0", cfg) {
                        delete_save(filename, None);
                        fading(FADING_TIME);
                        return StartMenuAction::NewGame(None);
                    }
                }
                2 => {
                    let scan = keyboard::scan();
                    open_settings_with_save_editor::<T>(
                        opt,
                        cfg,
                        filename,
                        scan.key_down(key::SHIFT),
                    );
                }
                3 => {
                    controls(controls_text, cfg);
                }
                _ => {
                    fading(FADING_TIME);
                    return StartMenuAction::Exit;
                }
            }
        } else {
            match action {
                0 => {
                    fading(FADING_TIME);
                    return StartMenuAction::NewGame(None);
                }
                1 => {
                    let scan = keyboard::scan();
                    open_settings_with_save_editor::<T>(
                        opt,
                        cfg,
                        filename,
                        scan.key_down(key::SHIFT),
                    );
                }
                2 => {
                    controls(controls_text, cfg);
                }
                _ => {
                    fading(FADING_TIME);
                    return StartMenuAction::Exit;
                }
            }
        }
    }
}

/// 100% backward-compatible start_menu for legacy games without saves.
/// Returns 0 for Play, 3 for Exit.
pub fn start_menu(
    title: &str,
    opt: &mut [&mut Setting],
    cfg: &ColorConfig,
    vis_addon: fn(),
    controls_text: &str,
    filename: &str,
) -> u8 {
    #[derive(Clone, Copy)]
    struct NoSave;
    impl GameSave for NoSave {
        fn to_words(&self) -> heapless::Vec<u32, MAX_STORAGE_VALUES> {
            heapless::Vec::new()
        }
        fn from_words(_: &[u32]) -> Option<Self> {
            None
        }
        fn describe_fields(&self) -> heapless::Vec<SaveField, MAX_SAVE_FIELDS> {
            heapless::Vec::new()
        }
        fn update_from_fields(&mut self, _: &[SaveField]) {}
    }

    match start_menu_single_slot::<NoSave>(
        title,
        opt,
        cfg,
        vis_addon,
        controls_text,
        filename,
        false,
    ) {
        StartMenuAction::NewGame(_) => 0,
        _ => 3,
    }
}

/// Enhanced start menu with dynamic save detection and slot mode routing.
pub fn start_menu_with_save<T: GameSave>(
    title: &str,
    opt: &mut [&mut Setting],
    cfg: &ColorConfig,
    vis_addon: fn(),
    controls_text: &str,
    filename: &str,
    slot_mode: SaveSlotMode,
) -> StartMenuAction<T> {
    match slot_mode {
        SaveSlotMode::Single => {
            start_menu_single_slot(title, opt, cfg, vis_addon, controls_text, filename, true)
        }
        SaveSlotMode::Multiple(max_slots) => {
            todo!(
                "Multiple save slots picker not implemented yet! Requested slots: {}",
                max_slots
            );
        }
    }
}

pub fn pause_menu(cfg: &ColorConfig, y_offset: i16) -> u8 {
    selection(
        cfg,
        &MenuConfig {
            choices: &["Resume\0", "Menu\0", "Exit\0"],
            rect_margins: (20, 10),
            dimensions: (
                SCREEN_WIDTH * 2 / 5,
                LARGE_CHAR_HEIGHT * 3 + SPACE_BETWEEN_LINES * 4,
            ),
            offset: (0, y_offset),
            back_key_return: 0,
        },
        false,
    )
}

pub fn selection(color: &ColorConfig, config: &MenuConfig, horizontal: bool) -> u8 {
    display::wait_for_vblank();
    display::push_rect_uniform(
        Rect {
            x: (((SCREEN_WIDTH - config.dimensions.0) / 2) as i16 + config.offset.0) as u16,
            y: (((SCREEN_HEIGHT - config.dimensions.1) / 2) as i16 + config.offset.1) as u16,
            width: config.dimensions.0,
            height: config.dimensions.1,
        },
        color.bckgrd,
    );
    for (i, _) in config.choices.iter().enumerate() {
        draw_selection_string(i as u8, color, config, i == 0, horizontal);
    }
    wait_for_no_keydown();

    let mut cursor_pos: u8 = 0;
    let mut last_action: u64 = timing::millis();
    let mut last_action_key: u32 = key::ALPHA;
    loop {
        let keyboard_state = keyboard::scan();
        if (keyboard_state.key_down(key::DOWN)
            || keyboard_state.key_down(key::UP)
            || keyboard_state.key_down(key::LEFT)
            || keyboard_state.key_down(key::RIGHT))
            && (timing::millis() >= (last_action + REPETITION_SPEED as u64))
        {
            display::wait_for_vblank();
            draw_selection_string(cursor_pos, color, config, false, horizontal);
            if (!horizontal && keyboard_state.key_down(key::DOWN))
                || (horizontal && keyboard_state.key_down(key::RIGHT))
            {
                cursor_pos += 1;
                if cursor_pos >= config.choices.len() as u8 {
                    cursor_pos = 0;
                }
                last_action_key = if keyboard_state.key_down(key::DOWN) {
                    key::DOWN
                } else {
                    key::RIGHT
                };
            } else if (!horizontal && keyboard_state.key_down(key::UP))
                || (horizontal && keyboard_state.key_down(key::LEFT))
            {
                if cursor_pos == 0 {
                    cursor_pos = config.choices.len() as u8 - 1;
                } else {
                    cursor_pos -= 1;
                }
                last_action_key = if keyboard_state.key_down(key::UP) {
                    key::UP
                } else {
                    key::LEFT
                };
            }
            draw_selection_string(cursor_pos, color, config, true, horizontal);
            last_action = timing::millis();
        } else if keyboard_state.key_down(key::OK) {
            loop {
                let keyboard_state_test = keyboard::scan();
                if !keyboard_state_test.key_down(key::OK) {
                    return cursor_pos;
                }
            }
        } else if keyboard_state.key_down(key::BACK) {
            return config.back_key_return;
        } else if !keyboard_state.key_down(last_action_key) {
            last_action = 0;
        }
    }
}

fn draw_selection_string(
    cursor_pos: u8,
    color: &ColorConfig,
    config: &MenuConfig,
    selected: bool,
    horizontal: bool,
) {
    let text: &str = config.choices[cursor_pos as usize];

    let y_pos: u16 = if !horizontal {
        (CENTER.y - config.dimensions.1 / 2
            + config.rect_margins.1
            + cursor_pos as u16
                * (config.dimensions.1 - config.rect_margins.1 * 2 - LARGE_CHAR_HEIGHT)
                / (config.choices.len() as u16 - 1)) as i16
            + config.offset.1
    } else {
        (CENTER.y - LARGE_CHAR_HEIGHT / 2) as i16 + config.offset.1
    } as u16;

    let x_coordos: u16 = if horizontal {
        let box_left = (((SCREEN_WIDTH - config.dimensions.0) / 2) as i16 + config.offset.0) as u16;
        let box_right = box_left + config.dimensions.0;

        if cursor_pos == 0 {
            if config.rect_margins.0 != 0 {
                box_left + config.rect_margins.0
            } else {
                box_left + 10
            }
        } else if cursor_pos == config.choices.len() as u8 - 1 {
            box_right
                - get_string_pixel_size(config.choices[cursor_pos as usize], true)
                - config.rect_margins.0
        } else {
            // Original balanced calculation bounded strictly to the menu box
            let start_x = box_left + get_string_pixel_size(config.choices[0], true);
            let max_x = box_right
                - get_string_pixel_size(config.choices.last().unwrap(), true)
                - config.rect_margins.0;
            start_x + cursor_pos as u16 * (max_x - start_x) / (config.choices.len() as u16 - 1)
                - get_string_pixel_size(config.choices[cursor_pos as usize], true) / 2
        }
    } else {
        (get_centered_text_x_coordo(text, true) as i16 + config.offset.0) as u16
    };

    draw_string_cfg(text, Point::new(x_coordos, y_pos), true, color, selected);
    push_rect_uniform(
        Rect {
            x: if x_coordos > 15 { x_coordos - 15 } else { 0 },
            y: y_pos + LARGE_CHAR_HEIGHT / 2,
            width: if x_coordos > 15 { 10 } else { x_coordos - 2 },
            height: 2,
        },
        if selected { color.alt } else { color.bckgrd },
    );
}

fn controls(text: &str, cfg: &ColorConfig) -> u8 {
    wait_for_vblank();
    fill_screen(cfg.bckgrd);
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
    wait_for_vblank();
    for (i, line) in text.split('\n').enumerate() {
        draw_string_cfg(
            line,
            Point::new(0, i as u16 * SMALL_CHAR_HEIGHT),
            false,
            cfg,
            false,
        );
    }
    loop {
        let keyboard_state = keyboard::scan();
        if keyboard_state.key_down(key::BACK) {
            break;
        }
    }
    0
}
