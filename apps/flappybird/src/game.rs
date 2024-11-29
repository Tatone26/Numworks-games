use heapless::Vec;

use numworks_utils::{
    eadk::{
        display::{self, SCREEN_HEIGHT, SCREEN_WIDTH},
        key, keyboard, Color, Point,
    },
    graphical::{draw_centered_string, fill_screen, ColorConfig},
    menu::{
        selection,
        settings::{Setting, SettingType},
        start_menu, MenuConfig,
    },
    numbers::floor,
    utils::{CENTER, LARGE_CHAR_HEIGHT},
};

use crate::{
    bird::Player,
    flappy_ui::{
        countdown, draw_bird, draw_constant_ui, draw_dead_bird, draw_ground, draw_ui, Cloud,
        BACKGROUND, PIXELS, TILESET, TILESET_TILE_SIZE, UI_BACKGROUND,
    },
    pipes::{OptiTiles, Pipes},
};

// This dictates the principal colors that will be used for menu etc
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: BACKGROUND,
    alt: Color::from_rgb888(255, 140, 65),
};

fn vis_addon() {
    draw_bird(
        Point {
            x: SCREEN_WIDTH / 2 - TILESET_TILE_SIZE / 2,
            y: 70,
        },
        0,
    );
}
/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut Setting; 6] = [
        &mut Setting {
            name: "Starting speed\0",
            value: 1,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((SettingType::Double(0.5), "Slow\0")) };
                unsafe { v.push_unchecked((SettingType::Double(0.75), "Normal\0")) };
                unsafe { v.push_unchecked((SettingType::Double(1.0), "Fast\0")) };
                v
            },
        },
        &mut Setting {
            name: "Pipes density\0",
            value: 1,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((SettingType::Int(1), "Sparse\0")) };
                unsafe { v.push_unchecked((SettingType::Int(2), "Normal\0")) };
                unsafe { v.push_unchecked((SettingType::Int(3), "Dense\0")) };
                v
            },
        },
        &mut Setting {
            name: "Speed increase\0",
            value: 2,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((SettingType::Int(1000), "Never\0")) };
                unsafe { v.push_unchecked((SettingType::Int(10), "Every 10 pts\0")) };
                unsafe { v.push_unchecked((SettingType::Int(5), "Every 5 pts\0")) };
                unsafe { v.push_unchecked((SettingType::Int(1), "Every point\0")) };
                v
            },
        },
        &mut Setting {
            name: "Die on floor\0",
            value: 1,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((SettingType::Bool(false), "No\0")) };
                unsafe { v.push_unchecked((SettingType::Bool(true), "Yes\0")) };
                v
            },
        },
        &mut Setting {
            name: "Jump strength\0",
            value: 1,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((SettingType::Double(5.5), "Weak\0")) };
                unsafe { v.push_unchecked((SettingType::Double(6.5), "Normal\0")) };
                unsafe { v.push_unchecked((SettingType::Double(7.5), "Strong\0")) };
                v
            },
        },
        &mut Setting {
            name: "No collisions\0",
            value: 0,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((SettingType::Bool(false), "No\0")) };
                unsafe { v.push_unchecked((SettingType::Bool(true), "Yes (CHEAT)\0")) };
                v
            },
        },
    ];
    loop {
        let start = start_menu(
            "FLAPPY BIRD\0",
            &mut opt,
            &COLOR_CONFIG,
            vis_addon,
            include_str!("./data/model_controls.txt"),
        );
        // The menu does everything itself !
        if start == 0 {
            loop {
                // a loop where the game is played again and again, which means it should be 100% contained after the menu
                let action = game(
                    opt[0].get_setting_value(),
                    opt[1].get_setting_value(),
                    opt[2].get_setting_value(),
                    opt[3].get_setting_value(),
                    opt[4].get_setting_value(),
                    opt[5].get_setting_value(),
                ); // calling the game based on the parameters is better
                if action == 2 {
                    // 2 means quitting
                    return;
                } else if action == 1 {
                    // 1 means back to menu
                    break;
                } // if action == 0 : rejouer
            }
        } else {
            return;
        }
    }
}

/// number of pixels of the window border.
pub const WINDOW_SIZE: u16 = 20;

const MAX_PIPES_ON_SCREEN: usize = 4;

/// The entire game is here.
pub fn game(
    starting_speed: f32,
    density: u16,
    speed_increase: u16,
    killer_floor: bool,
    jump_power: f32,
    no_collisions: bool,
) -> u8 {
    let mut cloud = Cloud::new(
        Point {
            x: SCREEN_WIDTH - WINDOW_SIZE - TILESET_TILE_SIZE * 2,
            y: WINDOW_SIZE + 5,
        },
        0.20,
    );

    // Optimisation : storing the transparent images in RAM. Those are loaded anyway, so may has well do it only once.
    // The other tiles don't have to be loaded in RAM to be drawn by a single call.
    let tiles = OptiTiles {
        left_bottom_tile: TILESET.get_tile::<PIXELS>(Point { x: 0, y: 2 }),
        right_bottom_tile: TILESET.get_tile::<PIXELS>(Point { x: 3, y: 2 }),
        left_top_tile: TILESET.get_tile::<PIXELS>(Point { x: 0, y: 1 }),
        right_top_tile: TILESET.get_tile::<PIXELS>(Point { x: 3, y: 1 }),
    };

    let mut pipes_list: Vec<Pipes, MAX_PIPES_ON_SCREEN> = Vec::new();
    for _ in 0..MAX_PIPES_ON_SCREEN {
        let _ = pipes_list.push(Pipes::new(starting_speed, 75, &tiles));
    }
    pipes_list[0].active = true;
    pipes_list[0].has_moved = true;

    let mut bird = Player::new(jump_power);

    display::wait_for_vblank();
    fill_screen(BACKGROUND);
    cloud.draw_self();
    for p in pipes_list.iter() {
        p.draw_self(); // technically, only the first is necessary, and only the first will be called.
    }
    bird.draw_self();

    draw_ground(0);

    draw_constant_ui();
    draw_ui(0);

    let mut score: u16 = 0;
    let mut can_increase_speed: bool = true; // if true, can check for speed increase (true as soon as a point is won)

    let mut frame_counter: u16 = 0;
    let mut start: bool = false;

    let mut previous_pipe_active: bool = true; // used to know if a given pipe can start
    let mut previous_pipe_x_pos: u16 = 0; // Used to know if a given pipe can start
    let mut previous_pipe_decimal_offset: f32 = 0.0; // Used to align all pipes

    let mut right_most_pipe: usize = 0;

    countdown(Point {
        x: CENTER.x - TILESET_TILE_SIZE,
        y: CENTER.y - TILESET_TILE_SIZE * 2,
    });

    'gameloop: loop {
        // By optimising the s*** out of my graphical methods, I was able to draw all 3 double pipes, the cloud, the floor and the bird every SINGLE frame !!
        // I compute everything during the frame time, and I draw eveything during the vblank time (which is short so it's difficult)
        // Need to make sure the game logic is fast enough and that I draw NOTHING unnecessary !!
        let scan = keyboard::scan();

        // Pause
        if scan.key_down(key::BACK) {
            let action = flappy_pause(false);
            if action != 0 {
                return action;
            } else {
                display::wait_for_vblank();
                fill_screen(BACKGROUND);
                cloud.draw_self();
                for p in pipes_list.iter() {
                    p.draw_self();
                }
                bird.draw_self();

                draw_ground(frame_counter);

                draw_constant_ui();
                draw_ui(0);
                countdown(Point {
                    x: CENTER.x - TILESET_TILE_SIZE,
                    y: CENTER.y - TILESET_TILE_SIZE * 3,
                });
                continue;
            }
        }

        // Moving the bird (and checking the floor collision)
        if start || frame_counter > 20 || scan.key_down(key::OK) {
            start = true;
            if bird.action_function(scan, killer_floor && (!no_collisions)) {
                break;
            }
        }
        // Moving every pipe
        for pipe in pipes_list.iter_mut() {
            // if pipe active :
            if pipe.action() > 0 {
                can_increase_speed = true;
                score += 1;
            }
            // if pipe not active : try to activate it
            if !pipe.active
                && previous_pipe_active
                && previous_pipe_x_pos
                    < SCREEN_WIDTH
                        - SCREEN_WIDTH / (density + 1)
                        - TILESET_TILE_SIZE * (MAX_PIPES_ON_SCREEN as u16 + 1 - density)
                        - TILESET_TILE_SIZE / 2
            {
                pipe.active = true;
                pipe.move_pipe(previous_pipe_decimal_offset); // aligning all the pipes onto the same moving frame
            }
            // that's for the next pipe
            previous_pipe_active = pipe.active;
            previous_pipe_x_pos = pipe.x_pos;
            previous_pipe_decimal_offset = pipe.true_pos - floor(pipe.true_pos);
        }
        cloud.action();

        // collisions -> game over
        for pipe in pipes_list.iter().filter(|p| p.active) {
            if !no_collisions && bird_collide_with(&bird, pipe) {
                break 'gameloop;
            }
        }

        // speed increase
        if can_increase_speed && score != 0 && score % speed_increase == 0 {
            for i in 0..MAX_PIPES_ON_SCREEN {
                pipes_list[i].increase_speed();
            }
            can_increase_speed = false;
        }

        let next_index = (right_most_pipe + 1) % MAX_PIPES_ON_SCREEN;
        if pipes_list[next_index].active
            && pipes_list[next_index].x_pos > SCREEN_WIDTH - WINDOW_SIZE - TILESET_TILE_SIZE * 2
        {
            right_most_pipe = next_index;
        }

        // Drawing everything
        display::wait_for_vblank();
        cloud.clear_old_self();
        cloud.draw_self();

        // Drawing the pipes right to left
        let mut current_pipe = right_most_pipe;
        for _ in 0..MAX_PIPES_ON_SCREEN {
            let pipe = &pipes_list[current_pipe]; // error
            pipe.clear_old_self();
            pipe.draw_self();
            current_pipe = if current_pipe == 0 {
                MAX_PIPES_ON_SCREEN - 1
            } else {
                current_pipe - 1
            }
        }

        draw_ground(frame_counter); // this too

        draw_ui(score); // TODO : this is just too late in the drawing time. -> optimisation necessary
        {
            bird.clear_old_self();
            bird.draw_self();
        }

        frame_counter = frame_counter.wrapping_add(1);
    }
    // Game over
    bird.clear_old_self();
    draw_dead_bird(Point {
        x: bird.x_pos,
        y: bird.y_pos,
    });
    draw_centered_string("GAME OVER\0", 70, true, &COLOR_CONFIG, true);
    flappy_pause(true)
}

/// Numbers of pixels where there should be collisions but there aren't because the game would be a lot more difficult
const NICE_COLLISION_MARGIN: u16 = 2;

#[inline]
fn bird_collide_with(bird: &Player, pipes: &Pipes) -> bool {
    if bird.x_pos + TILESET_TILE_SIZE
        < pipes
            .x_pos
            .saturating_sub(5)
            .saturating_add(NICE_COLLISION_MARGIN)
        || bird.x_pos
            > pipes
                .x_pos
                .saturating_add(3)
                .saturating_sub(NICE_COLLISION_MARGIN)
                + TILESET_TILE_SIZE * 2
    {
        false
    } else {
        bird.y_pos < pipes.interval.0.saturating_sub(NICE_COLLISION_MARGIN)
            || bird.y_pos + TILESET_TILE_SIZE > pipes.interval.1 + NICE_COLLISION_MARGIN
    }
}

fn flappy_pause(death: bool) -> u8 {
    selection(
        &ColorConfig {
            text: Color::WHITE,
            bckgrd: UI_BACKGROUND,
            alt: COLOR_CONFIG.alt,
        },
        &MenuConfig {
            choices: if death {
                &["Play again\0", "Menu\0", "Exit\0"]
            } else {
                &["Resume\0", "Menu\0", "Exit\0"]
            },
            rect_margins: (20, 0),
            dimensions: (SCREEN_WIDTH, LARGE_CHAR_HEIGHT + LARGE_CHAR_HEIGHT / 2),
            offset: (
                0,
                SCREEN_HEIGHT as i16 / 2 - 2 * LARGE_CHAR_HEIGHT as i16 / 3,
            ),
            back_key_return: if death { 2 } else { 1 },
        },
        true,
    )
}
