use numworks_utils::eadk::{
    display::{SCREEN_HEIGHT, SCREEN_WIDTH},
    Point,
};

pub struct DirtyGrid<const TILE_SIZE: usize, const COLS: usize, const ROWS: usize> {
    pub curr: [u32; ROWS],
    pub prev: [u32; ROWS],
}

// Notice: <const TILE_SIZE: usize, const COLS: usize, const ROWS: usize> goes after impl too!
impl<const TILE_SIZE: usize, const COLS: usize, const ROWS: usize>
    DirtyGrid<TILE_SIZE, COLS, ROWS>
{
    pub const fn new() -> Self {
        Self {
            curr: [0; ROWS],
            prev: [0; ROWS],
        }
    }

    pub fn mark_rect(&mut self, pos: Point, w: u16, h: u16) {
        if w == 0 || h == 0 {
            return;
        }

        let px0 = (pos.x).min(SCREEN_WIDTH - 1) as usize;
        let py0 = (pos.y).min(SCREEN_HEIGHT - 1) as usize;
        let px1 = (pos.x + w - 1).min(SCREEN_WIDTH - 1) as usize;
        let py1 = (pos.y + h - 1).min(SCREEN_HEIGHT - 1) as usize;

        let tx0 = (px0 / TILE_SIZE).min(COLS - 1);
        let ty0 = (py0 / TILE_SIZE).min(ROWS - 1);
        let tx1 = (px1 / TILE_SIZE).min(COLS - 1);
        let ty1 = (py1 / TILE_SIZE).min(ROWS - 1);

        let mask = ((1u32 << (tx1 - tx0 + 1)) - 1) << tx0;

        for cy in ty0..=ty1 {
            self.curr[cy] |= mask;
        }
    }

    pub fn mark_all(&mut self) {
        let full_row_mask = if COLS >= 32 {
            u32::MAX
        } else {
            (1u32 << COLS) - 1
        };

        for cy in 0..ROWS {
            self.curr[cy] = full_row_mask;
        }
    }

    #[inline(always)]
    pub fn swap_and_clear(&mut self) {
        self.prev = self.curr;
        self.curr = [0; ROWS];
    }
}
