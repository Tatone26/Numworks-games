use crate::world::{GRID_HEIGHT, GRID_WIDTH};
use numworks_utils::eadk::{key, timing, Point, State};

#[derive(Copy, Clone, PartialEq)]
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

    /// Calculates target coordinates and clamps strictly within grid bounds
    pub fn try_move(&self, point: Point) -> Option<Point> {
        let vec = self.to_vector();
        let nx = point.x as i16 + vec[0] as i16;
        let ny = point.y as i16 + vec[1] as i16;

        if nx >= 0 && nx < GRID_WIDTH as i16 && ny >= 0 && ny < GRID_HEIGHT as i16 {
            Some(Point {
                x: nx as u16,
                y: ny as u16,
            })
        } else {
            None // Block movement past screen edges
        }
    }
}

const JUMP_DURATION_MS: u64 = 500;

pub struct Frog {
    grid_pos: Point,
    speed: f32,
    direction: Direction,
    is_landed: bool,
    lands_at: u64,
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

    /// Triggers jump cooldown without changing grid position (used during scrolling)
    pub fn start_cooldown(&mut self) {
        self.direction = Direction::Up;
        self.is_landed = false;
        self.lands_at = timing::millis() + JUMP_DURATION_MS;
    }

    pub fn jump(&mut self, direction: Direction) {
        if let Some(new_pos) = direction.try_move(self.grid_pos) {
            self.grid_pos = new_pos;
            self.direction = direction;
            self.start_cooldown();
        }
    }
}
