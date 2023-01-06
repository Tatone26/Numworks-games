use crate::{menu::{MyOption, menu, pause_menu}, utils::{ColorConfig, draw_centered_string, fill_screen, draw_image}, eadk::{Color, display::{push_rect_uniform, push_rect}, Rect, timing}};

/// The number of Boolean Options used. Public so menu() can use it.
pub const BOOL_OPTIONS_NUMBER: usize = 1;

// This dictates the principal colors that will be used
const COLOR_CONFIG: ColorConfig = ColorConfig {
    text: Color::BLACK,
    bckgrd: Color::WHITE,
    alt: Color::RED,
};

static mut EXEMPLE: bool = false;

fn vis_addon(){
    push_rect_uniform(Rect{x:0, y:0, width:10, height:10}, Color::BLACK);
}
/// Menu, Options and Game start
pub fn start() {
    let mut opt: [&mut MyOption<bool, 2>; BOOL_OPTIONS_NUMBER] = [
        &mut MyOption {
            name: "Option !\0",
            value: 0,
            possible_values: [(true, "True\0"), (false, "False\0")],
        }
    ];
    loop {
        let start = menu("SNAKE 2.0\0", &mut opt, &COLOR_CONFIG, vis_addon); // The menu does everything itself !
        if start == 1 {
            unsafe{
                EXEMPLE = opt[0].get_value().0; // You could use mutable statics, but it is not very good
            }
            loop { // a loop where the game is played again and again, which means it should be 100% contained after the menu
                let action = game(opt[0].get_value().0); // calling the game based on the parameters is better
                if action == 0 { // 0 means quitting
                    return; 
                } else if action == 2 { // 2 means back to menu
                    break;
                } // if action == 1 : rejouer
            }
        } else {
            return;
        }
    }
}

/// The entire game is here.
pub fn game(_exemple: bool) -> u8 {
    let test_image = *include_bytes!("./icon_snake.ppm");
    draw_image(test_image, 100, 100);
    loop {
        
    };
    return 1
}

