use heapless::{binary_heap::Min, BinaryHeap};

use crate::sprite::{Sprite, SpriteType};

pub struct Scene<'a, const MAX_SPRITES: usize> {
    movables: BinaryHeap<Sprite<'a>, Min, MAX_SPRITES>,
    fixed: BinaryHeap<Sprite<'a>, Min, MAX_SPRITES>,
}

impl<'a, const MAX_SPRITES: usize> Scene<'a, MAX_SPRITES> {
    pub fn insert(&mut self, object: &Sprite<'a>) {
        let list = match object.sprite_type {
            SpriteType::Movable => &mut self.movables,
            SpriteType::Fixed => &mut self.fixed,
        };
        let _ = list.push(*object);
    }

    pub fn draw_entire_scene(&self) {
        for b in self.fixed.iter() {
            b.draw();
        }
        for m in self.movables.iter() {
            m.draw();
        }
    }

    pub fn draw_moved_sprites(&mut self) {
        for b in self.movables.iter_mut() {
            if b.moved {
                b.draw();
                b.moved = false;
            }
        }
    }
}

impl<'a, const MAX_SPRITES: usize> Default for Scene<'a, MAX_SPRITES> {
    fn default() -> Self {
        Self {
            movables: BinaryHeap::<Sprite, Min, MAX_SPRITES>::new(),
            fixed: BinaryHeap::<Sprite, Min, MAX_SPRITES>::new(),
        }
    }
}
