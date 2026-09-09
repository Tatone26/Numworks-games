//! Bitset-based dirty cell tracking for partial screen updates.

use numworks_utils::eadk::Rect;

/// Bitset tracking invalidation states across a 2D tile grid.
///
/// Uses one `u32` bitmask per row, supporting up to 32 grid columns without heap allocation.
///
/// # Generics
/// * `TILE_SIZE` - Pixel dimension of each grid cell (typically 20).
/// * `COLS` - Total columns in the viewport grid (must be `<= 32`).
/// * `ROWS` - Total rows in the viewport grid.
pub(crate) struct DirtyGrid<const TILE_SIZE: usize, const COLS: usize, const ROWS: usize> {
    /// Cells marked dirty during the current frame update.
    pub curr: [u32; ROWS],
    /// Cells that were dirty during the previous frame (cleared regions requiring redraw).
    pub prev: [u32; ROWS],
}

// Notice: <const TILE_SIZE: usize, const COLS: usize, const ROWS: usize> goes after impl too!
impl<const TILE_SIZE: usize, const COLS: usize, const ROWS: usize>
    DirtyGrid<TILE_SIZE, COLS, ROWS>
{
    /// Creates an empty dirty grid with all bits set to 0.
    pub const fn new() -> Self {
        Self {
            curr: [0; ROWS],
            prev: [0; ROWS],
        }
    }

    /// Marks all cells intersecting a given pixel rectangle as dirty in the current frame.
    pub fn mark_rect(&mut self, rect: Rect) {
        if rect.x >= (COLS * TILE_SIZE) as u16 || rect.y >= (ROWS * TILE_SIZE) as u16 {
            return;
        }
        let col_start = (rect.x as usize / TILE_SIZE).min(COLS - 1);
        let col_end =
            ((rect.x as usize + rect.width as usize).saturating_sub(1) / TILE_SIZE).min(COLS - 1);
        let row_start = (rect.y as usize / TILE_SIZE).min(ROWS - 1);
        let row_end =
            ((rect.y as usize + rect.height as usize).saturating_sub(1) / TILE_SIZE).min(ROWS - 1);

        for r in row_start..=row_end {
            for c in col_start..=col_end {
                self.curr[r] |= 1u32 << c;
            }
        }
    }

    /// Flags a single cell coordinate `(cx, cy)` as dirty.
    #[inline(always)]
    pub fn mark_cell(&mut self, cx: usize, cy: usize) {
        if cx < COLS && cy < ROWS {
            self.curr[cy] |= 1u32 << cx;
        }
    }

    /// Sets all bits across all rows to dirty, forcing a complete screen redraw.
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

    /// Rotates dirty buffers at the end of a frame, moving `curr` into `prev` and resetting `curr` to zero.
    #[inline(always)]
    pub fn swap_and_clear(&mut self) {
        self.prev = self.curr;
        self.curr = [0; ROWS];
    }
}
