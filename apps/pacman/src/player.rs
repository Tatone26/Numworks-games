use numworks_utils::eadk::{key, keyboard, timing, Point};

use crate::{
    game::{Grid, Space, GRID_WIDTH, STEPS_PER_CELL},
    moveable::{can_go_to, Direction, Moveable},
};

const SUPERBALL_DURATION_MS: u64 = 7000;
const SUPERBALL_BLINK_WINDOW_MS: u64 = 2000; // Blinks during the last 2 seconds

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SuperballEvent {
    None,
    BlinkingStarted,
    Expired,
}

#[derive(Clone, Copy)]
pub struct Player {
    pub moveable: Moveable,
    pub superball_active: bool,
    superball_until: u64,
    blinking_triggered: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            moveable: Moveable::new(
                Point { x: 13, y: 22 },
                Point { x: 14, y: 22 },
                Direction::Right,
                1.0,
            ),
            superball_active: false,
            superball_until: 0,
            blinking_triggered: false,
        }
    }

    pub fn handle_events(&mut self) -> SuperballEvent {
        if self.superball_until == 0 {
            return SuperballEvent::None;
        }

        let now = timing::millis();
        if now >= self.superball_until {
            self.superball_active = false;
            self.superball_until = 0;
            self.blinking_triggered = false;
            self.moveable.speed = 1.0;
            SuperballEvent::Expired
        } else if now
            >= self
                .superball_until
                .saturating_sub(SUPERBALL_BLINK_WINDOW_MS)
            && !self.blinking_triggered
        {
            self.blinking_triggered = true;
            SuperballEvent::BlinkingStarted
        } else {
            SuperballEvent::None
        }
    }

    pub fn activate_superball(&mut self) {
        self.superball_active = true;
        self.superball_until = timing::millis() + SUPERBALL_DURATION_MS;
        self.blinking_triggered = false;
        self.moveable.speed = 1.5;
    }

    pub fn move_player(&mut self, grid: &mut Grid) -> (Option<Space>, SuperballEvent) {
        let event = self.handle_events();

        let on = self.moveable.move_moveable(grid, false);
        let eaten = match on {
            Space::Superball => {
                if self.moveable.steps >= 3.0 * (STEPS_PER_CELL / 4.0) {
                    let idx = (self.moveable.destination.x
                        + self.moveable.destination.y * GRID_WIDTH)
                        as usize;
                    grid[idx] = Space::Empty;
                    self.activate_superball();
                    Some(Space::Superball)
                } else {
                    None
                }
            }
            Space::Point => {
                if self.moveable.steps >= STEPS_PER_CELL / 4.0 {
                    let idx = (self.moveable.destination.x
                        + self.moveable.destination.y * GRID_WIDTH)
                        as usize;
                    grid[idx] = Space::Empty;
                    Some(Space::Point)
                } else {
                    None
                }
            }
            Space::Fruit => {
                if self.moveable.steps >= STEPS_PER_CELL / 4.0 {
                    let idx = (self.moveable.destination.x
                        + self.moveable.destination.y * GRID_WIDTH)
                        as usize;
                    grid[idx] = Space::Empty;
                    Some(Space::Fruit)
                } else {
                    None
                }
            }
            _ => None,
        };

        (eaten, event)
    }

    pub fn read_input(&mut self, grid: &Grid) {
        let scan = keyboard::scan();
        let new_dir = if scan.key_down(key::UP)
            && can_go_to(self.moveable.grid_position, &Direction::Up, grid)
        {
            Direction::Up
        } else if scan.key_down(key::DOWN)
            && can_go_to(self.moveable.grid_position, &Direction::Down, grid)
        {
            Direction::Down
        } else if scan.key_down(key::RIGHT)
            && can_go_to(self.moveable.grid_position, &Direction::Right, grid)
        {
            Direction::Right
        } else if scan.key_down(key::LEFT)
            && can_go_to(self.moveable.grid_position, &Direction::Left, grid)
        {
            Direction::Left
        } else {
            self.moveable.direction
        };
        self.moveable.change_direction(new_dir);
    }
}
