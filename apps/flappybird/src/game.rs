use heapless::Vec;

use numworks_utils::{
    eadk::{
        display::{self, SCREEN_WIDTH},
        key, keyboard, Color, Point,
    },
    graphical::{draw_centered_string, fading, fill_screen, ColorConfig},
    menu::{
        selection,
        settings::{Setting, SettingType},
        start_menu, MenuConfig,
    },
    utils::LARGE_CHAR_HEIGHT,
};

use crate::{
    bird::Player,
    flappy_ui::{
        draw_bird, draw_constant_ui, draw_top_pipe, draw_top_pipes_pipes, draw_ui, move_cloud,
        BACKGROUND, PIXELS, TILESET, TILESET_TILE_SIZE,
    },
    pipes::Pipes,
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
                unsafe { v.push_unchecked((SettingType::Int(1), "Slow\0")) };
                unsafe { v.push_unchecked((SettingType::Int(2), "Normal\0")) };
                unsafe { v.push_unchecked((SettingType::Int(4), "Fast\0")) };
                v
            },
        },
        &mut Setting {
            name: "Pipes density\0",
            value: 2,
            possible_values: {
                let mut v = Vec::new();
                unsafe { v.push_unchecked((SettingType::Int(1), "Easy\0")) };
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
                unsafe { v.push_unchecked((SettingType::Double(6.0), "Weak\0")) };
                unsafe { v.push_unchecked((SettingType::Double(6.8), "Normal\0")) };
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
                    opt[0].get_param_value(),
                    opt[1].get_param_value(),
                    opt[2].get_param_value(),
                    opt[3].get_param_value(),
                    opt[4].get_param_value(),
                    opt[5].get_param_value(),
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

const MAX_PIPES_ON_SCREEN: usize = 3;

/// The entire game is here.
pub fn game(
    starting_speed: u16,
    nb_pipes_u: u16,
    speed_increase: u16,
    killer_floor: bool,
    jump_power: f32,
    no_collisions: bool,
) -> u8 {
    // Optimisation : storing the pipes in RAM !
    let left_pipe_tile: [Color; PIXELS] = TILESET.get_tile::<PIXELS>(Point { x: 1, y: 0 });
    let right_pipe_tile: [Color; PIXELS] = TILESET.get_tile::<PIXELS>(Point { x: 2, y: 0 });
    let left_cloud_tile: [Color; PIXELS] = TILESET.get_tile::<PIXELS>(Point { x: 1, y: 3 });
    let right_cloud_tile: [Color; PIXELS] = TILESET.get_tile::<PIXELS>(Point { x: 2, y: 3 });

    fill_screen(BACKGROUND);
    draw_constant_ui();

    let nb_pipes = nb_pipes_u as usize;
    let mut pipes_list: Vec<Pipes, MAX_PIPES_ON_SCREEN> = Vec::new();
    for _ in 0..nb_pipes {
        let _ = pipes_list.push(Pipes::new(starting_speed, 75));
    }
    pipes_list[0].draw_self(&left_pipe_tile, &right_pipe_tile);

    let mut score: u16 = 0;
    let mut bird = Player::new(jump_power);
    bird.draw_self();

    let mut can_increase_speed: bool = true; // if true, can check for speed increase (true as soon as a point is won)

    let mut active_pipe: Vec<bool, MAX_PIPES_ON_SCREEN> = Vec::new();
    let _ = active_pipe.push(true);
    for _ in 1..nb_pipes {
        let _ = active_pipe.push(false);
    }
    let mut counter: usize = 0;
    let mut start: bool = false;

    // The cloud serve a parallax purpose. I spend FAR TOO MUCH TIME on that but it's so nice to have
    // one downside is that it might slow things down...
    let mut cloud_pos: Point = Point {
        x: SCREEN_WIDTH - WINDOW_SIZE - TILESET_TILE_SIZE * 2,
        y: WINDOW_SIZE + 5,
    };

    loop {
        display::wait_for_vblank();

        let pipes_turn = counter % nb_pipes;
        let scan = keyboard::scan();
        let death = if start || counter > 20 || scan.key_down(key::OK) {
            start = true;
            bird.action_function(scan, killer_floor && (!no_collisions)) // if true : bird touched floor and killer_floor
        } else {
            false
        };

        // pipes turn, activation etc
        if !death && active_pipe[pipes_turn] {
            // if pipe active, do that
            let add = pipes_list[pipes_turn].action_function(&left_pipe_tile, &right_pipe_tile);
            if add != 0 {
                active_pipe[pipes_turn] = false;
                can_increase_speed = true;
            }
            score += add;

            let current = pipes_turn;
            counter = counter.wrapping_add(1);
            if nb_pipes > 1
                && !active_pipe[counter % nb_pipes]
                && pipes_list[current].x_pos
                    < SCREEN_WIDTH - SCREEN_WIDTH / nb_pipes as u16 - TILESET_TILE_SIZE * nb_pipes_u
            {
                active_pipe[counter % nb_pipes] = true;
            } else if nb_pipes == 1 {
                active_pipe[current] = true;
            }
        } else {
            // else skip pipe
            counter = counter.wrapping_add(1);
        }

        if counter % nb_pipes * 2 == 0 {
            // cloud
            move_cloud(
                &mut cloud_pos,
                counter as u16,
                &left_cloud_tile,
                &right_cloud_tile,
            );
            for i in 0..nb_pipes {
                if active_pipe[i]
                    && cloud_pos.x < pipes_list[i].x_pos + TILESET_TILE_SIZE * 2
                    && cloud_pos.x + TILESET_TILE_SIZE * 2 > pipes_list[i].x_pos
                {
                    if pipes_list[i].interval.0 > WINDOW_SIZE + TILESET_TILE_SIZE * 2 + 5 {
                        draw_top_pipes_pipes(
                            pipes_list[i].x_pos,
                            pipes_list[i].interval,
                            &left_pipe_tile,
                            &right_pipe_tile,
                        );
                    } else {
                        draw_top_pipe(
                            pipes_list[i].x_pos,
                            pipes_list[i].interval,
                            &left_pipe_tile,
                            &right_pipe_tile,
                        );
                    }
                    break;
                }
            }
        }
        draw_ui(score);
        // speed increase
        if can_increase_speed && score % speed_increase == 0 {
            for i in 0..nb_pipes {
                pipes_list[i].increase_speed();
            }
            can_increase_speed = false;
        }

        // collisions
        let mut collision = false;
        for i in 0..nb_pipes {
            if !no_collisions && bird_collide_with(&bird, &pipes_list[i]) {
                collision = true;
                break;
            }
        }
        if death || collision {
            // TODO : game over screen and menu
            draw_centered_string("GAME OVER\0", 70, true, &COLOR_CONFIG, true);
            let action = flappy_pause(death || collision);
            return action;
        }
    }
}

const NICE_COLLISION_MARGIN: u16 = 2;

#[inline]
fn bird_collide_with(bird: &Player, pipes: &Pipes) -> bool {
    if bird.x_pos + TILESET_TILE_SIZE < pipes.x_pos.saturating_sub(5)
        || bird.x_pos > pipes.x_pos + TILESET_TILE_SIZE * 2
    {
        false
    } else {
        bird.y_pos < pipes.interval.0.saturating_sub(NICE_COLLISION_MARGIN)
            || bird.y_pos + TILESET_TILE_SIZE > pipes.interval.1 + NICE_COLLISION_MARGIN
    }
}

fn flappy_pause(death: bool) -> u8 {
    let action = selection(
        &COLOR_CONFIG,
        &MenuConfig {
            choices: if death {
                &["Play again\0", "Menu\0", "Exit\0"]
            } else {
                &["Resume\0", "Menu\0", "Exit\0"]
            },
            rect_margins: (20, 12),
            dimensions: (SCREEN_WIDTH * 7 / 15, LARGE_CHAR_HEIGHT * 5),
            offset: (0, if death { 50 } else { 0 }),
            back_key_return: if death { 2 } else { 1 },
        },
        false,
    );
    if action != 0 {
        fading(500);
    }
    action
}
