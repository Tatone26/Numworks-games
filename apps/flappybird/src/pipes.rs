use heapless::Vec;
use num_engine::{hitbox, physics::Body, spawn_sprite, world::Entity};
use numworks_utils::utils::randint;

use crate::{
    flappy_ui::{ANIM_PIPE_LIP_BOT, ANIM_PIPE_LIP_TOP, ANIM_PIPE_SHAFT, TILESET_TILE_SIZE},
    game::{
        GameWorld, GROUND_Y, MAX_PIPES_ON_SCREEN, TILE_SIZE, VIEW_SCREEN_W, VIEW_SCREEN_X,
        WINDOW_ROWS,
    },
};

const SHAFT_HEIGHT_PX: i16 = (WINDOW_ROWS * TILE_SIZE) as i16;
pub const PIPE_WIDTH_PX: u16 = TILESET_TILE_SIZE * 2;

pub struct PipePair {
    pub entity_id: usize,
    pub gap_y: i16,
    pub gap_size: i16,
    pub scored: bool,
    pub active: bool,
}

impl PipePair {
    pub fn spawn(gap_size: i16, speed: f32, world: &mut GameWorld) -> Self {
        let spawn_x = (VIEW_SCREEN_X + VIEW_SCREEN_W) as i16;
        let gap_y = Self::random_gap(gap_size);
        let collar_offset_x = -(TILESET_TILE_SIZE as i16);

        let body = Body::new(spawn_x as f32, 0.0).with_velocity(-speed, 0.0);

        let top_shaft = spawn_sprite!(&ANIM_PIPE_SHAFT, [0, 0], 10)
            .with_offset([0, (gap_y - 20) - SHAFT_HEIGHT_PX]);
        let top_lip = spawn_sprite!(&ANIM_PIPE_LIP_TOP, [0, 0], 11)
            .with_offset([collar_offset_x, gap_y - 20]);
        let bot_lip = spawn_sprite!(&ANIM_PIPE_LIP_BOT, [0, 0], 11)
            .with_offset([collar_offset_x, gap_y + gap_size]);
        let bot_shaft =
            spawn_sprite!(&ANIM_PIPE_SHAFT, [0, 0], 10).with_offset([0, gap_y + gap_size + 20]);

        let top_box = hitbox!(0, 0, PIPE_WIDTH_PX, gap_y.max(0) as u16);
        let bot_y = gap_y + gap_size;
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
            gap_y,
            gap_size,
            scored: false,
            active: false,
        }
    }

    fn random_gap(gap_size: i16) -> i16 {
        randint(20, (GROUND_Y - 20 - gap_size).max(21) as u32) as i16
    }

    pub fn set_active(&mut self, world: &mut GameWorld, active: bool) {
        self.active = active;
        world[self.entity_id].set_active(active);
    }

    fn apply_layout(&self, world: &mut GameWorld) {
        let collar_offset_x = -(TILESET_TILE_SIZE as i16);
        let ent = &mut world[self.entity_id];

        ent.sprites[0].offset = [0, (self.gap_y - 20) - SHAFT_HEIGHT_PX];
        ent.sprites[1].offset = [collar_offset_x, self.gap_y - 20];
        ent.sprites[2].offset = [collar_offset_x, self.gap_y + self.gap_size];
        ent.sprites[3].offset = [0, self.gap_y + self.gap_size + 20];

        ent.hitboxes[0].height = self.gap_y.max(0) as u16;
        let bot_y = self.gap_y + self.gap_size;
        ent.hitboxes[1].offset_y = bot_y;
        ent.hitboxes[1].height = (GROUND_Y - bot_y).max(0) as u16;
    }

    pub fn warp_to(&mut self, world: &mut GameWorld, target_x: i16) {
        self.gap_y = Self::random_gap(self.gap_size);
        self.scored = false;

        self.apply_layout(world);
        world[self.entity_id].set_pos(target_x as f32, 0.0);
    }

    #[inline(always)]
    pub fn set_speed(&mut self, world: &mut GameWorld, speed: f32) {
        world[self.entity_id].set_vx(-speed);
    }

    #[inline(always)]
    pub fn x(&self, world: &GameWorld) -> i16 {
        world[self.entity_id].position()[0]
    }
}

pub struct PipePool {
    pub pipes: Vec<PipePair, MAX_PIPES_ON_SCREEN>,
    pub spacing: i16,
}

impl PipePool {
    pub fn new(density: u16, speed: f32, world: &mut GameWorld) -> Self {
        let spacing = match density {
            3 => 75,
            2 => 105,
            _ => 145,
        };

        let mut pipes = Vec::new();
        for _ in 0..MAX_PIPES_ON_SCREEN {
            let _ = pipes.push(PipePair::spawn(75, speed, world));
        }

        // Initialize the first pipe just off the right edge of the screen
        pipes[0].set_active(world, true);
        pipes[0].warp_to(world, VIEW_SCREEN_W as i16);

        Self { pipes, spacing }
    }

    /// Single-pass update handling scoring, queue advancement, and recycling.
    /// Returns true if a pipe was passed this frame.
    pub fn update(&mut self, world: &mut GameWorld, bird_x: i16) -> bool {
        let mut scored_point = false;

        // 1. Scoring & Finding the current true leading pipe position
        let mut max_x = i16::MIN;
        for p in self.pipes.iter_mut().filter(|p| p.active) {
            let px = p.x(world);

            if px > max_x {
                max_x = px;
            }

            if !p.scored && (px + PIPE_WIDTH_PX as i16) < bird_x {
                p.scored = true;
                scored_point = true;
            }
        }

        // If no active pipes exist, fallback to screen edge
        if max_x == i16::MIN {
            max_x = VIEW_SCREEN_W as i16;
        }

        // 2. Queue activation (fill out the screen initially until pool limit is reached)
        if max_x <= VIEW_SCREEN_W as i16 {
            if let Some(inactive) = self.pipes.iter_mut().find(|p| !p.active) {
                inactive.set_active(world, true);
                let target_x = max_x + self.spacing;
                inactive.warp_to(world, target_x);
                max_x = target_x;
            }
        }

        // 3. Recycle pipes that have completely left the screen on the left.
        // Lip width is 80 px (collar offset -20 to 60), so <-80 guarantees fully offscreen.
        for p in self.pipes.iter_mut().filter(|p| p.active) {
            let px = p.x(world);
            if px < -80 {
                let target_x = max_x + self.spacing;
                p.warp_to(world, target_x);
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
        for p in self.pipes.iter_mut() {
            p.set_speed(world, speed);
        }
    }
}
