use heapless::Vec;

use crate::{eadk::Color, utils::randint};

#[derive(Clone)]
pub enum TetriType {
    T,
    J,
    Z,
    O,
    S,
    L,
    I,
}

/// Exactly like a point, but signed
#[derive(Clone, Copy)]
pub struct SignedPoint {
    pub x: i16,
    pub y: i16,
}

impl SignedPoint {
    /*
    pub const fn from_unsigned_point(p : Point) -> SignedPoint{
        return SignedPoint { x: p.x as i16, y: p.y as i16 }
    }
    */
}

/// Represents a shape, its type, rotation (0-3), color
#[derive(Clone)]
pub struct Tetrimino {
    pub tetri: TetriType,
    pub rotation: u8,
    pub color: Color,
    pub pos: SignedPoint,
}

impl Tetrimino {
    /// Rotate the tetri to the right
    pub fn rotate_right(&mut self) {
        // needs tp check if can be rotated or not lol
        if self.rotation < 3 {
            self.rotation += 1
        } else {
            self.rotation = 0
        }
    }

    /// Rotate the tetri to the left
    pub fn rotate_left(&mut self) {
        if self.rotation > 0 {
            self.rotation -= 1
        } else {
            self.rotation = 3
        }
    }

    /// Returns the position of all blocks of the tetri
    /// The center of the tetri is (0, 0) every time, so need to call something like tetri.pos.x + x to get absolute pos
    pub fn get_blocks(&self) -> [(i16, i16); 4] {
        match self.tetri {
            TetriType::T => match self.rotation {
                0 => return [(-1, 0), (0, 0), (0, -1), (1, 0)],
                1 => return [(0, 1), (0, 0), (0, -1), (1, 0)],
                2 => return [(-1, 0), (0, 0), (0, 1), (1, 0)],
                _ => return [(-1, 0), (0, 0), (0, -1), (0, 1)],
            },
            TetriType::J => match self.rotation {
                0 => return [(-1, 1), (0, 1), (0, 0), (0, -1)],
                1 => return [(-1, 0), (-1, -1), (0, 0), (1, 0)],
                2 => return [(0, 1), (0, 0), (0, -1), (1, -1)],
                _ => return [(-1, 0), (0, 0), (1, 0), (1, 1)],
            },
            TetriType::Z => match self.rotation {
                0 | 2 => return [(-1, 0), (0, 0), (0, 1), (1, 1)],
                _ => return [(0, 0), (0, 1), (1, 0), (1, -1)],
            },
            TetriType::O => return [(0, 0), (-1, 0), (-1, 1), (0, 1)],
            TetriType::S => match self.rotation {
                0 | 2 => return [(1, 0), (0, 0), (0, 1), (-1, 1)],
                _ => return [(0, 0), (1, 1), (1, 0), (0, -1)],
            },
            TetriType::L => match self.rotation {
                0 => return [(1, 1), (0, 1), (0, 0), (0, -1)],
                1 => return [(-1, 0), (-1, 1), (0, 0), (1, 0)],
                2 => return [(0, 1), (0, 0), (0, -1), (-1, -1)],
                _ => return [(-1, 0), (0, 0), (1, 0), (1, -1)],
            },
            TetriType::I => match self.rotation {
                0 | 2 => return [(0, 0), (0, 1), (0, -1), (0, -2)],
                _ => return [(0, 0), (1, 0), (-1, 0), (-2, 0)],
            },
        }
    }

    pub fn get_blocks_grid_pos(&self) -> [SignedPoint; 4] {
        let mut res = Vec::<SignedPoint, 4>::new();
        for (i, e) in self.get_blocks().iter().enumerate() {
            res.push(SignedPoint {
                x: e.0 + self.pos.x,
                y: e.1 + self.pos.y,
            });
        }
        unsafe {
            return res.into_array().unwrap_unchecked();
        }
    }
}

pub const T_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::T,
    rotation: 2,
    color: Color::RED,
    pos: SignedPoint { x: 5, y: 0 },
};
pub const J_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::J,
    rotation: 3,
    color: Color::BLUE,
    pos: SignedPoint { x: 5, y: 0 },
};
pub const Z_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::Z,
    rotation: 0,
    color: Color::GREEN,
    pos: SignedPoint { x: 5, y: 0 },
};
pub const O_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::O,
    rotation: 0,
    color: Color::from_rgb888(255, 255, 0),
    pos: SignedPoint { x: 5, y: 0 },
};
pub const S_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::S,
    rotation: 0,
    color: Color::from_rgb888(0, 255, 255),
    pos: SignedPoint { x: 5, y: 0 },
};
pub const L_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::L,
    rotation: 1,
    color: Color::from_rgb888(255, 0, 255),
    pos: SignedPoint { x: 5, y: 0 },
};
pub const I_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::I,
    rotation: 1,
    color: Color::from_rgb888(100, 255, 100),
    pos: SignedPoint { x: 5, y: 0 },
};

/// Returns a new random tetrimino
pub fn get_new_tetrimino() -> Tetrimino {
    match randint(0, 6) {
        0 => return T_SHAPE,
        1 => return J_SHAPE,
        2 => return O_SHAPE,
        3 => return L_SHAPE,
        4 => return I_SHAPE,
        5 => return S_SHAPE,
        _ => return Z_SHAPE,
    }
}

pub fn get_random_bag() -> Vec<Tetrimino, 7> {
    let mut res = Vec::<Tetrimino, 7>::new();
    let mut tetri_list: Vec<Tetrimino, 7> = Vec::from_slice(&[
        T_SHAPE, J_SHAPE, O_SHAPE, L_SHAPE, I_SHAPE, S_SHAPE, Z_SHAPE,
    ])
    .unwrap();

    for _ in 0..6 {
        unsafe {
            res.push(tetri_list.swap_remove(randint(0, tetri_list.len() as u32 - 1) as usize))
                .unwrap_unchecked();
        }
    }

    unsafe {
        res.push(tetri_list.swap_remove(0)).unwrap_unchecked();
    }

    return res;
}
