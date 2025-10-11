use numworks_utils::eadk::Point;

use crate::game::{Grid, Space, GRID_HEIGHT, GRID_WIDTH, STEPS_PER_CELL};

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    pub const fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }

    pub const fn to_vector(self) -> (i16, i16) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Right => (1, 0),
            Direction::Left => (-1, 0),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Moveable {
    pub grid_position: Point,
    pub destination: Point,
    pub steps: f32,
    pub direction: Direction,
    pub speed: f32,
    pub wrapping: bool,
}

impl Moveable {
    pub fn new(grid_position: Point, destination: Point, direction: Direction, speed: f32) -> Self {
        Self {
            grid_position,
            destination,
            steps: 0.0,
            direction,
            speed,
            wrapping: false,
        }
    }

    /// Returns the type of cell it ends up on
    pub fn move_moveable(&mut self, grid: &mut Grid, ignore_walls: bool) -> Space {
        if self.grid_position.x == self.destination.x && self.grid_position.y == self.destination.y
        {
            return Space::Empty;
        }
        self.steps += self.speed;
        // if reached end of current cell, then move to next cell
        if self.steps >= STEPS_PER_CELL {
            // if arrived to next node
            self.grid_position = self.destination;
            let (next_pos, wrap) = next_pos(self.grid_position, &self.direction);
            self.wrapping = wrap;
            (self.destination, self.steps) =
                match grid.get((next_pos.x + next_pos.y * GRID_WIDTH) as usize) {
                    Some(Space::Wall) => (self.grid_position, 0.0),
                    Some(_) => (next_pos, self.steps % STEPS_PER_CELL as f32),
                    _ => (self.grid_position, 0.0),
                };
            if ignore_walls {
                self.destination = next_pos;
                self.steps = self.steps % STEPS_PER_CELL as f32;
            }
        };
        *grid
            .get((self.destination.x + self.destination.y * GRID_WIDTH) as usize)
            .unwrap_or(&Space::Empty)
    }

    pub fn change_direction(&mut self, new_dir: Direction) {
        if new_dir.opposite() == self.direction {
            let stopped = self.grid_position.x == self.destination.x
                && self.grid_position.y == self.destination.y;
            self.direction = new_dir;
            self.grid_position = self.destination;
            (self.destination, self.wrapping) = next_pos(self.grid_position, &self.direction);
            if !stopped {
                self.steps = f32::abs(STEPS_PER_CELL - self.steps);
            } else {
                self.steps = 0.0;
            }
        } else if new_dir != self.direction {
            self.steps = 0.0;
            self.direction = new_dir;
            (self.destination, self.wrapping) = next_pos(self.grid_position, &self.direction);
        }
    }
}

pub fn can_go_to(from: Point, dir: &Direction, grid: &Grid) -> bool {
    let (next, _) = next_pos(from, dir);
    match grid.get((next.x + next.y * GRID_WIDTH) as usize) {
        Some(Space::Wall) => false,
        Some(_) => true,
        None => true,
    }
}

pub fn next_pos(from: Point, dir: &Direction) -> (Point, bool) {
    let next = (
        (from.x as i16 + dir.to_vector().0),
        (from.y as i16 + dir.to_vector().1),
    );
    if next.0 < 0 || next.1 < 0 || next.0 >= GRID_WIDTH as i16 || next.1 >= GRID_HEIGHT as i16 {
        // wrapping
        (
            Point {
                x: ((GRID_WIDTH as i16 + next.0) % GRID_WIDTH as i16) as u16,
                y: ((GRID_HEIGHT as i16 + next.1) % GRID_HEIGHT as i16) as u16,
            },
            true,
        )
    } else {
        (
            Point {
                x: next.0 as u16,
                y: next.1 as u16,
            },
            false,
        )
    }
}
