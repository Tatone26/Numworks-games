use numworks_utils::eadk::{Color, Point, Rect};

use crate::image::Image;

/* #[derive(Clone, Copy)]
pub enum SpriteType {
    Movable,
    Fixed,
} */

/// Represents a Sprite (the image) that can easily be drawn.
#[derive(Clone, Copy)]
pub struct Sprite<'a> {
    /// Where in the world it is. For now only positive values.
    pub pos: Point,
    /// A link to the entire image from which the sprite's image is taken. Images should be static anyway.
    pub linked_image: &'a Image,
    /// For now, the drawing size is decided via the linked_image_part size. Can be changed later each frame for animations.
    pub linked_image_part: Rect,
    /// The color to use for transparency. Please use "None" where none is needed, as it speeds up the drawing a lot.
    pub transparency: Option<Color>,
    /// The order of printing. Higher Z will be drawned later, making it look like it's on front of the others.
    pub z_position: u8,
}

impl<'a> Sprite<'a> {
    pub fn new(
        pos: Point,
        image: &'a Image,
        image_part: Rect,
        transparency: Option<Color>,
        z_position: u8,
    ) -> Self {
        /* let mut new_part = Rect {
            x: image_part.x,
            y: image_part.y,
            width: image_part.width,
            height: image_part.height,
        };
        if new_part.x >= image.width {
            new_part.x = new_part.width - 1;
        }
        if new_part.y >= image.height {
            new_part.y = image.height - 1;
        }
        if new_part.x + new_part.width >= image.width {
            new_part.width = image.width - new_part.x - 1;
        }
        if new_part.y + new_part.height >= image.height {
            new_part.height = image.height - new_part.y - 1;
        } */

        return Sprite {
            pos,
            linked_image: image,
            linked_image_part: image_part,
            transparency,
            z_position,
        };
    }
    pub fn collide_with(&self, _other: &Self) -> bool {
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
