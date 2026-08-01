use crate::game::{Grid, Space, GRID_WIDTH};
use numworks_utils::eadk::Point;

pub const TARGET_FPS: f32 = 45.0;
pub const ARCADE_FPS: f32 = 60.0;

/// Scale factor to convert 60 FPS arcade speeds to the 45 FPS of the NumWorks
pub const FPS_SCALE: f32 = ARCADE_FPS / TARGET_FPS; // ~1.333333

// Total dots in the maze
pub const TOTAL_DOTS: u16 = 238;

// Dot counts that trigger fruit spawns (1st fruit at 70 dots, 2nd at 170 dots)
pub const FRUIT_SPAWN_DOT_COUNT_1: u16 = 70;
pub const FRUIT_SPAWN_DOT_COUNT_2: u16 = 170;
pub const FRUIT_DESPAWN_TIME_MS: u64 = 9500;

// Standard blinking warning window before frightened mode expires
pub const DEFAULT_BLINK_WINDOW_MS: u64 = 2000;

// 0: Cherry, 1: Strawberry, 2: Peach, 3: Apple, 4: Grapes, 5: Galaxian, 6: Bell, 7: Key
pub const FRUIT_SCORES: [u32; 8] = [100, 300, 500, 700, 1000, 2000, 3000, 5000];

pub const fn fruit_for_level(level: u16) -> u8 {
    match level {
        1 => 0,
        2 => 1,
        3 | 4 => 2,
        5 | 6 => 3,
        7 | 8 => 4,
        9 | 10 => 5,
        11 | 12 => 6,
        _ => 7, // Level 13+
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
            1 => Self {
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
                chase_3: 1033000, // ~17 minutes chase
                scatter_4: 1,     // 1ms then permanent chase
            },
            _ => Self {
                // Level 5+
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
    pub level: u16,

    // Speeds (steps per frame at 45 FPS)
    pub pac_speed: f32,
    pub ghost_speed: f32,
    pub frightened_ghost_speed: f32,
    pub tunnel_ghost_speed: f32,
    pub eaten_ghost_speed: f32,

    // Frightened state mechanics
    pub frightened_duration_ms: u64,
    pub blink_window_ms: u64,
    pub flash_count: u8, // Number of warning blinks

    // House release delays for ghosts (Inky and Clyde)
    pub inky_release_ms: u64,
    pub clyde_release_ms: u64,

    // Active fruit index (0 to 7)
    pub fruit_id: u8,

    // AI Wave timings
    pub wave_timings: WaveTimings,
}

impl LevelConfig {
    pub fn for_level(level: u16) -> Self {
        let (pac_pct, ghost_pct, fright_ghost_pct, tunnel_ghost_pct) = match level {
            1 => (0.80, 0.75, 0.50, 0.40),
            2..=4 => (0.90, 0.85, 0.55, 0.45),
            5..=20 => (1.00, 0.95, 0.60, 0.50),
            _ => (0.90, 0.95, 0.60, 0.50), // Level 21+ Pac-Man slows slightly
        };

        let (fright_ms, flashes) = match level {
            1 => (6000, 5),
            2 => (5000, 5),
            3 => (4000, 5),
            4 => (3000, 5),
            5 | 7 | 8 | 11 => (2000, 5),
            6 | 10 => (5000, 5),
            9 | 14 => (1000, 3),
            12 | 13 | 15 | 16 | 18 => (1000, 0),
            _ => (0, 0), // Level 17, 19+ (No frightened mode)
        };

        let blink_window = if fright_ms == 0 {
            0
        } else {
            DEFAULT_BLINK_WINDOW_MS.min(fright_ms / 2)
        };

        let (inky_release, clyde_release) = match level {
            1 => (4000, 8000),
            2 => (3000, 6000),
            _ => (2000, 4000),
        };

        Self {
            level,

            pac_speed: pac_pct * FPS_SCALE,
            ghost_speed: ghost_pct * FPS_SCALE,
            frightened_ghost_speed: fright_ghost_pct * FPS_SCALE,
            tunnel_ghost_speed: tunnel_ghost_pct * FPS_SCALE,
            eaten_ghost_speed: 1.60 * FPS_SCALE,

            frightened_duration_ms: fright_ms,
            blink_window_ms: blink_window,
            flash_count: flashes,

            inky_release_ms: inky_release,
            clyde_release_ms: clyde_release,

            fruit_id: fruit_for_level(level),
            wave_timings: WaveTimings::for_level(level),
        }
    }
}

// ============================================================================
// LEVEL MANAGER STRUCT
// ============================================================================

pub struct LevelManager {
    pub current_level: u16,
    pub config: LevelConfig,
    pub fruit_spawn_pos: Point,
    pub fruit_active: bool,
    pub fruit_eaten: bool,
    despawn_until: u64,
    fruits_spawned_this_level: u8,
}

impl LevelManager {
    pub fn new(start_level: u16) -> Self {
        Self {
            current_level: start_level,
            config: LevelConfig::for_level(start_level),
            fruit_spawn_pos: Point { x: 13, y: 16 },
            fruit_active: false,
            fruit_eaten: false,
            despawn_until: 0,
            fruits_spawned_this_level: 0,
        }
    }

    pub fn advance_level(&mut self) {
        self.current_level += 1;
        self.config = LevelConfig::for_level(self.current_level);
        self.reset_level_state();
    }

    pub fn reset_level_state(&mut self) {
        self.fruit_active = false;
        self.fruit_eaten = false;
        self.despawn_until = 0;
        self.fruits_spawned_this_level = 0;
    }

    pub fn check_fruit_spawn(&mut self, dots_eaten: u16, now: u64, grid: &mut Grid) {
        let should_spawn = (dots_eaten >= FRUIT_SPAWN_DOT_COUNT_1
            && self.fruits_spawned_this_level == 0)
            || (dots_eaten >= FRUIT_SPAWN_DOT_COUNT_2 && self.fruits_spawned_this_level == 1);

        if should_spawn {
            self.fruits_spawned_this_level += 1;
            self.fruit_active = true;
            self.despawn_until = now + FRUIT_DESPAWN_TIME_MS;

            let idx = (self.fruit_spawn_pos.x + self.fruit_spawn_pos.y * GRID_WIDTH) as usize;
            grid[idx] = Space::Fruit;
        }
    }

    pub fn update(&mut self, now: u64, grid: &mut Grid) {
        if self.fruit_active && now >= self.despawn_until {
            self.fruit_active = false;
            self.despawn_until = 0;

            let idx = (self.fruit_spawn_pos.x + self.fruit_spawn_pos.y * GRID_WIDTH) as usize;
            if grid[idx] == Space::Fruit {
                grid[idx] = Space::Empty;
            }
        }
    }

    pub fn collect_fruit(&mut self, grid: &mut Grid) -> u32 {
        self.fruit_active = false;
        self.fruit_eaten = true;
        self.despawn_until = 0;

        let idx = (self.fruit_spawn_pos.x + self.fruit_spawn_pos.y * GRID_WIDTH) as usize;
        grid[idx] = Space::Empty;

        FRUIT_SCORES[self.config.fruit_id as usize]
    }
}
