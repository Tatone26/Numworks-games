use crate::game::{Grid, Space, GRID_WIDTH};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    #[inline(always)]
    pub const fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }

    #[inline(always)]
    pub const fn to_vector(self) -> (i16, i16) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Right => (1, 0),
            Direction::Left => (-1, 0),
        }
    }
}

/// Checks whether a tile position can be traversed in the given direction.
#[inline(always)]
pub fn can_go_to(tx: i16, ty: i16, dir: Direction, grid: &Grid) -> bool {
    let (dx, dy) = dir.to_vector();
    let nx = tx + dx;
    let ny = ty + dy;

    // Side tunnel edges are passable
    if nx < 0 || nx >= GRID_WIDTH as i16 {
        return ny == 13 || ny == 14;
    }
    if ny < 0 || ny >= 30 {
        return false;
    }

    let idx = ny as usize * GRID_WIDTH + nx as usize;
    grid[idx] != Space::Wall
}

#[inline(always)]
pub fn distance_sq(x1: i16, y1: i16, x2: i16, y2: i16) -> u32 {
    let dx = x1 as i32 - x2 as i32;
    let dy = y1 as i32 - y2 as i32;
    (dx * dx + dy * dy) as u32
}
