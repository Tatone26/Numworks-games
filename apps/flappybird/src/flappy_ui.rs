use numworks_utils::{
    eadk::{display::push_rect_uniform, Color, Point, Rect},
    utils::{draw_tile, Tileset},
};

/// Images work really well with square tiles. You can still draw other images, but it is less good.
pub static TILESET: Tileset = Tileset {
    tile_size: 20,
    image: include_bytes!("./image.ppm"),
};
pub const PIXELS: usize = { 20 * 20 } as usize;

pub const BACKGROUND: Color = Color::from_rgb888(128, 212, 255);

pub fn draw_bird(pos: Point) {
    draw_tile::<PIXELS>(&TILESET, pos, Point { x: 0, y: 0 }, 1, true);
}

pub fn clear_tile(pos: Point) {
    push_rect_uniform(
        Rect {
            x: pos.x,
            y: pos.y,
            width: 20,
            height: 20,
        },
        BACKGROUND,
    )
}
