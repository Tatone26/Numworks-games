use numworks_utils::eadk::{display::push_rect, Color, Point, Rect};

/// This struct is to be used with images included in executable, via include_bytes! macro.
/// data points to the same data as include_bytes, it just reads it as a Color slice.
pub struct Image {
    pub width: u16,
    pub height: u16,
    data: &'static [Color],
}

impl Image {
    /// WARNING : the image NEED to be made via MY ppm_decoder, that can be found in this repo too. Will think of making it automatic later.
    /// Otherwise, big boom and crash of the calculator (probably).
    /// This image is in the format width (2 bytes), height (2 bytes), separator (1 null byte), all colors (2 bytes each), everything next to one another without delimiter or anything.
    /// No need for real precautions, we are working on a calculator that forgets everything every reset.
    pub fn from_bytes(input: &'static [u8]) -> Self {
        Self {
            width: ((input[0] as u16) << 8 | (input[1] as u16)),
            height: ((input[2] as u16) << 8 | (input[3] as u16)),
            data: unsafe { &input.align_to().1[2..] },
        }
    }

    /// draws the entire image to screen at the given pos. May not be useful, but it is the fastest option.
    pub fn draw(&self, pos: Point) {
        let rect = Rect {
            x: pos.x,
            y: pos.y,
            width: self.width,
            height: self.height,
        };
        push_rect(rect, self.data);
    }

    #[inline]
    fn get_row_at_height(&self, n: u16) -> &'static [Color] {
        &self.data[(self.width * n) as usize..]
    }

    // This version does multiple pushes to screen, so not to have to copy into RAM the image to display.
    // If we were to copy first, we would need a hard-coded number of pixels, and that's bad.
    pub fn draw_part(&self, part: Rect, pos: Point) {
        debug_assert!(part.x + part.width <= self.width);
        debug_assert!(part.y + part.height <= self.height);
        // Since the images are stored row by row, printing row by row is kind of the only option.
        let mut rect = Rect {
            x: pos.x,
            y: pos.y,
            width: part.width,
            height: 1,
        };
        for i in part.y..(part.y + part.height) {
            push_rect(rect, &self.get_row_at_height(i)[(part.x as usize)..]);
            rect.y += 1;
        }
    }

    #[allow(clippy::clone_on_copy)]
    pub fn draw_part_with_transparency(&self, part: Rect, pos: Point, transparency: Color) {
        debug_assert!(part.x + part.width <= self.width);
        debug_assert!(part.y + part.height <= self.height);
        // for each row
        for i in 0..(part.height) {
            let row = self.get_row_at_height(i + part.y);
            let mut last: Option<u16> = None;

            // for each (useful) pixel of the row
            for j in 0..(part.width) {
                // do NOT remove the .clone(), even if clippy says so, because it result in a crash (don't ask me)
                let pixel: Color = row[(part.x + j) as usize];
                if pixel.rgb565 == transparency.rgb565 {
                    // if we have a transparent pixel at position j.
                    if let Some(s) = last {
                        // if we were on a non-transparent streak
                        // doing this this way to minimise the number of calls to the display, which are very slow.
                        let rect = Rect {
                            x: pos.x + s,
                            y: pos.y + i,
                            width: j.saturating_sub(s),
                            height: 1,
                        };
                        push_rect(rect, &row[(part.x + s) as usize..]);
                        last = None;
                    }
                    // else we have nothing to do.
                } else if last.is_none() {
                    last = Some(j);
                }
            }
            if let Some(s) = last {
                let rect = Rect {
                    x: pos.x + s,
                    y: pos.y + i,
                    width: part.width.saturating_sub(s),
                    height: 1,
                };
                push_rect(rect, &row[(part.x + s) as usize..]);
            }
        }
    }
}
