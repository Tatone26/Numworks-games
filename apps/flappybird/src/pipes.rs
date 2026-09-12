use heapless::Vec;
use num_engine::{hitbox, physics::Body, spawn_sprite, world::Entity};
use numworks_utils::utils::randint;

use crate::{
    events::EventManager,
    flappy_ui::{ANIM_PIPE_LIP_BOT, ANIM_PIPE_LIP_TOP, ANIM_PIPE_SHAFT, TILESET_TILE_SIZE},
    game::{GameWorld, GROUND_Y, MAX_PIPES_ON_SCREEN, TILE_SIZE, VIEW_SCREEN_W, WINDOW_ROWS},
};

const SHAFT_HEIGHT_PX: i16 = (WINDOW_ROWS * TILE_SIZE) as i16;
pub const PIPE_WIDTH_PX: u16 = TILESET_TILE_SIZE * 2;
pub const DEFAULT_GAP_SIZE: f32 = 68.0;

const MIN_GAP_Y: i16 = 14;
const MAX_GAP_MARGIN_FROM_GROUND: i16 = 14;
const COLLAR_OFFSET_X: i16 = -(TILESET_TILE_SIZE as i16);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OscillationMode {
    Off,
    Rarely,
    Sometimes,
    Often,
    AlwaysSlow,
    AlwaysFast,
    EventsOnly,
}

impl OscillationMode {
    pub fn from_u32(val: u32) -> Self {
        match val {
            1 => Self::Rarely,
            2 => Self::Sometimes,
            3 => Self::Often,
            4 => Self::AlwaysSlow,
            5 => Self::AlwaysFast,
            6 => Self::EventsOnly,
            _ => Self::Off,
        }
    }
}

pub struct PipePair {
    pub entity_id: usize,
    pub base_gap_y: i16,
    pub current_gap: f32,
    pub target_gap: f32,
    pub scored: bool,
    pub active: bool,
    pub min_y: f32,
    pub max_y: f32,
    pub baseline_osc: bool,
    pub is_dense: bool,
}

impl PipePair {
    pub fn spawn(speed: f32, world: &mut GameWorld) -> Self {
        let spawn_x = VIEW_SCREEN_W as f32 + 100.0;
        let gap_y = 70;

        let body = Body::new(spawn_x, 0.0).with_velocity(-speed, 0.0);

        let top_shaft = spawn_sprite!(&ANIM_PIPE_SHAFT, [0, 0], 10)
            .with_offset([0, (gap_y - 20) - SHAFT_HEIGHT_PX]);
        let top_lip = spawn_sprite!(&ANIM_PIPE_LIP_TOP, [0, 0], 11)
            .with_offset([COLLAR_OFFSET_X, gap_y - 20]);
        let bot_lip = spawn_sprite!(&ANIM_PIPE_LIP_BOT, [0, 0], 11)
            .with_offset([COLLAR_OFFSET_X, gap_y + DEFAULT_GAP_SIZE as i16]);
        let bot_shaft = spawn_sprite!(&ANIM_PIPE_SHAFT, [0, 0], 10)
            .with_offset([0, gap_y + DEFAULT_GAP_SIZE as i16 + 20]);

        let top_box = hitbox!(0, 0, PIPE_WIDTH_PX, gap_y.max(0) as u16);
        let bot_y = gap_y + DEFAULT_GAP_SIZE as i16;
        let bot_box = hitbox!(0, bot_y, PIPE_WIDTH_PX, (GROUND_Y - bot_y).max(0) as u16);

        let mut entity = Entity::new()
            .with_body(body)
            .with_sprite(top_shaft)
            .with_sprite(top_lip)
            .with_sprite(bot_lip)
            .with_sprite(bot_shaft)
            .with_hitbox(top_box)
            .with_hitbox(bot_box);

        entity.set_active(false);
        let entity_id = world.spawn(entity).expect("World entity capacity exceeded");

        Self {
            entity_id,
            base_gap_y: gap_y,
            current_gap: DEFAULT_GAP_SIZE,
            target_gap: DEFAULT_GAP_SIZE,
            scored: false,
            active: false,
            min_y: 0.0,
            max_y: 0.0,
            baseline_osc: false,
            is_dense: false,
        }
    }

    pub fn set_active(&mut self, world: &mut GameWorld, active: bool) {
        self.active = active;
        world[self.entity_id].set_active(active);
    }

    fn apply_layout(&self, world: &mut GameWorld) {
        let gy = self.base_gap_y;
        let gap_int = self.current_gap as i16;
        let ent = &mut world[self.entity_id];

        ent.sprites[0].offset = [0, (gy - 20) - SHAFT_HEIGHT_PX];
        ent.sprites[1].offset = [COLLAR_OFFSET_X, gy - 20];
        ent.sprites[2].offset = [COLLAR_OFFSET_X, gy + gap_int];
        ent.sprites[3].offset = [0, gy + gap_int + 20];

        ent.hitboxes[0].offset_y = 0;
        ent.hitboxes[0].height = gy.max(0) as u16;

        let bot_y = gy + gap_int;
        ent.hitboxes[1].offset_y = bot_y;
        ent.hitboxes[1].height = (GROUND_Y - bot_y).max(0) as u16;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn warp_to(
        &mut self,
        world: &mut GameWorld,
        target_x: f32,
        mode: OscillationMode,
        events: &EventManager,
        prev_gap_y: i16,
        spacing: i16,
        speed: f32,
        jump_power: f32,
        gap_difficulty: u8,
        wave_dir: &mut i8,
        wave_steps_left: &mut u8,
    ) {
        self.target_gap = events.target_gap_size();
        self.current_gap = self.target_gap;
        self.base_gap_y = PipePool::safe_gap(
            prev_gap_y,
            self.current_gap,
            spacing,
            jump_power,
            gap_difficulty,
            events,
            wave_dir,
            wave_steps_left,
        );
        self.scored = false;
        self.is_dense = events.is_dense();

        self.apply_layout(world);

        let ent = &mut world[self.entity_id];
        ent.set_pos(target_x, 0.0);
        ent.set_vx(-speed);

        let (should_move, fast) = match mode {
            OscillationMode::Off | OscillationMode::EventsOnly => (false, false),
            OscillationMode::Rarely => (randint(0, 99) < 25, false),
            OscillationMode::Sometimes => (randint(0, 99) < 50, false),
            OscillationMode::Often => (randint(0, 99) < 75, randint(0, 99) < 30),
            OscillationMode::AlwaysSlow => (true, false),
            OscillationMode::AlwaysFast => (true, true),
        };

        self.baseline_osc = should_move;
        let move_now = should_move || events.is_surge();

        if move_now {
            let is_tight = self.current_gap < 52.0;
            let max_travel = if is_tight { 8.0 } else { 20.0 };
            let vy = if is_tight {
                0.20
            } else if fast {
                0.50
            } else {
                0.32
            };

            let dir = if randint(0, 1) == 0 { 1.0 } else { -1.0 };
            ent.set_vy(vy * dir);

            let travel_up = (self.base_gap_y - MIN_GAP_Y).max(0).min(max_travel as i16) as f32;
            let travel_down = ((GROUND_Y - MAX_GAP_MARGIN_FROM_GROUND - self.current_gap as i16)
                - self.base_gap_y)
                .max(0)
                .min(max_travel as i16) as f32;

            self.min_y = -travel_up;
            self.max_y = travel_down;

            if self.min_y >= self.max_y {
                ent.set_vy(0.0);
            }
        } else {
            ent.set_vy(0.0);
            self.min_y = 0.0;
            self.max_y = 0.0;
        }
    }

    #[inline(always)]
    pub fn update_gap_and_bounds(&mut self, world: &mut GameWorld, target: f32) {
        self.target_gap = target;
        if (self.current_gap - self.target_gap).abs() > 0.25 {
            self.current_gap += (self.target_gap - self.current_gap) * 0.08;
            self.apply_layout(world);

            let travel_down = ((GROUND_Y - MAX_GAP_MARGIN_FROM_GROUND - self.current_gap as i16)
                - self.base_gap_y)
                .max(0)
                .min(20) as f32;
            self.max_y = travel_down;
        }
    }

    #[inline(always)]
    pub fn set_speed(&mut self, world: &mut GameWorld, speed: f32) {
        world[self.entity_id].set_vx(-speed);
    }

    #[inline(always)]
    pub fn x(&self, world: &GameWorld) -> f32 {
        world[self.entity_id].x()
    }
}

pub struct PipePool {
    pub pipes: Vec<PipePair, MAX_PIPES_ON_SCREEN>,
    pub base_spacing: i16,
    pub speed: f32,
    pub jump_power: f32,
    pub gap_difficulty: u8,
    pub osc_mode: OscillationMode,
    pub last_gap_y: i16,
    pub wave_dir: i8,
    pub wave_steps_left: u8,
    pub surge_active: bool,
}

impl PipePool {
    pub fn new(
        density: u16,
        speed: f32,
        jump_power: f32,
        gap_difficulty: u8,
        osc_mode: OscillationMode,
        events: &EventManager,
        world: &mut GameWorld,
    ) -> Self {
        let base_spacing = match density {
            4 => 60,
            3 => 76,
            2 => 92,
            _ => 118,
        };

        let mut pipes = Vec::new();
        for _ in 0..MAX_PIPES_ON_SCREEN {
            let _ = pipes.push(PipePair::spawn(speed, world));
        }

        let mut wave_dir = 1i8;
        let mut wave_steps_left = 3u8;
        let initial_gap_y = 65;

        // Pipe 0 starts in view ahead of the bird (bird at x = 60)
        let first_x = 160.0;
        pipes[0].set_active(world, true);
        pipes[0].warp_to(
            world,
            first_x,
            osc_mode,
            events,
            initial_gap_y,
            base_spacing,
            speed,
            jump_power,
            gap_difficulty,
            &mut wave_dir,
            &mut wave_steps_left,
        );

        // Pipe 1 starts at exact spacing distance, visible at the right boundary
        let second_x = first_x + base_spacing as f32;
        let prev_gap_y = pipes[0].base_gap_y;
        pipes[1].set_active(world, true);
        pipes[1].warp_to(
            world,
            second_x,
            osc_mode,
            events,
            prev_gap_y,
            base_spacing,
            speed,
            jump_power,
            gap_difficulty,
            &mut wave_dir,
            &mut wave_steps_left,
        );

        let last_gap = pipes[1].base_gap_y;
        Self {
            pipes,
            base_spacing,
            speed,
            jump_power,
            gap_difficulty,
            osc_mode,
            last_gap_y: last_gap,
            wave_dir,
            wave_steps_left,
            surge_active: false,
        }
    }

    pub fn safe_gap(
        prev_gap_y: i16,
        gap_size: f32,
        spacing: i16,
        jump_power: f32,
        gap_difficulty: u8,
        events: &EventManager,
        wave_dir: &mut i8,
        wave_steps_left: &mut u8,
    ) -> i16 {
        let gap_int = gap_size as i16;
        let min_y = MIN_GAP_Y;
        let max_y = (GROUND_Y - MAX_GAP_MARGIN_FROM_GROUND - gap_int).max(min_y + 1);

        let spacing_factor = (spacing as f32 / 92.0).clamp(0.55, 1.35);

        // 1. Dense wave profile
        if events.is_dense() || spacing <= 64 {
            if *wave_steps_left == 0 {
                let roll = randint(0, 9);
                if roll < 2 {
                    *wave_dir = 0; // Short plateau
                    *wave_steps_left = randint(1, 2) as u8;
                } else if *wave_dir <= 0 && prev_gap_y <= min_y + 24 {
                    *wave_dir = 1;
                    *wave_steps_left = randint(2, 4) as u8;
                } else if *wave_dir >= 0 && prev_gap_y >= max_y - 24 {
                    *wave_dir = -1;
                    *wave_steps_left = randint(2, 4) as u8;
                } else {
                    *wave_dir = if *wave_dir <= 0 { 1 } else { -1 };
                    *wave_steps_left = randint(2, 4) as u8;
                }
            } else {
                *wave_steps_left -= 1;
            }

            let step = if *wave_dir == 0 {
                0
            } else {
                let mag = (randint(14, 24) as f32 * spacing_factor) as i16;
                mag.max(8) * (*wave_dir as i16)
            };
            return (prev_gap_y + step).clamp(min_y, max_y);
        }

        // 2. Standard mode with distinct difficulty levels
        let grav = events.gravity_multiplier();
        let jump_factor = jump_power / 4.6;

        let (base_climb, base_dive, min_step_px, allow_flat_chance) = match gap_difficulty {
            0 => (45.0, 50.0, 8, 30),    // Easy
            2 => (105.0, 115.0, 18, 12), // Hard
            _ => (82.0, 88.0, 14, 18),   // Normal
        };

        let max_climb = (((base_climb * jump_factor * spacing_factor) / grav) as i16).max(25);
        let max_dive =
            (((base_dive * spacing_factor * (2.0 - jump_factor).max(0.6)) * grav) as i16).max(28);

        let mut low = (prev_gap_y - max_climb).max(min_y);
        let mut high = (prev_gap_y + max_dive).min(max_y);

        // Anti-trap edge bias
        if prev_gap_y <= min_y + 25 {
            high = (high + 30).min(max_y);
        } else if prev_gap_y >= max_y - 25 {
            low = (low - 30).max(min_y);
        }

        if low >= high {
            return low;
        }

        let sampled = randint(low as u32, high as u32) as i16;
        let delta = sampled - prev_gap_y;

        // Anti-flatness step nudge
        if delta.abs() < min_step_px && randint(0, 99) >= allow_flat_chance {
            let sign = if delta >= 0 {
                if prev_gap_y + min_step_px <= max_y {
                    1
                } else {
                    -1
                }
            } else {
                if prev_gap_y - min_step_px >= min_y {
                    -1
                } else {
                    1
                }
            };
            (prev_gap_y + sign * min_step_px).clamp(min_y, max_y)
        } else {
            sampled
        }
    }

    fn sync_surge(&mut self, world: &mut GameWorld, active: bool) {
        for (i, p) in self.pipes.iter_mut().filter(|p| p.active).enumerate() {
            let ent = &mut world[p.entity_id];
            if active {
                if ent.vy() == 0.0 {
                    let dir = if (i % 2) == 0 { 1.0 } else { -1.0 };
                    let speed = 0.30 + (randint(0, 2) as f32 * 0.08);
                    ent.set_vy(speed * dir);
                    p.min_y = -16.0;
                    p.max_y = 16.0;
                }
            } else if !p.baseline_osc {
                ent.set_vy(0.0);
                p.min_y = 0.0;
                p.max_y = 0.0;
            }
        }
    }

    pub fn update(&mut self, world: &mut GameWorld, bird_x: i16, events: &EventManager) -> bool {
        let surge = events.is_surge();
        if surge != self.surge_active {
            self.surge_active = surge;
            self.sync_surge(world, surge);
        }

        let mut scored_point = false;
        let target_gap = events.target_gap_size();
        let effective_spacing = (self.base_spacing as f32 * events.spacing_multiplier()) as i16;

        let mut max_x: f32 = f32::MIN;
        for p in self.pipes.iter_mut().filter(|p| p.active) {
            p.update_gap_and_bounds(world, target_gap);

            let ent = &mut world[p.entity_id];
            let y = ent.y();
            let vy = ent.vy();

            if (vy < 0.0 && y <= p.min_y) || (vy > 0.0 && y >= p.max_y) {
                ent.set_vy(-vy);
            }

            let px = p.x(world);
            if px > max_x {
                max_x = px;
            }

            if !p.scored && (px + PIPE_WIDTH_PX as f32) < bird_x as f32 {
                p.scored = true;
                scored_point = true;
            }
        }

        if max_x == f32::MIN {
            max_x = VIEW_SCREEN_W as f32;
        }

        // Spawn inactive queue pipes off-screen
        if max_x <= VIEW_SCREEN_W as f32 {
            if let Some(inactive) = self.pipes.iter_mut().find(|p| !p.active) {
                inactive.set_active(world, true);
                let target_x = max_x + effective_spacing as f32;
                inactive.warp_to(
                    world,
                    target_x,
                    self.osc_mode,
                    events,
                    self.last_gap_y,
                    effective_spacing,
                    self.speed,
                    self.jump_power,
                    self.gap_difficulty,
                    &mut self.wave_dir,
                    &mut self.wave_steps_left,
                );
                self.last_gap_y = inactive.base_gap_y;
                max_x = target_x;
            }
        }

        // Recycle pipes once fully past the left boundary
        for p in self.pipes.iter_mut().filter(|p| p.active) {
            let px = p.x(world);
            if px < -((PIPE_WIDTH_PX + 20) as f32) {
                let target_x = max_x + effective_spacing as f32;
                p.warp_to(
                    world,
                    target_x,
                    self.osc_mode,
                    events,
                    self.last_gap_y,
                    effective_spacing,
                    self.speed,
                    self.jump_power,
                    self.gap_difficulty,
                    &mut self.wave_dir,
                    &mut self.wave_steps_left,
                );
                self.last_gap_y = p.base_gap_y;
                max_x = target_x;
            }
        }

        scored_point
    }

    #[inline(always)]
    pub fn has_dense_ahead(&self, world: &GameWorld, bird_x: i16) -> bool {
        self.pipes
            .iter()
            .any(|p| p.active && p.is_dense && (p.x(world) + PIPE_WIDTH_PX as f32 >= bird_x as f32))
    }

    pub fn has_steep_leap_ahead(&self, world: &GameWorld, bird_x: i16) -> bool {
        let mut upcoming_pipes: [Option<&PipePair>; 2] = [None, None];
        let mut count = 0;

        for p in self.pipes.iter().filter(|p| p.active) {
            if p.x(world) + PIPE_WIDTH_PX as f32 >= bird_x as f32 && count < 2 {
                upcoming_pipes[count] = Some(p);
                count += 1;
            }
        }

        match (upcoming_pipes[0], upcoming_pipes[1]) {
            (Some(p1), Some(p2)) => (p1.base_gap_y - p2.base_gap_y).abs() >= 54,
            _ => false,
        }
    }

    #[inline(always)]
    pub fn collides(&self, world: &GameWorld, bird_id: usize) -> bool {
        self.pipes
            .iter()
            .any(|p| p.active && world.collides(bird_id, p.entity_id))
    }

    pub fn set_speed(&mut self, world: &mut GameWorld, speed: f32) {
        self.speed = speed;
        for p in self.pipes.iter_mut().filter(|p| p.active) {
            p.set_speed(world, speed);
        }
    }
}
