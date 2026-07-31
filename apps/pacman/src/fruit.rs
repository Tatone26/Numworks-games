use crate::{
    game::{Grid, Space, GRID_WIDTH},
    pac_ui::draw_fruit,
};
use numworks_utils::{
    eadk::{timing, Point},
    utils::randint,
};

pub struct FruitManager {
    pub spawn_pos: Point,
    pub fruit_active: bool,
    despawn_until: u64,
    spawn_count: u8, // Tracks if 1st (70 dots) or 2nd (170 dots) fruit spawned
}

const FRUIT_VALUES: [u32; 8] = [100, 300, 500, 700, 1000, 2000, 3000, 5000];
const FRUITS_SPAWNS: [usize; 13] = [0, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7];

impl FruitManager {
    pub fn new() -> Self {
        Self {
            spawn_pos: Point { x: 13, y: 16 },
            fruit_active: false,
            despawn_until: 0,
            spawn_count: 0,
        }
    }

    pub fn get_current_fruit_type(&self, level: u32) -> usize {
        let corresponding_index = (level as i16 - 1).clamp(0, 12) as usize;
        FRUITS_SPAWNS[corresponding_index]
    }

    pub fn get_fruit_value(&self, level: u32) -> u32 {
        FRUIT_VALUES[FRUITS_SPAWNS[self.get_current_fruit_type(level)]]
    }

    /// Call this whenever Pac-Man eats a dot
    pub fn check_dot_spawn(&mut self, dots_eaten: u16, grid: &mut Grid) {
        if (dots_eaten == 70 && self.spawn_count == 0)
            || (dots_eaten == 170 && self.spawn_count == 1)
        {
            self.spawn_count += 1;
            self.fruit_active = true;
            self.despawn_until = timing::millis() + 9000 + randint(0, 1000) as u64;

            // Put fruit on the grid
            let idx = (self.spawn_pos.x + self.spawn_pos.y * GRID_WIDTH) as usize;
            grid[idx] = Space::Fruit;
        }
    }

    /// Call this every frame in advance_game_state
    pub fn update(&mut self, grid: &mut Grid) {
        if self.fruit_active && timing::millis() >= self.despawn_until {
            self.fruit_active = false;
            self.despawn_until = 0;

            // Remove fruit if Pac-Man didn't eat it in time
            let idx = (self.spawn_pos.x + self.spawn_pos.y * GRID_WIDTH) as usize;
            if grid[idx] == Space::Fruit {
                grid[idx] = Space::Empty;
            }
        }
    }
}
