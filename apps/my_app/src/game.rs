use crate::{eadk, utils::fill_screen, menu::pause_menu};
use eadk::{Color};

pub fn game(){

    fill_screen(Color::BLACK);
    if pause_menu(Color::RED, Color::WHITE, Color::BLUE) == 0{
        fill_screen(Color::WHITE);
        return
    };
    loop{

    }

}