use crate::{eadk, utils::{fill_screen, draw_centered_string}, menu::pause_menu};
use eadk::{Color};

pub fn game(){

    fill_screen(Color::BLACK);
    let action  = pause_menu(Color::RED, Color::WHITE, Color::BLUE);
    if action == 0 {
        fill_screen(Color::WHITE);
        return
    } else if action == 1{
        fill_screen(Color::BLACK);
        draw_centered_string("on continue !\0", 50, true, Color::WHITE, Color::BLACK);
    }
    loop{

    }

}