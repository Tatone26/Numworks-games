use heapless::{binary_heap::Min, BinaryHeap};
use numworks_utils::{
    eadk::{display::push_rect_uniform, Color, Rect},
    utils::fill_screen,
};

use crate::sprite::Sprite;

/// Represents a Graphical Scene, essentially a list of sprites.
///
/// Used to simplify the relationship between sprites.
///
/// Since sprites are likely to be linked to more complicated structs, it only stores references to them.
///
/// the Sprites need to have a similar lifetime as the Scene (all created at the same time if possible).
pub struct Scene<'a, const MAX_SPRITES: usize> {
    sprites: BinaryHeap<&'a Sprite<'a>, Min, MAX_SPRITES>,
    background_color: Color,
}

impl<'a, const MAX_SPRITES: usize> Scene<'a, MAX_SPRITES> {
    pub fn new(c: Color) -> Self {
        Scene {
            background_color: c,
            ..Default::default()
        }
    }

    /// Adds a sprite to the list.
    ///
    /// WARNING : it is then impossible to remove.
    pub fn insert(&mut self, object: &'a Sprite) {
        let _ = self.sprites.push(object);
    }

    /// Quite explicit. And stupid.
    pub fn draw_entire_scene(&self) {
        fill_screen(self.background_color);
        for b in self.sprites.iter() {
            b.draw();
        }
    }

    /// Clears a sprite, very simply (just fills it with background color)
    ///
    /// Can be used for simple movement : clears, redraw what is behind, and redraws at the new position. Not optimised.
    pub fn clear_sprite(&mut self, sprite: &'a Sprite) {
        push_rect_uniform(
            Rect {
                x: sprite.pos.x,
                y: sprite.pos.y,
                width: sprite.linked_image_part.width,
                height: sprite.linked_image_part.height,
            },
            self.background_color,
        );
    }

    // TODO : a function to redraw a specific sprite and everything that was behind it, and everything that is now in front of it.
}

impl<'a, const MAX_SPRITES: usize> Default for Scene<'a, MAX_SPRITES> {
    fn default() -> Self {
        Self {
            sprites: BinaryHeap::<&'a Sprite, Min, MAX_SPRITES>::new(),
            // need_to_redraw: Deque::default(),
            background_color: Color::WHITE,
        }
    }
}
