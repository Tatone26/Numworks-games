use crate::{eadk::Color, utils::randint};

pub enum TetriType {
    T,
    J,
    Z,
    O,
    S,
    L,
    I,
}

pub struct SignedPoint {
    pub x : i16,
    pub y : i16,
}

/// Represents a shape, its type, rotation (0-3), color
pub struct Tetrimino {
    pub tetri: TetriType,
    pub rotation: u8,
    pub color: Color,
    pub pos: SignedPoint,
}

impl Tetrimino {
    pub fn rotate_right(&mut self) {
        // needs tp check if can be rotated or not lol
        if self.rotation < 3 {
            self.rotation += 1
        } else {
            self.rotation = 0
        }
    }

    pub fn rotate_left(&mut self) {
        if self.rotation > 0 {
            self.rotation -= 1
        } else {
            self.rotation = 3
        }
    }

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