use heapless::Vec;

use crate::utils::randint;

#[derive(Clone, PartialEq)]
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
#[derive(Clone, Copy, Debug)]
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
    pub color: u8, // position in Tilemap
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
                0 => [(0, 0), (-1, 0), (1, 0), (0, -1)],
                1 => [(0, 0), (1, 0), (0, 1), (0, -1)],
                2 => [(0, 0), (-1, 0), (1, 0), (0, 1)],
                _ => [(0, 0), (-1, 0), (0, 1), (0, -1)],
            },
            TetriType::J => match self.rotation {
                0 => [(-1, 0), (-1, -1), (0, 0), (1, 0)],
                1 => [(0, 1), (0, 0), (0, -1), (1, -1)],
                2 => [(-1, 0), (0, 0), (1, 0), (1, 1)],
                _ => [(-1, 1), (0, 1), (0, 0), (0, -1)],
            },
            TetriType::Z => match self.rotation {
                0 => [(-1, -1), (0, -1), (0, 0), (1, 0)],
                1 => [(0, 0), (0, 1), (1, 0), (1, -1)],
                2 => [(-1, 0), (0, 0), (0, 1), (1, 1)],
                _ => [(-1, 1), (-1, 0), (0, 0), (0, -1)],
            },
            TetriType::O => [(0, 0), (-1, 0), (-1, 1), (0, 1)],
            TetriType::S => match self.rotation {
                0 => [(-1, 0), (0, 0), (0, -1), (1, -1)],
                1 => [(0, 0), (1, 1), (1, 0), (0, -1)],
                2 => [(1, 0), (0, 0), (0, 1), (-1, 1)],
                _ => [(-1, -1), (-1, 0), (0, 0), (0, 1)],
            },
            TetriType::L => match self.rotation {
                0 => [(-1, 0), (0, 0), (1, 0), (1, -1)],
                1 => [(1, 1), (0, 1), (0, 0), (0, -1)],
                2 => [(-1, 0), (-1, 1), (0, 0), (1, 0)],
                _ => [(0, 1), (0, 0), (0, -1), (-1, -1)],
            },
            TetriType::I => match self.rotation {
                0 => [(0, 0), (1, 0), (-1, 0), (-2, 0)],
                1 => [(0, 0), (0, -1), (0, 1), (0, 2)],
                2 => [(0, 1), (1, 1), (-1, 1), (-2, 1)],
                _ => [(-1, 0), (-1, -1), (-1, 1), (-1, 2)],
            },
        }
    }

    pub fn get_blocks_grid_pos(&self) -> [SignedPoint; 4] {
        let mut res = Vec::<SignedPoint, 4>::new();
        for e in self.get_blocks().iter() {
            res.push(SignedPoint {
                x: e.0 + self.pos.x,
                y: e.1 + self.pos.y,
            })
            .unwrap();
        }
        res.into_array().unwrap()
    }
}

pub const T_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::T,
    rotation: 0,
    color: 6,
    pos: SignedPoint { x: 5, y: -1 },
};
pub const J_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::J,
    rotation: 0,
    color: 1,
    pos: SignedPoint { x: 5, y: -1 },
};
pub const Z_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::Z,
    rotation: 0,
    color: 4,
    pos: SignedPoint { x: 5, y: -1 },
};
pub const O_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::O,
    rotation: 0,
    color: 0,
    pos: SignedPoint { x: 5, y: -2 },
};
pub const S_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::S,
    rotation: 0,
    color: 3,
    pos: SignedPoint { x: 5, y: -1 },
};
pub const L_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::L,
    rotation: 0,
    color: 2,
    pos: SignedPoint { x: 5, y: -1 },
};
pub const I_SHAPE: Tetrimino = Tetrimino {
    tetri: TetriType::I,
    rotation: 0,
    color: 5,
    pos: SignedPoint { x: 5, y: -1 },
};

pub fn get_initial_tetri(tetritype: TetriType) -> Tetrimino {
    match tetritype {
        TetriType::T => T_SHAPE,
        TetriType::J => J_SHAPE,
        TetriType::Z => Z_SHAPE,
        TetriType::O => O_SHAPE,
        TetriType::S => S_SHAPE,
        TetriType::L => L_SHAPE,
        TetriType::I => I_SHAPE,
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

    res
}

/// Tetris.wiki uses an inversed y axis compared to me, so all y values are inversed.
static WALL_KICK_TABLE: [(i16, i16); 32] = [
    (-1, 0),
    (-1, -1),
    (0, 2),
    (-1, 2),
    (1, 0),
    (1, 1),
    (0, -2),
    (1, -2),
    (1, 0),
    (1, 1),
    (0, -2),
    (1, -2),
    (-1, 0),
    (-1, -1),
    (0, 2),
    (-1, 2),
    (1, 0),
    (1, -1),
    (0, 2),
    (1, 2),
    (-1, 0),
    (-1, 1),
    (0, -2),
    (-1, -2),
    (-1, 0),
    (-1, 1),
    (0, -2),
    (-1, -2),
    (1, 0),
    (1, -1),
    (0, 2),
    (1, 2),
];

static I_WALL_KICKS_TABLE: [(i16, i16); 32] = [
    (-2, 0),
    (1, 0),
    (1, -2),
    (-2, 1),
    (2, 0),
    (-1, 0),
    (2, -1),
    (-1, 2),
    (-1, 0),
    (2, 0),
    (-1, -2),
    (2, 1),
    (-2, 0),
    (1, 0),
    (-2, -1),
    (1, 1),
    (2, 0),
    (-1, 0),
    (2, -1),
    (-1, 1),
    (1, 0),
    (-2, 0),
    (1, -2),
    (-2, 1),
    (-2, 0),
    (1, 0),
    (-2, -1),
    (1, 2),
    (2, 0),
    (-1, 0),
    (-1, -2),
    (2, 1),
];

pub fn get_wall_kicks_data(tetri: &Tetrimino, right: bool) -> &[(i16, i16)] {
    let table: &[(i16, i16); 32] = match tetri.tetri {
        TetriType::I => &I_WALL_KICKS_TABLE,
        TetriType::O => return &[],
        _ => &WALL_KICK_TABLE,
    };
    if right {
        match tetri.rotation {
            0 => &table[0..4],
            1 => &table[8..12],
            2 => &table[16..20],
            _ => &table[24..28],
        }
    } else {
        match tetri.rotation {
            0 => &table[28..32],
            3 => &table[20..24],
            2 => &table[12..16],
            _ => &table[4..8],
        }
    }
}
