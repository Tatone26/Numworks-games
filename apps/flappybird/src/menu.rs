use heapless::Vec;
use numworks_utils::menu::{
    settings::{write_values_to_file, Setting},
    start_menu,
};

use crate::{
    events::EventConfig,
    flappy_ui::menu_vis_addon,
    game::{game, COLOR_CONFIG},
    pipes::OscillationMode,
};

pub fn start() {
    let mut opt: [&mut Setting; 15] = [
        // 0: Speed
        &mut Setting {
            name: "Starting speed\0",
            choice: 2,
            values: Vec::from_slice(&[
                0.60_f32.to_bits(),
                0.85_f32.to_bits(),
                1.15_f32.to_bits(),
                1.45_f32.to_bits(),
                1.85_f32.to_bits(),
                2.40_f32.to_bits(),
            ])
            .unwrap(),
            texts: Vec::from_slice(&[
                "Very Slow\0",
                "Slow\0",
                "Normal\0",
                "Fast\0",
                "Insane\0",
                "Impossible\0",
            ])
            .unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 1: Pipes Density
        &mut Setting {
            name: "Pipes density\0",
            choice: 1,
            values: Vec::from_slice(&[1, 2, 3, 4]).unwrap(),
            texts: Vec::from_slice(&["Sparse\0", "Normal\0", "Dense\0", "Extreme\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 2: Gap Difficulty (Delta Y step scale)
        &mut Setting {
            name: "Gap difficulty\0",
            choice: 1,
            values: Vec::from_slice(&[0, 1, 2]).unwrap(),
            texts: Vec::from_slice(&["Easy\0", "Normal\0", "Hard\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 3: Moving Pipes Baseline Mode
        &mut Setting {
            name: "Moving pipes\0",
            choice: 6,
            values: Vec::from_slice(&[0, 1, 2, 3, 4, 5, 6]).unwrap(),
            texts: Vec::from_slice(&[
                "Off\0",
                "Rarely\0",
                "Sometimes\0",
                "Often\0",
                "Always Slow\0",
                "Always Fast\0",
                "Events only\0",
            ])
            .unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 4: Random Events Frequency
        &mut Setting {
            name: "Events frequency\0",
            choice: 2,
            values: Vec::from_slice(&[0, 1, 2, 3]).unwrap(),
            texts: Vec::from_slice(&["Never\0", "Rare\0", "Normal\0", "Frequent\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 5: Speed Progression
        &mut Setting {
            name: "Speed increase\0",
            choice: 2,
            values: Vec::from_slice(&[1000, 15, 10, 5, 1]).unwrap(),
            texts: Vec::from_slice(&[
                "Never\0",
                "Every 15 pts\0",
                "Every 10 pts\0",
                "Every 5 pts\0",
                "Every point\0",
            ])
            .unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 6: Jump Strength (Corrected naming: higher impulse = floatier pop)
        &mut Setting {
            name: "Jump strength\0",
            choice: 2,
            values: Vec::from_slice(&[
                5.4_f32.to_bits(),
                5.0_f32.to_bits(),
                4.6_f32.to_bits(),
                4.0_f32.to_bits(),
                3.5_f32.to_bits(),
            ])
            .unwrap(),
            texts: Vec::from_slice(&["Floaty\0", "Bouncy\0", "Normal\0", "Snappy\0", "Heavy\0"])
                .unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 7: Event Option - Pipe Surge
        &mut Setting {
            name: "Ev: Pipe surge\0",
            choice: 1,
            values: Vec::from_slice(&[0, 1]).unwrap(),
            texts: Vec::from_slice(&["No\0", "Yes\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 8: Event Option - Wind Gusts
        &mut Setting {
            name: "Ev: Wind gusts\0",
            choice: 1,
            values: Vec::from_slice(&[0, 1]).unwrap(),
            texts: Vec::from_slice(&["No\0", "Yes\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 9: Event Option - Narrow Gaps
        &mut Setting {
            name: "Ev: Narrow gaps\0",
            choice: 1,
            values: Vec::from_slice(&[0, 1]).unwrap(),
            texts: Vec::from_slice(&["No\0", "Yes\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 10: Event Option - Wide Gaps
        &mut Setting {
            name: "Ev: Wide gaps\0",
            choice: 1,
            values: Vec::from_slice(&[0, 1]).unwrap(),
            texts: Vec::from_slice(&["No\0", "Yes\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 11: Event Option - Dense Pipes Surge
        &mut Setting {
            name: "Ev: Dense pipes\0",
            choice: 1,
            values: Vec::from_slice(&[0, 1]).unwrap(),
            texts: Vec::from_slice(&["No\0", "Yes\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 12: Event Option - Gravity Changes
        &mut Setting {
            name: "Ev: Grav changes\0",
            choice: 1,
            values: Vec::from_slice(&[0, 1]).unwrap(),
            texts: Vec::from_slice(&["No\0", "Yes\0"]).unwrap(),
            user_modifiable: true,
            fixed_values: true,
        },
        // 13: Invincibility Cheat
        &mut Setting {
            name: "No collisions\0",
            choice: 0,
            values: Vec::from_slice(&[0, 1]).unwrap(),
            texts: Vec::from_slice(&["No\0", "Yes (CHEAT)\0"]).unwrap(),
            user_modifiable: false,
            fixed_values: true,
        },
        // 14: High Score
        &mut Setting {
            name: "High Score\0",
            choice: 0,
            values: Vec::from_slice(&[0, 0, u16::MAX as u32]).unwrap(),
            texts: Vec::new(),
            user_modifiable: false,
            fixed_values: false,
        },
    ];

    loop {
        let start = start_menu(
            "FLAPPY\0",
            &mut opt,
            &COLOR_CONFIG,
            menu_vis_addon,
            include_str!("./data/model_controls.txt"),
            "flappybird",
        );
        if start == 0 {
            loop {
                let mut high_score = opt[14].get_setting_value();
                let event_config = EventConfig {
                    frequency: opt[4].get_setting_value() as u8,
                    enable_moving_surge: opt[7].get_setting_value() != 0,
                    enable_wind: opt[8].get_setting_value() != 0,
                    enable_narrow_gaps: opt[9].get_setting_value() != 0,
                    enable_wide_gaps: opt[10].get_setting_value() != 0,
                    enable_dense_pipes: opt[11].get_setting_value() != 0,
                    enable_low_gravity: opt[12].get_setting_value() != 0,
                    enable_high_gravity: opt[12].get_setting_value() != 0,
                };

                let action = game(
                    f32::from_bits(opt[0].get_setting_value()),
                    opt[1].get_setting_value() as u16,
                    opt[2].get_setting_value() as u8,
                    OscillationMode::from_u32(opt[3].get_setting_value()),
                    event_config,
                    opt[5].get_setting_value() as u16,
                    f32::from_bits(opt[6].get_setting_value()),
                    opt[13].get_setting_value() != 0,
                    &mut high_score,
                );

                opt[14].set_value(high_score);
                write_values_to_file(&mut opt, "flappybird");

                if action == 2 {
                    return;
                } else if action == 1 {
                    break;
                }
            }
        } else {
            return;
        }
    }
}
