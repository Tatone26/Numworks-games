use numworks_utils::eadk::{Color, Point, Rect};

use crate::image::Image;

#[derive(Clone, Copy)]
pub struct Sprite<'a> {
    pub pos: Point,
    pub linked_image: &'a Image,
    pub linked_image_part: Rect,
    pub transparency: Option<Color>,
    pub z_position: u8,
    pub sprite_type: SpriteType,
    pub moved: bool,
}

impl<'a> Sprite<'a> {
    pub fn collide_with(&self, other: &Self) -> bool {
        todo!()
    }

    pub fn move_to(&mut self, pos: &Point) {
        self.pos = *pos;
    }

    pub fn draw(&self) {
        if let Some(c) = self.transparency {
            self.linked_image
                .draw_part_with_transparency(self.linked_image_part, self.pos, c);
        } else {
            self.linked_image
                .draw_part(self.linked_image_part, self.pos);
        }
    }
}

impl PartialOrd for Sprite<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Sprite<'_> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.z_position.cmp(&other.z_position)
    }
}
impl PartialEq for Sprite<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.z_position == other.z_position
    }
}

impl Eq for Sprite<'_> {}

#[derive(Clone, Copy)]
pub enum SpriteType {
    Movable,
    Fixed,
}
