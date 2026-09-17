use num_engine::{graphics::Animation, world::World};
use numworks_utils::{
    eadk::timing,
    numbers::{ceil, floor},
    utils::randint,
};

use crate::{
    game::Grid,
    levels::LevelConfig,
    moveable::{can_go_to, distance_sq, Direction},
    pac_ui::*,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GhostType {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

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
    Inside,
    Leaving,
    Outside,
    Entering,
}

pub struct Ghost {
    pub entity_id: usize,
    pub gtype: GhostType,
    pub dir: Direction,
    pub speed: f32,
    pub movement_mode: MovementMode,
    pub house_state: HouseState,
    pub last_turn_node: (i16, i16),
    release_timer: u64,
    wave_timer: u64,
    wave_step: u8,
}

impl Ghost {
    pub fn new(gtype: GhostType, config: &LevelConfig) -> Self {
        let now = timing::millis();
        let (house_state, release_timer, dir) = match gtype {
            GhostType::Blinky => (HouseState::Outside, 0, Direction::Left),
            GhostType::Pinky => (HouseState::Leaving, 0, Direction::Up),
            GhostType::Inky => (
                HouseState::Inside,
                now + config.inky_release_ms,
                Direction::Right,
            ),
            GhostType::Clyde => (
                HouseState::Inside,
                now + config.clyde_release_ms,
                Direction::Left,
            ),
        };

        Self {
            entity_id: 0,
            gtype,
            dir,
            speed: config.ghost_speed,
            movement_mode: MovementMode::Scatter,
            house_state,
            last_turn_node: (-1, -1),
            release_timer,
            wave_timer: now + config.wave_timings.scatter_1,
            wave_step: 0,
        }
    }

    pub fn spawn_coords(&self) -> [f32; 2] {
        match self.gtype {
            GhostType::Blinky => [100.0, 84.0], // Tile (13, 11)
            GhostType::Pinky => [100.0, 104.0], // Tile (13, 13)
            GhostType::Inky => [92.0, 104.0],   // Tile (12, 13)
            GhostType::Clyde => [116.0, 104.0], // Tile (15, 13)
        }
    }

    pub fn reset_position<
        const TS: usize,
        const CA: usize,
        const EC: usize,
        const MC: usize,
        const PE: usize,
        const RC: usize,
        const PC: usize,
        const PSC: usize,
    >(
        &mut self,
        world: &mut World<'_, TS, CA, EC, MC, PE, RC, PC, PSC>,
        config: &LevelConfig,
    ) {
        let now = timing::millis();
        let (house_state, release_timer, dir) = match self.gtype {
            GhostType::Blinky => (HouseState::Outside, 0, Direction::Left),
            GhostType::Pinky => (HouseState::Leaving, 0, Direction::Up),
            GhostType::Inky => (
                HouseState::Inside,
                now + config.inky_release_ms,
                Direction::Right,
            ),
            GhostType::Clyde => (
                HouseState::Inside,
                now + config.clyde_release_ms,
                Direction::Left,
            ),
        };

        self.dir = dir;
        self.speed = config.ghost_speed;
        self.movement_mode = MovementMode::Scatter;
        self.house_state = house_state;
        self.last_turn_node = (-1, -1);
        self.release_timer = release_timer;
        self.wave_timer = now + config.wave_timings.scatter_1;
        self.wave_step = 0;

        let [sx, sy] = self.spawn_coords();
        let ent = &mut world[self.entity_id];
        ent.set_pos(sx, sy);
        ent.set_vx(0.0);
        ent.set_vy(0.0);
        ent.set_animation(self.current_animation());
        ent.active = true;
    }

    pub fn prepare_kinetics<
        const TS: usize,
        const CA: usize,
        const EC: usize,
        const MC: usize,
        const PE: usize,
        const RC: usize,
        const PC: usize,
        const PSC: usize,
    >(
        &mut self,
        world: &mut World<'_, TS, CA, EC, MC, PE, RC, PC, PSC>,
        config: &LevelConfig,
    ) {
        self.update_wave_timer(config);
        self.recalculate_speed(config);

        let (dx, dy) = self.dir.to_vector();
        let eff_speed = match self.house_state {
            HouseState::Inside => self.speed * 0.45,
            HouseState::Leaving => 0.8,
            HouseState::Entering => 1.6,
            HouseState::Outside => self.speed,
        };

        let ent = &mut world[self.entity_id];
        ent.set_vx(dx as f32 * eff_speed);
        ent.set_vy(dy as f32 * eff_speed);
        ent.set_animation(self.current_animation());
    }

    pub fn post_physics<
        const TS: usize,
        const CA: usize,
        const EC: usize,
        const MC: usize,
        const PE: usize,
        const RC: usize,
        const PC: usize,
        const PSC: usize,
    >(
        &mut self,
        world: &mut World<'_, TS, CA, EC, MC, PE, RC, PC, PSC>,
        pac_tx: i16,
        pac_ty: i16,
        pac_dir: Direction,
        blinky_tx: i16,
        blinky_ty: i16,
        grid: &Grid,
    ) {
        let mut x = world[self.entity_id].x();
        let mut y = world[self.entity_id].y();

        match self.house_state {
            HouseState::Inside => {
                if self.release_timer != 0 && timing::millis() >= self.release_timer {
                    self.house_state = HouseState::Leaving;
                } else {
                    y = 104.0;
                    if x <= 92.0 {
                        x = 92.0;
                        self.dir = Direction::Right;
                    } else if x >= 116.0 {
                        x = 116.0;
                        self.dir = Direction::Left;
                    }
                }
            }
            HouseState::Leaving => {
                if (x - 100.0).abs() > 1.0 {
                    self.dir = if x < 100.0 {
                        Direction::Right
                    } else {
                        Direction::Left
                    };
                } else {
                    x = 100.0;
                    self.dir = Direction::Up;
                    if y <= 84.0 {
                        y = 84.0;
                        self.dir = Direction::Left;
                        self.last_turn_node = (13, 11);
                        self.house_state = HouseState::Outside;
                    }
                }
            }
            HouseState::Entering => {
                x = 100.0;
                self.dir = Direction::Down;
                if y >= 104.0 {
                    y = 104.0;
                    self.movement_mode = MovementMode::Scatter;
                    self.last_turn_node = (-1, -1);
                    self.house_state = HouseState::Leaving;
                }
            }
            HouseState::Outside => {
                let max_x = (crate::game::GRID_WIDTH * 8) as f32; // 224.0

                // Tunnel row detection: rows 13 and 14 center around 100.0 and 108.0
                let in_tunnel_corridor = (y - 108.0).abs() < 8.0 || (y - 100.0).abs() < 8.0;

                // 1. Seamless Tunnel Wrapping
                if in_tunnel_corridor {
                    if x < -12.0 {
                        x = max_x + 4.0;
                        y = 108.0; // Strict tunnel centerline snap
                        self.last_turn_node = (-1, -1);
                    } else if x > max_x + 12.0 {
                        x = -4.0;
                        y = 108.0; // Strict tunnel centerline snap
                        self.last_turn_node = (-1, -1);
                    }
                }

                // 2. Off-screen tunnel traversal: lock horizontally, never turn vertically
                if in_tunnel_corridor && (x < 4.0 || x > max_x - 12.0) {
                    y = 108.0;
                    if self.dir != Direction::Left && self.dir != Direction::Right {
                        self.dir = if x < max_x / 2.0 {
                            Direction::Left
                        } else {
                            Direction::Right
                        };
                    }
                } else {
                    // Normal in-maze navigation
                    let ty = (((y + 4.0) / 8.0) + 0.5) as i16;
                    let target_node = match self.dir {
                        Direction::Right => {
                            let tx = floor((x + 4.0) / 8.0) as i16;
                            (tx, ty)
                        }
                        Direction::Left => {
                            let tx = ceil((x + 4.0) / 8.0) as i16;
                            (tx, ty)
                        }
                        Direction::Down => {
                            let tx = (((x + 4.0) / 8.0) + 0.5) as i16;
                            let nty = floor((y + 4.0) / 8.0) as i16;
                            (tx, nty)
                        }
                        Direction::Up => {
                            let tx = (((x + 4.0) / 8.0) + 0.5) as i16;
                            let nty = ceil((y + 4.0) / 8.0) as i16;
                            (tx, nty)
                        }
                    };

                    let cx = (target_node.0 * 8 - 4) as f32;
                    let cy = (target_node.1 * 8 - 4) as f32;

                    let passed_center = match self.dir {
                        Direction::Right => x >= cx,
                        Direction::Left => x <= cx,
                        Direction::Down => y >= cy,
                        Direction::Up => y <= cy,
                    };

                    // Dead eyes returning to house door
                    if self.movement_mode == MovementMode::Eaten
                        && target_node.1 == 11
                        && (target_node.0 == 13 || target_node.0 == 14)
                        && passed_center
                    {
                        x = 100.0;
                        y = 84.0;
                        self.house_state = HouseState::Entering;
                    } else if passed_center && target_node != self.last_turn_node {
                        let overshoot = match self.dir {
                            Direction::Right => x - cx,
                            Direction::Left => cx - x,
                            Direction::Down => y - cy,
                            Direction::Up => cy - y,
                        }
                        .max(0.0);

                        self.last_turn_node = target_node;
                        let (tx, ty_node) = target_node;

                        let (t_target_x, t_target_y) =
                            self.calculate_target(pac_tx, pac_ty, pac_dir, blinky_tx, blinky_ty);
                        let prev_dir = self.dir;
                        self.choose_direction(tx, ty_node, t_target_x, t_target_y, grid);

                        x = cx;
                        y = cy;
                        if self.dir != prev_dir {
                            let (ndx, ndy) = self.dir.to_vector();
                            x += ndx as f32 * overshoot;
                            y += ndy as f32 * overshoot;
                        }
                    } else {
                        match self.dir {
                            Direction::Left | Direction::Right => y = cy,
                            Direction::Up | Direction::Down => x = cx,
                        }
                    }
                }
            }
        }

        let ent = &mut world[self.entity_id];
        ent.set_pos(x, y);
    }

    fn choose_direction(&mut self, tx: i16, ty: i16, target_x: i16, target_y: i16, grid: &Grid) {
        let dirs = [
            Direction::Up,
            Direction::Left,
            Direction::Down,
            Direction::Right,
        ];
        let is_fright = matches!(
            self.movement_mode,
            MovementMode::Frightened | MovementMode::FrightenedBlinking
        );
        let is_eaten = self.movement_mode == MovementMode::Eaten;

        let mut valid_dirs = [Direction::Up; 4];
        let mut num_valid = 0;

        for &d in &dirs {
            if d == self.dir.opposite() {
                continue;
            }
            if !can_go_to(tx, ty, d, grid) {
                continue;
            }
            if !is_eaten && d == Direction::Down && ty == 11 && (tx == 13 || tx == 14) {
                continue;
            }
            valid_dirs[num_valid] = d;
            num_valid += 1;
        }

        if num_valid > 0 {
            if is_fright {
                let pick = randint(0, num_valid as u32) as usize;
                self.dir = valid_dirs[pick];
            } else {
                let mut best_dir = valid_dirs[0];
                let mut best_dist = u32::MAX;

                for &d in &valid_dirs[..num_valid] {
                    let (ndx, ndy) = d.to_vector();
                    let dist = distance_sq(tx + ndx, ty + ndy, target_x, target_y);
                    if dist < best_dist {
                        best_dist = dist;
                        best_dir = d;
                    }
                }

                self.dir = best_dir;
            }
        } else if can_go_to(tx, ty, self.dir.opposite(), grid) {
            self.dir = self.dir.opposite();
        }
    }

    fn calculate_target(
        &self,
        pac_tx: i16,
        pac_ty: i16,
        pac_dir: Direction,
        blinky_tx: i16,
        blinky_ty: i16,
    ) -> (i16, i16) {
        match self.movement_mode {
            MovementMode::Scatter => match self.gtype {
                GhostType::Blinky => (27, 0),
                GhostType::Pinky => (0, 0),
                GhostType::Inky => (27, 29),
                GhostType::Clyde => (0, 29),
            },
            MovementMode::Eaten => (13, 11),
            MovementMode::Frightened | MovementMode::FrightenedBlinking => {
                (randint(0, 27) as i16, randint(0, 29) as i16)
            }
            MovementMode::Chase => match self.gtype {
                GhostType::Blinky => (pac_tx, pac_ty),
                GhostType::Pinky => {
                    let (dx, dy) = pac_dir.to_vector();
                    (pac_tx + dx * 4, pac_ty + dy * 4)
                }
                GhostType::Inky => {
                    let (dx, dy) = pac_dir.to_vector();
                    let ix = pac_tx + dx * 2;
                    let iy = pac_ty + dy * 2;
                    (2 * ix - blinky_tx, 2 * iy - blinky_ty)
                }
                GhostType::Clyde => {
                    let d = distance_sq(
                        self.spawn_coords()[0] as i16 / 8,
                        self.spawn_coords()[1] as i16 / 8,
                        pac_tx,
                        pac_ty,
                    );
                    if d > 64 {
                        (pac_tx, pac_ty)
                    } else {
                        (0, 29)
                    }
                }
            },
        }
    }

    pub fn set_frightened(&mut self, config: &LevelConfig) {
        if self.movement_mode != MovementMode::Eaten {
            if config.frightened_duration_ms > 0 {
                self.movement_mode = MovementMode::Frightened;
                self.speed = config.frightened_ghost_speed;
            }

            if self.house_state == HouseState::Outside {
                self.dir = self.dir.opposite();
                self.last_turn_node = (-1, -1);
            }
        }
    }

    pub fn set_frightened_blinking(&mut self) {
        if self.movement_mode == MovementMode::Frightened {
            self.movement_mode = MovementMode::FrightenedBlinking;
        }
    }

    pub fn stop_frightened(&mut self, config: &LevelConfig) {
        if matches!(
            self.movement_mode,
            MovementMode::Frightened | MovementMode::FrightenedBlinking
        ) {
            self.movement_mode = MovementMode::Chase;
            self.speed = config.ghost_speed;
            self.last_turn_node = (-1, -1);
        }
    }

    pub fn set_eaten(&mut self, config: &LevelConfig) {
        self.movement_mode = MovementMode::Eaten;
        self.speed = config.eaten_ghost_speed;
        self.last_turn_node = (-1, -1);
    }

    fn recalculate_speed(&mut self, config: &LevelConfig) {
        self.speed = match self.movement_mode {
            MovementMode::Eaten => config.eaten_ghost_speed,
            MovementMode::Frightened | MovementMode::FrightenedBlinking => {
                config.frightened_ghost_speed
            }
            MovementMode::Chase | MovementMode::Scatter => config.ghost_speed,
        };
    }

    fn update_wave_timer(&mut self, config: &LevelConfig) {
        if matches!(
            self.movement_mode,
            MovementMode::Frightened | MovementMode::FrightenedBlinking | MovementMode::Eaten
        ) {
            return;
        }
        let now = timing::millis();
        if self.wave_timer == 0 || now < self.wave_timer {
            return;
        }
        let t = &config.wave_timings;
        match self.wave_step {
            0 => {
                self.wave_step = 1;
                self.movement_mode = MovementMode::Chase;
                self.wave_timer = now + t.chase_1;
            }
            1 => {
                self.wave_step = 2;
                self.movement_mode = MovementMode::Scatter;
                self.wave_timer = now + t.scatter_2;
            }
            2 => {
                self.wave_step = 3;
                self.movement_mode = MovementMode::Chase;
                self.wave_timer = now + t.chase_2;
            }
            3 => {
                self.wave_step = 4;
                self.movement_mode = MovementMode::Scatter;
                self.wave_timer = now + t.scatter_3;
            }
            4 => {
                self.wave_step = 5;
                self.movement_mode = MovementMode::Chase;
                self.wave_timer = now + t.chase_3;
            }
            5 => {
                self.wave_step = 6;
                self.movement_mode = MovementMode::Scatter;
                self.wave_timer = now + t.scatter_4;
            }
            6 => {
                self.wave_step = 7;
                self.movement_mode = MovementMode::Chase;
                self.wave_timer = 0;
            }
            _ => {}
        }
    }

    pub fn current_animation(&self) -> &'static Animation<'static> {
        match self.movement_mode {
            MovementMode::Frightened => &ANIM_GHOST_FRIGHTENED,
            MovementMode::FrightenedBlinking => &ANIM_GHOST_FLASHING,
            MovementMode::Eaten => match self.dir {
                Direction::Up => &ANIM_EYES_UP,
                Direction::Down => &ANIM_EYES_DOWN,
                Direction::Left => &ANIM_EYES_LEFT,
                Direction::Right => &ANIM_EYES_RIGHT,
            },
            MovementMode::Chase | MovementMode::Scatter => match self.gtype {
                GhostType::Blinky => match self.dir {
                    Direction::Right => &ANIM_BLINKY_RIGHT,
                    Direction::Left => &ANIM_BLINKY_LEFT,
                    Direction::Up => &ANIM_BLINKY_UP,
                    Direction::Down => &ANIM_BLINKY_DOWN,
                },
                GhostType::Pinky => match self.dir {
                    Direction::Right => &ANIM_PINKY_RIGHT,
                    Direction::Left => &ANIM_PINKY_LEFT,
                    Direction::Up => &ANIM_PINKY_UP,
                    Direction::Down => &ANIM_PINKY_DOWN,
                },
                GhostType::Inky => match self.dir {
                    Direction::Right => &ANIM_INKY_RIGHT,
                    Direction::Left => &ANIM_INKY_LEFT,
                    Direction::Up => &ANIM_INKY_UP,
                    Direction::Down => &ANIM_INKY_DOWN,
                },
                GhostType::Clyde => match self.dir {
                    Direction::Right => &ANIM_CLYDE_RIGHT,
                    Direction::Left => &ANIM_CLYDE_LEFT,
                    Direction::Up => &ANIM_CLYDE_UP,
                    Direction::Down => &ANIM_CLYDE_DOWN,
                },
            },
        }
    }
}
