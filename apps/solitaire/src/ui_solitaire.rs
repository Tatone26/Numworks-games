use numworks_utils::{utils::{Tileset, wait_for_no_keydown, fill_screen, draw_tile}, eadk::{Color, display, Point}};

/// Images work really well with square tiles. You can still draw other images, but it is less good.
static TILESET: Tileset = Tileset {
    tile_size: 35,
    image: include_bytes!("./data/cartes.ppm"),
};
const PIXELS: usize = { 35 * 35 } as usize;

pub fn ui_test() {
    wait_for_no_keydown();
    fill_screen(Color::from_rgb888(1, 115, 55));
    display::wait_for_vblank();
    for i in 0..5 {
        draw_tile::<PIXELS>(&TILESET, Point::new(i * 37, 0), Point::new(i, 0), 1, false);
        draw_tile::<PIXELS>(&TILESET, Point::new(i * 37, 35), Point::new(i, 1), 1, true);
    }
    draw_tile::<PIXELS>(&TILESET, Point::new(0, 0), Point::new(3, 2), 1, true);
    draw_tile::<PIXELS>(&TILESET, Point::new(0, 35), Point::new(3, 3), 1, true);
}
