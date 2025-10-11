use numworks_utils::eadk::{display::push_rect_uniform, random, timing, Color, Point, Rect};

use crate::{
    game::{Grid, GRID_HEIGHT, GRID_WIDTH, STEPS_PER_CELL},
    moveable::{can_go_to, next_pos, Direction, Moveable},
};

pub enum GhostType {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

enum MovementMode {
    Chase,
    Scatter,
    Frightened,
}

enum GhostEvent {
    LeavingHouse,
    StopBeingFrightened,
    StopBeingEaten,
}

pub struct Ghost {
    pub gtype: GhostType,
    pub moveable: Moveable,
    /// it will try going there, depending on personnality (gtype)
    target_cell: Point,
    movement_mode: MovementMode,
    /// true if inside the ghost house
    pub is_home: bool,
    /// true if leaving the ghost house
    leaving_home: bool,
    /// useful for doing something at this given time
    end_timer: u64,
    /// next event to do (when timer ends)
    next_event: Option<GhostEvent>,
    // need to add something to be sure it changes direction only at middle of intersection
    tried_change: bool,
}

// Depending on personnality, at each intersection, we update the target tile then
// take the best direction to respect the rules (minimizing distance for exemple)
// some details : excluding direct reversal unless mode change ;
// just take euclidean distance from next possible tile to find best one (don't care about path) ;
// ties broken by priority order : Up > Left > Down > Right

impl Ghost {
    pub fn new(start: Point, gtype: GhostType) -> Self {
        Self {
            gtype,
            moveable: Moveable::new(
                start,
                Point {
                    x: start.x - 1,
                    y: start.y,
                },
                Direction::Left,
                0.5,
            ),
            leaving_home: false,
            is_home: true,
            target_cell: Point { x: 0, y: 0 },
            movement_mode: MovementMode::Chase,
            end_timer: timing::millis() + 1000,
            next_event: Some(GhostEvent::LeavingHouse),
            tried_change: false,
        }
    }

    pub fn update(
        &mut self,
        pac_position: Point,
        pac_direction: &Direction,
        blinky_pos: Point,
        grid: &mut Grid,
    ) {
        // handle events
        if self.end_timer != 0 && timing::millis() >= self.end_timer {
            match self.next_event {
                Some(GhostEvent::LeavingHouse) => {
                    self.leaving_home = true;
                    self.end_timer = 0;
                    self.next_event = None;
                }
                Some(GhostEvent::StopBeingFrightened) => {
                    self.movement_mode = MovementMode::Chase;
                    self.end_timer = 0;
                    self.next_event = None;
                }
                Some(GhostEvent::StopBeingEaten) => {
                    self.movement_mode = MovementMode::Chase;
                    self.end_timer = 0;
                    self.next_event = None;
                }
                None => (),
            }
        }

        if self.is_home && !self.leaving_home {
            // TODO : move back and forth ?
            return;
        }

        if self.leaving_home {
            // go to x = 13.5 then up (ignore y as it will be good...)
            // Move towards the center x (13 or 14), then up to exit the ghost house
            let target_x = if self.moveable.grid_position.x >= 14 {
                14
            } else {
                13
            };
            if self.moveable.grid_position.x != target_x {
                if self.moveable.grid_position.x < target_x {
                    self.moveable.change_direction(Direction::Right);
                } else {
                    self.moveable.change_direction(Direction::Left);
                }
            } else {
                self.moveable.change_direction(Direction::Up);
            }
            let old_pos = self.moveable.grid_position;
            let old_steps = self.moveable.steps;
            self.moveable
                .move_moveable(grid, self.moveable.grid_position.y > 12);

            // Check if ghost has exited the house (y <= 12 is usually outside)
            if old_pos.y == self.moveable.grid_position.y
                && old_pos.x == self.moveable.grid_position.x
                && old_steps == self.moveable.steps
                && self.moveable.grid_position.y < 12
            {
                self.leaving_home = false;
                self.is_home = false;
                self.update_path(pac_position, pac_direction, blinky_pos, grid, false);
                self.tried_change = false;
            // Do not return here; allow the ghost to continue moving after leaving home
            } else {
                return;
            }
        }

        const TURN_FACTOR_EARLY: f32 = 2.0;
        let old_point = self.moveable.grid_position;
        // only update path once per cell
        if !self.tried_change
            && (self.moveable.steps + self.moveable.speed * TURN_FACTOR_EARLY
                >= STEPS_PER_CELL / 2.0 - 2.0
                || (self.moveable.grid_position.x == self.moveable.destination.x
                    && self.moveable.grid_position.y == self.moveable.destination.y))
        {
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

    fn update_path(
        &mut self,
        pac_position: Point,
        pac_direction: &Direction,
        blinky_pos: Point,
        grid: &Grid,
        ignore_intersections: bool,
    ) {
        // updating ONlY at intersections

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

        // if corner : will be only one choice (can't go reverse if no mode change)
        self.target_cell = match self.gtype {
            // aggressive
            GhostType::Blinky => pac_position,
            // ambusher
            GhostType::Pinky => match pac_direction {
                Direction::Up => Point {
                    x: pac_position.x,
                    y: pac_position.y.saturating_sub(4),
                },
                Direction::Down => Point {
                    x: pac_position.x,
                    y: pac_position.y.saturating_add(4),
                },
                Direction::Right => Point {
                    x: pac_position.x.saturating_add(4),
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
                        y: pac_position.y.saturating_add(2),
                    },
                    Direction::Right => Point {
                        x: pac_position.x.saturating_add(2),
                        y: pac_position.y,
                    },
                    Direction::Left => Point {
                        x: pac_position.x.saturating_sub(2),
                        y: pac_position.y,
                    },
                };
                Point {
                    x: (blinky_pos.x as i16 + 2 * temp.x as i16 - blinky_pos.x as i16)
                        .clamp(0, GRID_WIDTH as i16) as u16,
                    y: (blinky_pos.y as i16 + 2 * temp.y as i16 - blinky_pos.y as i16)
                        .clamp(0, GRID_HEIGHT as i16) as u16,
                }
            }
            // flee if close
            GhostType::Clyde => {
                if distance(self.moveable.grid_position, pac_position) > 8 {
                    pac_position
                } else {
                    scatter_point(&self.gtype)
                }
            }
        };
        // todo : try all directions to find best choice
        let possible_directions = [
            Direction::Up,
            Direction::Left,
            Direction::Down,
            Direction::Right,
        ];
        let mut best_direction = self.moveable.direction.opposite(); // in case no other choice
        let mut best_distance = u16::MAX;
        for &d in possible_directions.iter() {
            if d == self.moveable.direction.opposite() {
                continue; // can't go back (FOR NOW)
            }
            if !can_go_to(self.moveable.grid_position, &d, grid) {
                continue; // if can't go there, don't care
            }
            let next_cell = next_pos(self.moveable.grid_position, &d);
            if distance(next_cell.0, self.target_cell) < best_distance {
                best_distance = distance(next_cell.0, self.target_cell);
                best_direction = d;
            }
        }
        self.moveable.change_direction(best_direction);
    }
}

#[inline(always)]
const fn distance(p1: Point, p2: Point) -> u16 {
    p1.x.abs_diff(p2.x) + p1.y.abs_diff(p2.y)
}

const fn scatter_point(g: &GhostType) -> Point {
    match g {
        GhostType::Blinky => Point {
            x: GRID_WIDTH,
            y: 0,
        },
        GhostType::Pinky => Point { x: 0, y: 0 },
        GhostType::Inky => Point {
            x: GRID_WIDTH,
            y: GRID_HEIGHT,
        },
        GhostType::Clyde => Point {
            x: 0,
            y: GRID_HEIGHT,
        },
    }
}
