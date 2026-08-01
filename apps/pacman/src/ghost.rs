use numworks_utils::eadk::{timing, Point};

use crate::{
    game::{Grid, GRID_HEIGHT, GRID_WIDTH},
    levels::LevelConfig,
    moveable::{can_go_to, next_pos, Direction, Moveable},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GhostType {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MovementMode {
    Chase,
    Scatter,
    Frightened,
    FrightenedBlinking,
    Eaten,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HouseState {
    Inside,   // Shuffling side-to-side inside 1-tall prison
    Leaving,  // Moving horizontally to center door, then UP onto maze
    Outside,  // Active on the maze
    Entering, // Eaten eyes moving DOWN through door into prison
}

pub struct Ghost {
    pub gtype: GhostType,
    pub moveable: Moveable,
    pub movement_mode: MovementMode,
    pub house_state: HouseState,

    target_cell: Point,
    home_position: Point,
    door_position: Point,

    release_timer: u64,
    wave_timer: u64,
    wave_step: u8,
    tried_change: bool,
}

impl Ghost {
    pub fn new(start: Point, gtype: GhostType, config: &LevelConfig) -> Self {
        let now = timing::millis();

        let door_position = Point {
            x: 14,
            y: start.y.saturating_sub(1),
        };

        let (house_state, release_timer, initial_dir) = match gtype {
            GhostType::Blinky => (HouseState::Outside, 0, Direction::Left),
            GhostType::Pinky => (HouseState::Leaving, 0, Direction::Up),
            GhostType::Inky => (
                HouseState::Inside,
                now + config.inky_release_ms,
                Direction::Left,
            ),
            GhostType::Clyde => (
                HouseState::Inside,
                now + config.clyde_release_ms,
                Direction::Right,
            ),
        };

        let spawn_pos = if gtype == GhostType::Blinky {
            door_position
        } else {
            start
        };

        let (initial_dest, _) = next_pos(spawn_pos, &initial_dir);

        Self {
            gtype,
            moveable: Moveable::new(spawn_pos, initial_dest, initial_dir, config.ghost_speed),
            movement_mode: MovementMode::Scatter,
            house_state,
            target_cell: Point { x: 0, y: 0 },
            home_position: start,
            door_position,
            release_timer,
            wave_timer: now + config.wave_timings.scatter_1,
            wave_step: 0,
            tried_change: false,
        }
    }

    pub fn update(
        &mut self,
        pac_position: Point,
        pac_direction: &Direction,
        blinky_pos: Point,
        grid: &mut Grid,
        config: &LevelConfig,
    ) {
        self.handle_events(config);
        self.recalculate_speed(config);

        match self.house_state {
            HouseState::Inside => {
                if self.release_timer != 0 && timing::millis() >= self.release_timer {
                    self.house_state = HouseState::Leaving;
                } else {
                    self.update_house_shuffling(grid);
                }
            }
            HouseState::Leaving => {
                self.process_house_exit(grid);
            }
            HouseState::Entering => {
                self.process_house_entry(grid, config);
            }
            HouseState::Outside => {
                self.update_outside_movement(pac_position, pac_direction, blinky_pos, grid);
            }
        }
    }

    fn is_frightened(&self) -> bool {
        matches!(
            self.movement_mode,
            MovementMode::Frightened | MovementMode::FrightenedBlinking
        )
    }

    fn is_in_tunnel(&self) -> bool {
        let cur = self.moveable.grid_position;
        (cur.x <= 5 || cur.x >= 22) && (cur.y == 12 || cur.y == 13)
    }

    fn recalculate_speed(&mut self, config: &LevelConfig) {
        self.moveable.speed = match self.movement_mode {
            MovementMode::Eaten => config.eaten_ghost_speed,
            MovementMode::Frightened | MovementMode::FrightenedBlinking => {
                config.frightened_ghost_speed
            }
            MovementMode::Chase | MovementMode::Scatter => {
                if self.is_in_tunnel() {
                    config.tunnel_ghost_speed
                } else {
                    config.ghost_speed
                }
            }
        };
    }

    fn handle_events(&mut self, config: &LevelConfig) {
        // Do not advance AI wave timers while frightened or returning as eyes
        if self.movement_mode == MovementMode::Frightened
            || self.movement_mode == MovementMode::FrightenedBlinking
            || self.movement_mode == MovementMode::Eaten
        {
            return;
        }

        let now = timing::millis();
        if self.wave_timer == 0 || now < self.wave_timer {
            return;
        }

        let timings = &config.wave_timings;
        match self.wave_step {
            0 => {
                self.wave_step = 1;
                self.movement_mode = MovementMode::Chase;
                self.wave_timer = now + timings.chase_1;
            }
            1 => {
                self.wave_step = 2;
                self.movement_mode = MovementMode::Scatter;
                self.wave_timer = now + timings.scatter_2;
            }
            2 => {
                self.wave_step = 3;
                self.movement_mode = MovementMode::Chase;
                self.wave_timer = now + timings.chase_2;
            }
            3 => {
                self.wave_step = 4;
                self.movement_mode = MovementMode::Scatter;
                self.wave_timer = now + timings.scatter_3;
            }
            4 => {
                self.wave_step = 5;
                self.movement_mode = MovementMode::Chase;
                self.wave_timer = now + timings.chase_3;
            }
            5 => {
                self.wave_step = 6;
                self.movement_mode = MovementMode::Scatter;
                self.wave_timer = now + timings.scatter_4;
            }
            6 => {
                self.wave_step = 7;
                self.movement_mode = MovementMode::Chase;
                self.wave_timer = 0;
            }
            _ => {}
        }
    }

    fn update_house_shuffling(&mut self, grid: &mut Grid) {
        if self.moveable.direction != Direction::Left && self.moveable.direction != Direction::Right
        {
            self.moveable.change_direction(Direction::Left);
        }

        let cur = self.moveable.grid_position;
        if cur.x <= self.home_position.x.saturating_sub(1)
            && self.moveable.direction == Direction::Left
        {
            self.moveable.change_direction(Direction::Right);
        } else if cur.x >= self.home_position.x.saturating_add(1)
            && self.moveable.direction == Direction::Right
        {
            self.moveable.change_direction(Direction::Left);
        }

        self.moveable.move_moveable(grid, true);
    }

    fn process_house_exit(&mut self, grid: &mut Grid) {
        let cur = self.moveable.grid_position;

        if cur.x < self.door_position.x {
            self.moveable.change_direction(Direction::Right);
        } else if cur.x > self.door_position.x {
            self.moveable.change_direction(Direction::Left);
        } else if cur.y > self.door_position.y {
            self.moveable.change_direction(Direction::Up);
        } else {
            self.house_state = HouseState::Outside;
            self.moveable.change_direction(Direction::Left);
            self.tried_change = true;
            self.moveable.move_moveable(grid, false);
            return;
        }

        self.moveable.move_moveable(grid, true);
    }

    fn process_house_entry(&mut self, grid: &mut Grid, config: &LevelConfig) {
        let cur = self.moveable.grid_position;

        if cur.y < self.home_position.y {
            self.moveable.change_direction(Direction::Down);
        } else if cur.x < self.home_position.x {
            self.moveable.change_direction(Direction::Right);
        } else if cur.x > self.home_position.x {
            self.moveable.change_direction(Direction::Left);
        } else {
            // ARCADE RULE: Eaten eyes revived inside the house exit IMMEDIATELY.
            // They revive in normal (dangerous) Scatter mode, even if Superball is still active on Pac-Man.
            self.movement_mode = MovementMode::Scatter;
            self.moveable.speed = config.ghost_speed;
            self.house_state = HouseState::Leaving;
            return;
        }

        self.moveable.move_moveable(grid, true);
    }

    fn update_outside_movement(
        &mut self,
        pac_position: Point,
        pac_direction: &Direction,
        blinky_pos: Point,
        grid: &mut Grid,
    ) {
        if self.movement_mode == MovementMode::Eaten
            && self.moveable.grid_position.x == self.door_position.x
            && self.moveable.grid_position.y == self.door_position.y
        {
            self.house_state = HouseState::Entering;
            return;
        }

        let old_point = self.moveable.grid_position;
        if self.should_retarget() {
            self.update_path(pac_position, pac_direction, blinky_pos, grid, false);
            self.tried_change = true;
        }
        self.moveable.move_moveable(grid, false);
        if self.moveable.grid_position.x != old_point.x
            || self.moveable.grid_position.y != old_point.y
        {
            self.tried_change = false;
        }
    }

    pub fn set_frightened(&mut self, config: &LevelConfig) {
        // ARCADE RULE: All ghosts (Inside, Leaving, or Outside) become Frightened when Superball is eaten,
        // EXCEPT for ghosts returning as Eaten eyes.
        if self.movement_mode != MovementMode::Eaten {
            self.movement_mode = MovementMode::Frightened;
            self.moveable.speed = config.frightened_ghost_speed;

            // Only reverse direction if actively navigating the open maze
            if self.house_state == HouseState::Outside {
                let rev = self.moveable.direction.opposite();
                self.moveable.change_direction(rev);
            }
        }
    }

    pub fn set_frightened_blinking(&mut self) {
        if self.movement_mode == MovementMode::Frightened {
            self.movement_mode = MovementMode::FrightenedBlinking;
        }
    }

    pub fn stop_frightened(&mut self, config: &LevelConfig) {
        if self.movement_mode == MovementMode::Frightened
            || self.movement_mode == MovementMode::FrightenedBlinking
        {
            self.movement_mode = MovementMode::Chase;
            self.moveable.speed = config.ghost_speed;
        }
    }

    pub fn set_eaten(&mut self, config: &LevelConfig) {
        self.movement_mode = MovementMode::Eaten;
        self.moveable.speed = config.eaten_ghost_speed;
    }

    fn should_retarget(&self) -> bool {
        !self.tried_change
            && (self.moveable.steps < self.moveable.speed * 1.5
                || (self.moveable.grid_position.x == self.moveable.destination.x
                    && self.moveable.grid_position.y == self.moveable.destination.y))
    }

    fn update_path(
        &mut self,
        pac_position: Point,
        pac_direction: &Direction,
        blinky_pos: Point,
        grid: &Grid,
        ignore_intersections: bool,
    ) {
        if !ignore_intersections
            && !(can_go_to(self.moveable.grid_position, &Direction::Up, grid)
                && self.moveable.direction != Direction::Down
                || can_go_to(self.moveable.grid_position, &Direction::Down, grid)
                    && self.moveable.direction != Direction::Up
                || can_go_to(self.moveable.grid_position, &Direction::Left, grid)
                    && self.moveable.direction != Direction::Right
                || can_go_to(self.moveable.grid_position, &Direction::Right, grid)
                    && self.moveable.direction != Direction::Left)
        {
            return;
        }

        let max_x = GRID_WIDTH - 1;
        let max_y = GRID_HEIGHT - 1;

        self.target_cell = match self.movement_mode {
            MovementMode::Scatter => scatter_point(&self.gtype),
            MovementMode::Frightened | MovementMode::FrightenedBlinking => Point {
                x: pac_position.x.saturating_add(2).min(max_x),
                y: pac_position.y.saturating_add(2).min(max_y),
            },
            MovementMode::Eaten => self.door_position,
            MovementMode::Chase => match self.gtype {
                GhostType::Blinky => pac_position,
                GhostType::Pinky => match pac_direction {
                    Direction::Up => Point {
                        x: pac_position.x,
                        y: pac_position.y.saturating_sub(4),
                    },
                    Direction::Down => Point {
                        x: pac_position.x,
                        y: pac_position.y.saturating_add(4).min(max_y),
                    },
                    Direction::Right => Point {
                        x: pac_position.x.saturating_add(4).min(max_x),
                        y: pac_position.y,
                    },
                    Direction::Left => Point {
                        x: pac_position.x.saturating_sub(4),
                        y: pac_position.y,
                    },
                },
                GhostType::Inky => {
                    let temp = match pac_direction {
                        Direction::Up => Point {
                            x: pac_position.x,
                            y: pac_position.y.saturating_sub(2),
                        },
                        Direction::Down => Point {
                            x: pac_position.x,
                            y: pac_position.y.saturating_add(2).min(max_y),
                        },
                        Direction::Right => Point {
                            x: pac_position.x.saturating_add(4).min(max_x),
                            y: pac_position.y,
                        },
                        Direction::Left => Point {
                            x: pac_position.x.saturating_sub(2),
                            y: pac_position.y,
                        },
                    };
                    Point {
                        x: (2 * temp.x as i16 - blinky_pos.x as i16).clamp(0, max_x as i16) as u16,
                        y: (2 * temp.y as i16 - blinky_pos.y as i16).clamp(0, max_y as i16) as u16,
                    }
                }
                GhostType::Clyde => {
                    if distance(self.moveable.grid_position, pac_position) > 64 {
                        pac_position
                    } else {
                        scatter_point(&self.gtype)
                    }
                }
            },
        };

        let possible_directions = [
            Direction::Up,
            Direction::Left,
            Direction::Down,
            Direction::Right,
        ];
        let mut best_direction = self.moveable.direction.opposite();
        let is_frightened = self.is_frightened();

        let mut best_distance = if is_frightened { 0 } else { u32::MAX };

        for &d in possible_directions.iter() {
            if d == self.moveable.direction.opposite() {
                continue;
            }
            if !can_go_to(self.moveable.grid_position, &d, grid) {
                continue;
            }

            if self.moveable.grid_position.x == self.door_position.x
                && self.moveable.grid_position.y == self.door_position.y
                && d == Direction::Down
                && self.movement_mode != MovementMode::Eaten
            {
                continue;
            }

            let next_cell = next_pos(self.moveable.grid_position, &d);
            let next_distance = distance(next_cell.0, self.target_cell);

            if is_frightened {
                if next_distance > best_distance {
                    best_distance = next_distance;
                    best_direction = d;
                }
            } else if next_distance < best_distance {
                best_distance = next_distance;
                best_direction = d;
            }
        }
        self.moveable.change_direction(best_direction);
    }
}

/// Strict Arcade Euclidean Distance Squared: (x1 - x2)^2 + (y1 - y2)^2
#[inline(always)]
fn distance(p1: Point, p2: Point) -> u32 {
    let dx = (p1.x as i32) - (p2.x as i32);
    let dy = (p1.y as i32) - (p2.y as i32);
    (dx * dx + dy * dy) as u32
}

const fn scatter_point(g: &GhostType) -> Point {
    match g {
        GhostType::Blinky => Point {
            x: GRID_WIDTH - 1,
            y: 0,
        },
        GhostType::Pinky => Point { x: 0, y: 0 },
        GhostType::Inky => Point {
            x: GRID_WIDTH - 1,
            y: GRID_HEIGHT - 1,
        },
        GhostType::Clyde => Point {
            x: 0,
            y: GRID_HEIGHT - 1,
        },
    }
}
