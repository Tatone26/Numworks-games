use numworks_utils::eadk::{key, timing, Point, State};

#[derive(Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn to_vector(&self) -> [i8; 2] {
        match self {
            Direction::Up => [0, -1],
            Direction::Down => [0, 1],
            Direction::Left => [-1, 0],
            Direction::Right => [1, 0],
        }
    }

    pub fn add_to_point(&self, point: Point) -> Point {
        let vector = self.to_vector();
        Point {
            x: (point.x as i16 + vector[0] as i16).max(0) as u16,
            y: (point.y as i16 + vector[1] as i16).max(0) as u16,
        }
    }
}

const JUMP_DURATION_MS: u64 = 500; // duration of the jump in milliseconds

pub struct Frog {
    grid_pos: Point,
    speed: f32,
    direction: Direction,
    is_landed: bool,
    lands_at: u64, // timing
}

impl Frog {
    pub fn new(grid_pos: Point) -> Self {
        Frog {
            grid_pos,
            speed: 1.0,
            direction: Direction::Up,
            is_landed: true,
            lands_at: 0,
        }
    }

    pub fn grid_pos(&self) -> Point {
        self.grid_pos
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    // called every frame
    pub fn update(&mut self) {
        if !self.is_landed && timing::millis() >= self.lands_at {
            self.is_landed = true;
        }
    }

    pub fn read_input(&mut self, scan: State) -> Option<Direction> {
        if !self.is_landed {
            return None;
        }
        if scan.key_down(key::UP) {
            Some(Direction::Up)
        } else if scan.key_down(key::DOWN) {
            Some(Direction::Down)
        } else if scan.key_down(key::RIGHT) {
            Some(Direction::Right)
        } else if scan.key_down(key::LEFT) {
            Some(Direction::Left)
        } else {
            None
        }
    }

    pub fn jump(&mut self, direction: Direction) {
        let vector = direction.to_vector();
        let temp_pos: [i16; 2] = [
            self.grid_pos.x as i16 + vector[0] as i16,
            self.grid_pos.y as i16 + vector[1] as i16,
        ];
        if temp_pos[0] < 0 || temp_pos[1] < 0 {
            return;
        }
        self.grid_pos.x = temp_pos[0] as u16;
        self.grid_pos.y = temp_pos[1] as u16;
        self.is_landed = false;
        self.lands_at = timing::millis() + JUMP_DURATION_MS;
    }
}
