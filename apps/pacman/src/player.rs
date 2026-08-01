use numworks_utils::eadk::{key, keyboard, timing, Point};

use crate::{
    game::{grid_index, Grid, Space, STEPS_PER_CELL},
    levels::LevelConfig,
    moveable::{can_go_to, Direction, Moveable},
};

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
    blink_at: u64,
    blinking_triggered: bool,
    pub dots_eaten: u16,
    just_ate: bool,
}

impl Player {
    pub fn new(config: &LevelConfig) -> Self {
        Self {
            moveable: Moveable::new(
                Point { x: 13, y: 22 },
                Point { x: 14, y: 22 },
                Direction::Right,
                config.pac_speed,
            ),
            superball_active: false,
            superball_until: 0,
            blinking_triggered: false,
            dots_eaten: 0,
            just_ate: false,
            blink_at: 0,
        }
    }

    fn consume_item(&mut self, grid: &mut Grid, threshold: f32) -> bool {
        if self.moveable.steps >= threshold {
            let idx = grid_index(self.moveable.destination);
            grid[idx] = Space::Empty;
            self.just_ate = true;
            true
        } else {
            false
        }
    }

    pub fn activate_superball(&mut self, config: &LevelConfig) {
        let duration = config.frightened_duration_ms;
        if duration == 0 {
            return;
        }

        let now = timing::millis();
        self.superball_active = true;
        self.superball_until = now + duration;
        self.blink_at = self.superball_until.saturating_sub(config.blink_window_ms);
        self.blinking_triggered = false;
        self.moveable.speed = config.pac_speed * 1.15;
    }

    pub fn handle_events(&mut self, config: &LevelConfig) -> SuperballEvent {
        if self.superball_until == 0 {
            return SuperballEvent::None;
        }

        let now = timing::millis();
        if now >= self.superball_until {
            self.superball_active = false;
            self.superball_until = 0;
            self.blinking_triggered = false;
            self.moveable.speed = config.pac_speed;
            SuperballEvent::Expired
        } else if self.blink_at != 0 && now >= self.blink_at && !self.blinking_triggered {
            self.blinking_triggered = true;
            SuperballEvent::BlinkingStarted
        } else {
            SuperballEvent::None
        }
    }

    pub fn move_player(
        &mut self,
        config: &LevelConfig,
        grid: &mut Grid,
    ) -> (Option<Space>, SuperballEvent) {
        let event = self.handle_events(config);

        if self.just_ate {
            self.just_ate = false;
            return (None, event);
        }

        let on = self.moveable.move_moveable(grid, false);
        let eaten = match on {
            Space::Superball => {
                if self.consume_item(grid, 3.0 * (STEPS_PER_CELL / 4.0)) {
                    self.activate_superball(config);
                    Some(Space::Superball)
                } else {
                    None
                }
            }
            Space::Point => {
                if self.consume_item(grid, STEPS_PER_CELL / 4.0) {
                    Some(Space::Point)
                } else {
                    None
                }
            }
            Space::Fruit => {
                if self.consume_item(grid, STEPS_PER_CELL / 4.0) {
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
