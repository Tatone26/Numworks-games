use heapless::Vec;
use num_engine::{hitbox, physics::Body, spawn_sprite, world::Entity};
use numworks_utils::utils::randint;

use crate::{
    events::EventManager,
    flappy_ui::{ANIM_PIPE_LIP_BOT, ANIM_PIPE_LIP_TOP, ANIM_PIPE_SHAFT, TILESET_TILE_SIZE},
    game::{
        GameWorld, GROUND_Y, MAX_PIPES_ON_SCREEN, TILE_SIZE, VIEW_SCREEN_W, VIEW_SCREEN_X,
        WINDOW_ROWS,
    },
};

const SHAFT_HEIGHT_PX: i16 = (WINDOW_ROWS * TILE_SIZE) as i16;
pub const PIPE_WIDTH_PX: u16 = TILESET_TILE_SIZE * 2;

// default gap between top and bottom pipe
pub const DEFAULT_GAP_SIZE: f32 = 68.0;

const MIN_GAP_Y: i16 = 12;
const MAX_GAP_MARGIN_FROM_GROUND: i16 = 12;

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
}

impl PipePair {
    pub fn spawn(speed: f32, world: &mut GameWorld) -> Self {
        let spawn_x = (VIEW_SCREEN_X + VIEW_SCREEN_W) as f32;
        let gap_y = 70;
        let collar_offset_x = -(TILESET_TILE_SIZE as i16);

        let body = Body::new(spawn_x, 0.0).with_velocity(-speed, 0.0);

        let top_shaft = spawn_sprite!(&ANIM_PIPE_SHAFT, [0, 0], 10)
            .with_offset([0, (gap_y - 20) - SHAFT_HEIGHT_PX]);
        let top_lip = spawn_sprite!(&ANIM_PIPE_LIP_TOP, [0, 0], 11)
            .with_offset([collar_offset_x, gap_y - 20]);
        let bot_lip = spawn_sprite!(&ANIM_PIPE_LIP_BOT, [0, 0], 11)
            .with_offset([collar_offset_x, gap_y + DEFAULT_GAP_SIZE as i16]);
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
        }
    }

    fn safe_gap(prev_gap_y: i16, gap_size: f32, spacing: i16, events: &EventManager) -> i16 {
        let gap_int = gap_size as i16;
        let min_y = MIN_GAP_Y;
        let max_y = (GROUND_Y - MAX_GAP_MARGIN_FROM_GROUND - gap_int).max(min_y + 1);

        let grav = events.gravity_multiplier();
        let reach_climb = ((spacing as f32 * 0.44) / grav).clamp(20.0, 85.0) as i16;
        let reach_dive = ((spacing as f32 * 0.48) * grav).clamp(20.0, 85.0) as i16;

        let low = (prev_gap_y - reach_climb).max(min_y);
        let high = (prev_gap_y + reach_dive).min(max_y);

        if low >= high {
            low
        } else {
            randint(low as u32, high as u32) as i16
        }
    }

    pub fn set_active(&mut self, world: &mut GameWorld, active: bool) {
        self.active = active;
        world[self.entity_id].set_active(active);
    }

    fn apply_layout(&self, world: &mut GameWorld) {
        let gy = self.base_gap_y;
        let gap_int = self.current_gap as i16;
        let collar_offset_x = -(TILESET_TILE_SIZE as i16);
        let ent = &mut world[self.entity_id];

        ent.sprites[0].offset = [0, (gy - 20) - SHAFT_HEIGHT_PX];
        ent.sprites[1].offset = [collar_offset_x, gy - 20];
        ent.sprites[2].offset = [collar_offset_x, gy + gap_int];
        ent.sprites[3].offset = [0, gy + gap_int + 20];

        ent.hitboxes[0].height = gy.max(0) as u16;
        let bot_y = gy + gap_int;
        ent.hitboxes[1].offset_y = bot_y;
        ent.hitboxes[1].height = (GROUND_Y - bot_y).max(0) as u16;
    }

    pub fn warp_to(
        &mut self,
        world: &mut GameWorld,
        target_x: f32,
        mode: OscillationMode,
        events: &EventManager,
        prev_gap_y: i16,
        spacing: i16,
    ) {
        self.target_gap = events.target_gap_size();
        self.current_gap = self.target_gap;
        self.base_gap_y = Self::safe_gap(prev_gap_y, self.current_gap, spacing, events);
        self.scored = false;

        self.apply_layout(world);

        let ent = &mut world[self.entity_id];
        ent.set_pos(target_x, 0.0);

        let (should_move, fast) = match mode {
            OscillationMode::Off | OscillationMode::EventsOnly => (false, false),
            OscillationMode::Rarely => (randint(0, 99) < 25, false),
            OscillationMode::Sometimes => (randint(0, 99) < 50, false),
            OscillationMode::Often => (randint(0, 99) < 75, randint(0, 99) < 30),
            OscillationMode::AlwaysSlow => (true, false),
            OscillationMode::AlwaysFast => (true, true),
        };

        self.baseline_osc = should_move;
        let move_now = should_move || events.is_moving_surge();

        if move_now {
            let is_tight = self.current_gap < 52.0;
            let max_travel: f32 = if is_tight { 8.0 } else { 22.0 };
            let speed = if is_tight {
                0.20
            } else if fast {
                0.55
            } else {
                0.35
            };

            let dir = if randint(0, 1) == 0 { 1.0 } else { -1.0 };
            ent.set_vy(speed * dir);

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
                .min(22) as f32;
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
    pub osc_mode: OscillationMode,
    pub last_gap_y: i16,
}

impl PipePool {
    pub fn new(
        density: u16,
        speed: f32,
        osc_mode: OscillationMode,
        events: &EventManager,
        world: &mut GameWorld,
    ) -> Self {
        let base_spacing = match density {
            4 => 60,  // Extreme
            3 => 76,  // Dense
            2 => 92,  // Normal (Authentic arcade spacing)
            _ => 118, // Sparse
        };

        let mut pipes = Vec::new();
        for _ in 0..MAX_PIPES_ON_SCREEN {
            let _ = pipes.push(PipePair::spawn(speed, world));
        }

        let initial_gap_y = 65;
        pipes[0].set_active(world, true);
        pipes[0].warp_to(
            world,
            VIEW_SCREEN_W as f32,
            osc_mode,
            events,
            initial_gap_y,
            base_spacing,
        );

        let last_gap = pipes[0].base_gap_y;
        Self {
            pipes,
            base_spacing,
            speed,
            osc_mode,
            last_gap_y: last_gap,
        }
    }

    pub fn apply_moving_surge(&mut self, world: &mut GameWorld) {
        for (i, p) in self.pipes.iter_mut().filter(|p| p.active).enumerate() {
            let ent = &mut world[p.entity_id];
            if ent.vy() == 0.0 {
                let dir = if (i % 2) == 0 { 1.0 } else { -1.0 };
                let speed = 0.32 + (randint(0, 3) as f32 * 0.08);
                ent.set_vy(speed * dir);
                p.min_y = -18.0;
                p.max_y = 18.0;
            }
        }
    }

    pub fn stop_surge(&mut self, world: &mut GameWorld) {
        for p in self.pipes.iter_mut().filter(|p| p.active) {
            if !p.baseline_osc {
                let ent = &mut world[p.entity_id];
                ent.set_vy(0.0);
                p.min_y = 0.0;
                p.max_y = 0.0;
            }
        }
    }

    pub fn update(&mut self, world: &mut GameWorld, bird_x: i16, events: &EventManager) -> bool {
        let mut scored_point = false;
        let target_gap = events.target_gap_size();
        let effective_spacing = (self.base_spacing as f32 * events.spacing_multiplier()) as i16;

        let mut max_x: f32 = f32::MIN;
        for p in self.pipes.iter_mut().filter(|p| p.active) {
            p.update_gap_and_bounds(world, target_gap);

            let ent = &mut world[p.entity_id];
            let y = ent.y();
            let vy = ent.vy();

            if vy < 0.0 && y <= p.min_y {
                ent.set_y(p.min_y);
                ent.set_vy(-vy);
            } else if vy > 0.0 && y >= p.max_y {
                ent.set_y(p.max_y);
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

        if max_x <= VIEW_SCREEN_W as f32 {
            if let Some(inactive) = self.pipes.iter_mut().find(|p| !p.active) {
                inactive.set_active(world, true);
                inactive.set_speed(world, self.speed);
                let target_x = max_x + effective_spacing as f32;
                inactive.warp_to(
                    world,
                    target_x,
                    self.osc_mode,
                    events,
                    self.last_gap_y,
                    effective_spacing,
                );
                self.last_gap_y = inactive.base_gap_y;
                max_x = target_x;
            }
        }

        for p in self.pipes.iter_mut().filter(|p| p.active) {
            let px = p.x(world);
            if px < -80.0 {
                let target_x = max_x + effective_spacing as f32;
                p.warp_to(
                    world,
                    target_x,
                    self.osc_mode,
                    events,
                    self.last_gap_y,
                    effective_spacing,
                );
                self.last_gap_y = p.base_gap_y;
                max_x = target_x;
            }
        }

        scored_point
    }

    #[inline(always)]
    pub fn collides(&self, world: &GameWorld, bird_id: usize) -> bool {
        self.pipes
            .iter()
            .any(|p| p.active && world.collides(bird_id, p.entity_id))
    }

    pub fn set_speed(&mut self, world: &mut GameWorld, speed: f32) {
        self.speed = speed;
        for p in self.pipes.iter_mut() {
            p.set_speed(world, speed);
        }
    }
}
