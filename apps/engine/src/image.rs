use numworks_utils::eadk::{display::push_rect_ptr, Color, Point, Rect};

pub struct Image {
    pub width: u16,
    pub height: u16,
    data: &'static [u8],
}

impl Image {
    pub fn from_bytes(input: &'static [u8]) -> Self {
        Self {
            width: ((input[0] as u16) << 8 | (input[1] as u16)),
            height: ((input[2] as u16) << 8 | (input[3] as u16)),
            data: &input[3..], // not 100% sure of that, but seems to work for now.
        }
    }

    pub fn draw(&self, pos: Point) {
        let rect = Rect {
            x: pos.x,
            y: pos.y,
            width: self.width,
            height: self.height,
        };
        push_rect_ptr(rect, self.data.as_ptr().cast::<Color>()); // had to add a new function to eadk.rs, because can't create a Color slice.
    }

    #[inline]
    fn raw_number(&self, n: u16) -> &'static [u8] {
        &self.data[((self.width * n * 2) as usize)..]
    }

    // This version does multiple pushes to screen, so not to have to copy into RAM the image to display.
    // If we were to copy first, we would need a hard-coded size.
    pub fn draw_part(&self, part: Rect, pos: Point) {
        debug_assert!(part.x + part.width <= self.width);
        debug_assert!(part.y + part.height <= self.height);
        // Since the images are stored row by row, printing row by row is easy.
        let mut rect = Rect {
            x: pos.x,
            y: pos.y,
            width: part.width,
            height: 1,
        };
        for i in part.y..(part.y + part.height) {
            push_rect_ptr(rect, self.raw_number(i).as_ptr().cast::<Color>());
            rect.y += 1;
        }
    }
}
