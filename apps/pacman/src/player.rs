use num_engine::world::World;
use numworks_utils::eadk::{key, keyboard, timing};

use crate::{
    game::{Grid, Space, GRID_WIDTH},
    levels::LevelConfig,
    moveable::{can_go_to, Direction},
    pac_ui::{ANIM_PAC_DOWN, ANIM_PAC_IDLE, ANIM_PAC_LEFT, ANIM_PAC_RIGHT, ANIM_PAC_UP},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SuperballEvent {
    None,
    BlinkingStarted,
    Expired,
}

pub struct Player {
    pub entity_id: usize,
    pub dir: Direction,
    pub desired_dir: Direction,
    pub speed: f32,
    pub moving: bool,
    pub eat_stutter_frames: u8,
    pub superball_active: bool,
    superball_until: u64,
    blink_at: u64,
    blinking_triggered: bool,
    pub dots_eaten: u16,
    pub buffer_window_frames: u8,
    desired_dir_timer: u8,
    pub pause_frames: u8,
}

impl Player {
    pub fn new(config: &LevelConfig) -> Self {
        Self {
            entity_id: 0,
            dir: Direction::Right,
            desired_dir: Direction::Right,
            speed: config.pac_speed,
            moving: false,
            eat_stutter_frames: 0,
            superball_active: false,
            superball_until: 0,
            blink_at: 0,
            blinking_triggered: false,
            dots_eaten: 0,
            buffer_window_frames: 15,
            desired_dir_timer: 0,
            pause_frames: 0,
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
        self.dir = Direction::Right;
        self.desired_dir = Direction::Right;
        self.speed = config.pac_speed;
        self.moving = false;
        self.eat_stutter_frames = 0;
        self.superball_active = false;
        self.superball_until = 0;
        self.blink_at = 0;
        self.blinking_triggered = false;
        self.desired_dir_timer = 0;
        self.pause_frames = 0;

        let ent = &mut world[self.entity_id];
        ent.set_pos(100.0, 172.0); // Tile (13, 22)
        ent.set_vx(0.0);
        ent.set_vy(0.0);
        ent.set_animation(&ANIM_PAC_IDLE);
        ent.active = true;
    }

    pub fn read_input(&mut self) {
        let scan = keyboard::scan();
        let pressed_dir = if scan.key_down(key::UP) {
            Some(Direction::Up)
        } else if scan.key_down(key::DOWN) {
            Some(Direction::Down)
        } else if scan.key_down(key::RIGHT) {
            Some(Direction::Right)
        } else if scan.key_down(key::LEFT) {
            Some(Direction::Left)
        } else {
            None
        };

        if let Some(d) = pressed_dir {
            self.desired_dir = d;
            self.desired_dir_timer = self.buffer_window_frames;
        } else if self.buffer_window_frames != 255 && self.desired_dir_timer > 0 {
            self.desired_dir_timer -= 1;
            if self.desired_dir_timer == 0 {
                self.desired_dir = self.dir;
            }
        }
    }

    /// Step 1: Assign velocity to the engine entity before `world.update`.
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
        if self.pause_frames > 0 {
            self.pause_frames -= 1;
            let ent = &mut world[self.entity_id];
            ent.set_vx(0.0);
            ent.set_vy(0.0);
            return;
        }

        if self.desired_dir == self.dir.opposite() {
            self.dir = self.desired_dir;
            self.moving = true;
        }

        let ent = &mut world[self.entity_id];
        if self.moving {
            let base_spd = if self.eat_stutter_frames > 0 {
                self.eat_stutter_frames -= 1;
                config.pac_dot_speed
            } else {
                config.pac_speed
            };

            let eff_spd = if self.superball_active {
                base_spd * 1.15
            } else {
                base_spd
            };

            self.speed = eff_spd;

            let (dx, dy) = self.dir.to_vector();
            ent.set_vx(dx as f32 * eff_spd);
            ent.set_vy(dy as f32 * eff_spd);
            ent.set_animation(match self.dir {
                Direction::Right => &ANIM_PAC_RIGHT,
                Direction::Left => &ANIM_PAC_LEFT,
                Direction::Up => &ANIM_PAC_UP,
                Direction::Down => &ANIM_PAC_DOWN,
            });
        } else {
            ent.set_vx(0.0);
            ent.set_vy(0.0);
            ent.set_animation(&ANIM_PAC_IDLE);
        }
    }

    /// Step 2: Constrain post-physics corridor constraints and handle item eating.
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
        grid: &mut Grid,
        config: &LevelConfig,
    ) -> (Option<(usize, usize, Space)>, SuperballEvent) {
        let event = self.handle_superball_timing(config);

        let mut x = world[self.entity_id].x();
        let mut y = world[self.entity_id].y();

        // Warp tunnel wrapping
        if x < -8.0 {
            x = (GRID_WIDTH * 8) as f32;
        } else if x > (GRID_WIDTH * 8) as f32 {
            x = -8.0;
        }

        let tx = (((x + 4.0) / 8.0) + 0.5) as i16;
        let ty = (((y + 4.0) / 8.0) + 0.5) as i16;
        let cx = (tx * 8 - 4) as f32;
        let cy = (ty * 8 - 4) as f32;

        // Lock perpendicular axis to corridor center without stalling the moving axis
        match self.dir {
            Direction::Left | Direction::Right => y = cy,
            Direction::Up | Direction::Down => x = cx,
        }

        // Corner turning and wall obstruction check at intersection centers
        let dist = match self.dir {
            Direction::Left | Direction::Right => (x - cx).abs(),
            Direction::Up | Direction::Down => (y - cy).abs(),
        };

        if dist <= self.speed.max(1.0) {
            if self.desired_dir != self.dir && can_go_to(tx, ty, self.desired_dir, grid) {
                self.dir = self.desired_dir;
                x = cx;
                y = cy;
                self.moving = true;
            } else if !can_go_to(tx, ty, self.dir, grid) {
                x = cx;
                y = cy;
                self.moving = false;
            } else {
                self.moving = true;
            }
        }

        let ent = &mut world[self.entity_id];
        ent.set_pos(x, y);

        // Dot & Item consumption
        let mut eaten = None;

        if tx >= 0 && tx < GRID_WIDTH as i16 && ty >= 0 && ty < 30 {
            let idx = ty as usize * GRID_WIDTH + tx as usize;
            match grid[idx] {
                Space::Point => {
                    grid[idx] = Space::Empty;
                    self.dots_eaten += 1;
                    self.eat_stutter_frames = 2;
                    eaten = Some((tx as usize, ty as usize, Space::Point));
                }
                Space::Superball => {
                    grid[idx] = Space::Empty;
                    self.activate_superball(config);
                    self.pause_frames = 1;
                    self.eat_stutter_frames = 2;
                    eaten = Some((tx as usize, ty as usize, Space::Superball));
                }
                Space::Fruit => {
                    grid[idx] = Space::Empty;
                    eaten = Some((tx as usize, ty as usize, Space::Fruit));
                }
                _ => {}
            }
        }

        (eaten, event)
    }

    pub fn activate_superball(&mut self, config: &LevelConfig) {
        if config.frightened_duration_ms == 0 {
            return;
        }
        let now = timing::millis();
        self.superball_active = true;
        self.superball_until = now + config.frightened_duration_ms;
        self.blink_at = self.superball_until.saturating_sub(config.blink_window_ms);
        self.blinking_triggered = false;
        self.speed = config.pac_speed * 1.15;
    }

    fn handle_superball_timing(&mut self, config: &LevelConfig) -> SuperballEvent {
        if !self.superball_active || self.superball_until == 0 {
            return SuperballEvent::None;
        }
        let now = timing::millis();
        if now >= self.superball_until {
            self.superball_active = false;
            self.superball_until = 0;
            self.speed = config.pac_speed;
            SuperballEvent::Expired
        } else if self.blink_at != 0 && now >= self.blink_at && !self.blinking_triggered {
            self.blinking_triggered = true;
            SuperballEvent::BlinkingStarted
        } else {
            SuperballEvent::None
        }
    }
}
