use crate::game::{Grid, Space, GRID_WIDTH};
use numworks_utils::eadk::Point;

pub const TARGET_FPS: f32 = 45.0;
pub const ARCADE_FPS: f32 = 60.0;
pub const FPS_SCALE: f32 = ARCADE_FPS / TARGET_FPS; // ~1.333333

pub const FRUIT_SPAWN_1: u16 = 70;
pub const FRUIT_SPAWN_2: u16 = 170;
pub const FRUIT_DURATION_MS: u64 = 9500;
pub const FRUIT_SCORES: [u32; 8] = [100, 300, 500, 700, 1000, 2000, 3000, 5000];

pub const fn fruit_for_level(level: u16) -> u8 {
    match level {
        0 | 1 => 0,   // Level 1: Cherry
        2 => 1,       // Level 2: Strawberry
        3 | 4 => 2,   // Peach
        5 | 6 => 3,   // Apple
        7 | 8 => 4,   // Grapes
        9 | 10 => 5,  // Galaxian
        11 | 12 => 6, // Bell
        _ => 7,       // Key (Level 13+)
    }
}

#[derive(Clone, Copy)]
pub struct WaveTimings {
    pub scatter_1: u64,
    pub chase_1: u64,
    pub scatter_2: u64,
    pub chase_2: u64,
    pub scatter_3: u64,
    pub chase_3: u64,
    pub scatter_4: u64,
}

impl WaveTimings {
    pub const fn for_level(level: u16) -> Self {
        match level {
            0 | 1 => Self {
                scatter_1: 7000,
                chase_1: 20000,
                scatter_2: 7000,
                chase_2: 20000,
                scatter_3: 5000,
                chase_3: 20000,
                scatter_4: 5000,
            },
            2..=4 => Self {
                scatter_1: 7000,
                chase_1: 20000,
                scatter_2: 7000,
                chase_2: 20000,
                scatter_3: 5000,
                chase_3: 1033000,
                scatter_4: 1,
            },
            _ => Self {
                scatter_1: 5000,
                chase_1: 20000,
                scatter_2: 5000,
                chase_2: 20000,
                scatter_3: 5000,
                chase_3: 1037000,
                scatter_4: 1,
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct LevelConfig {
    pub pac_speed: f32,
    pub pac_dot_speed: f32,
    pub ghost_speed: f32,
    pub frightened_ghost_speed: f32,
    pub tunnel_ghost_speed: f32,
    pub eaten_ghost_speed: f32,
    pub frightened_duration_ms: u64,
    pub blink_window_ms: u64,
    pub inky_release_ms: u64,
    pub clyde_release_ms: u64,
    pub fruit_id: u8,
    pub wave_timings: WaveTimings,
}

impl LevelConfig {
    pub fn for_level(level: u16) -> Self {
        let eff_level = if level == 0 { 1 } else { level };

        let (pac_pct, pac_dot_pct, ghost_pct, fright_pct, tunnel_pct) = match eff_level {
            1 => (0.80, 0.71, 0.75, 0.50, 0.40),
            2..=4 => (0.90, 0.79, 0.85, 0.55, 0.45),
            5..=20 => (1.00, 0.87, 0.95, 0.60, 0.50),
            _ => (0.90, 0.79, 0.95, 0.60, 0.50),
        };

        let fright_ms = match eff_level {
            1 => 6000,
            2 => 5000,
            3 => 4000,
            4 => 3000,
            5 | 7 | 8 | 11 => 2000,
            6 | 10 => 5000,
            9 | 12 | 13 | 14 => 1000,
            _ => 0, // Level 15-18 & 19+ has 0ms duration
        };

        Self {
            pac_speed: pac_pct * FPS_SCALE,
            pac_dot_speed: pac_dot_pct * FPS_SCALE,
            ghost_speed: ghost_pct * FPS_SCALE,
            frightened_ghost_speed: fright_pct * FPS_SCALE,
            tunnel_ghost_speed: tunnel_pct * FPS_SCALE,
            eaten_ghost_speed: 1.65 * FPS_SCALE,
            frightened_duration_ms: fright_ms,
            blink_window_ms: if fright_ms == 0 {
                0
            } else {
                2000.min(fright_ms / 2)
            },
            inky_release_ms: if eff_level == 1 { 4000 } else { 2000 },
            clyde_release_ms: if eff_level == 1 { 8000 } else { 4000 },
            fruit_id: fruit_for_level(eff_level),
            wave_timings: WaveTimings::for_level(eff_level),
        }
    }
}

pub struct LevelManager {
    pub current_level: u16,
    pub config: LevelConfig,
    pub fruit_spawn_pos: Point,
    pub fruit_active: bool,
    despawn_until: u64,
    fruits_spawned: u8,
}

impl LevelManager {
    pub fn new(start_level: u16) -> Self {
        let eff_level = if start_level == 0 { 1 } else { start_level };
        Self {
            current_level: eff_level,
            config: LevelConfig::for_level(eff_level),
            fruit_spawn_pos: Point { x: 13, y: 16 },
            fruit_active: false,
            despawn_until: 0,
            fruits_spawned: 0,
        }
    }

    pub fn advance_level(&mut self) {
        self.current_level += 1;
        self.config = LevelConfig::for_level(self.current_level);
        self.fruit_active = false;
        self.despawn_until = 0;
        self.fruits_spawned = 0;
    }

    pub fn check_fruit_spawn(&mut self, dots_eaten: u16, now: u64, grid: &mut Grid) -> bool {
        let should = (dots_eaten >= FRUIT_SPAWN_1 && self.fruits_spawned == 0)
            || (dots_eaten >= FRUIT_SPAWN_2 && self.fruits_spawned == 1);

        if should {
            self.fruits_spawned += 1;
            self.fruit_active = true;
            self.despawn_until = now + FRUIT_DURATION_MS;
            let idx =
                self.fruit_spawn_pos.y as usize * GRID_WIDTH + self.fruit_spawn_pos.x as usize;
            grid[idx] = Space::Fruit;
            true
        } else {
            false
        }
    }

    pub fn update(&mut self, now: u64, grid: &mut Grid) -> bool {
        if self.fruit_active && now >= self.despawn_until {
            self.fruit_active = false;
            let idx =
                self.fruit_spawn_pos.y as usize * GRID_WIDTH + self.fruit_spawn_pos.x as usize;
            if grid[idx] == Space::Fruit {
                grid[idx] = Space::Empty;
            }
            true
        } else {
            false
        }
    }

    pub fn collect_fruit(&mut self, grid: &mut Grid) -> u32 {
        self.fruit_active = false;
        let idx = self.fruit_spawn_pos.y as usize * GRID_WIDTH + self.fruit_spawn_pos.x as usize;
        grid[idx] = Space::Empty;
        FRUIT_SCORES[self.config.fruit_id as usize]
    }
}
