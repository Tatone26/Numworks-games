use numworks_utils::utils::randint;

use crate::pipes::DEFAULT_GAP_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GameEvent {
    #[default]
    None,
    MovingPipesSurge,
    Tailwind,
    Headwind,
    NarrowGaps,
    WideGaps,
    DensePipes,
    LowGravity,
    HighGravity,
}

#[derive(Clone, Copy, Debug)]
pub struct EventConfig {
    pub frequency: u8,
    pub enable_moving_surge: bool,
    pub enable_wind: bool,
    pub enable_narrow_gaps: bool,
    pub enable_wide_gaps: bool,
    pub enable_dense_pipes: bool,
    pub enable_low_gravity: bool,
    pub enable_high_gravity: bool,
}

pub struct EventManager {
    pub config: EventConfig,
    pub pipe_event: GameEvent,
    pub phys_event: GameEvent,
    pipe_timer: u16,
    phys_timer: u16,
    phys_warmup: u16,
    phys_winddown: u16,
    cooldown: u16,
    candidates: [GameEvent; 8],
    nb_active: usize,
    last_triggered: GameEvent,
}

impl EventManager {
    pub fn new(config: EventConfig) -> Self {
        let mut candidates = [GameEvent::None; 8];
        let mut count = 0;

        if config.enable_moving_surge {
            candidates[count] = GameEvent::MovingPipesSurge;
            count += 1;
        }
        if config.enable_wind {
            candidates[count] = GameEvent::Tailwind;
            count += 1;
            candidates[count] = GameEvent::Headwind;
            count += 1;
        }
        if config.enable_narrow_gaps {
            candidates[count] = GameEvent::NarrowGaps;
            count += 1;
        }
        if config.enable_wide_gaps {
            candidates[count] = GameEvent::WideGaps;
            count += 1;
        }
        if config.enable_dense_pipes {
            candidates[count] = GameEvent::DensePipes;
            count += 1;
        }
        if config.enable_low_gravity {
            candidates[count] = GameEvent::LowGravity;
            count += 1;
        }
        if config.enable_high_gravity {
            candidates[count] = GameEvent::HighGravity;
            count += 1;
        }

        let initial_delay = Self::sample_cooldown(config.frequency);

        Self {
            config,
            pipe_event: GameEvent::None,
            phys_event: GameEvent::None,
            pipe_timer: 0,
            phys_timer: 0,
            phys_warmup: 0,
            phys_winddown: 0,
            cooldown: initial_delay,
            candidates,
            nb_active: count,
            last_triggered: GameEvent::None,
        }
    }

    fn sample_cooldown(freq: u8) -> u16 {
        match freq {
            3 => randint(90, 161) as u16,  // ~2.2s to 4.0s
            2 => randint(180, 321) as u16, // ~4.5s to 8.0s
            1 => randint(340, 561) as u16, // ~8.5s to 14.0s
            _ => 1000,
        }
    }

    #[inline(always)]
    fn is_pipe_modifier(e: GameEvent) -> bool {
        matches!(
            e,
            GameEvent::MovingPipesSurge
                | GameEvent::NarrowGaps
                | GameEvent::WideGaps
                | GameEvent::DensePipes
        )
    }

    pub fn update(&mut self, dense_cleared: bool, steep_leap_ahead: bool) -> bool {
        if self.config.frequency == 0 {
            return false;
        }

        let mut changed = false;

        // 1. Tick pipe geometry/gap event
        if self.pipe_timer > 0 {
            self.pipe_timer -= 1;
            if self.pipe_timer == 0 && self.pipe_event != GameEvent::DensePipes {
                self.pipe_event = GameEvent::None;
                changed = true;
            }
        } else if self.pipe_event == GameEvent::DensePipes && dense_cleared {
            self.pipe_event = GameEvent::None;
            self.cooldown = Self::sample_cooldown(self.config.frequency);
            changed = true;
        }

        // 2. Tick atmospheric physics event
        if self.phys_warmup > 0 {
            self.phys_warmup -= 1;
            if self.phys_warmup == 0 {
                changed = true;
            }
        } else if self.phys_timer > 0 {
            self.phys_timer -= 1;
            if self.phys_timer == 0 {
                self.phys_winddown = match self.phys_event {
                    GameEvent::Tailwind | GameEvent::Headwind => 28,
                    _ => 22,
                };
                changed = true;
            }
        } else if self.phys_winddown > 0 {
            self.phys_winddown -= 1;
            if self.phys_winddown == 0 {
                self.phys_event = GameEvent::None;
                changed = true;
            }
        }

        // 3. Roll new event when cooldown expires
        if self.cooldown > 0 {
            self.cooldown -= 1;
        } else if self.nb_active > 0 {
            let has_active =
                self.pipe_event != GameEvent::None || self.phys_event != GameEvent::None;
            let stack_chance = match self.config.frequency {
                3 => 20,
                2 => 12,
                _ => 6,
            };

            if !has_active || randint(0, 100) < stack_chance {
                // Fair candidate probing: start at a random offset and check all options
                // to prevent starvation of index 0 when other slots are busy.
                let start_idx = randint(0, self.nb_active as u32) as usize;
                let mut triggered = false;

                for i in 0..self.nb_active {
                    let idx = (start_idx + i) % self.nb_active;
                    let cand = self.candidates[idx];

                    // Avoid repeating the exact same event back-to-back if choices exist
                    if self.nb_active > 1 && cand == self.last_triggered {
                        continue;
                    }

                    if self.try_trigger(cand, steep_leap_ahead) {
                        self.last_triggered = cand;
                        self.cooldown = Self::sample_cooldown(self.config.frequency);
                        changed = true;
                        triggered = true;
                        break;
                    }
                }

                if !triggered {
                    self.cooldown = 24;
                }
            } else {
                self.cooldown = Self::sample_cooldown(self.config.frequency);
            }
        }

        changed
    }

    fn try_trigger(&mut self, candidate: GameEvent, steep_leap_ahead: bool) -> bool {
        if Self::is_pipe_modifier(candidate) {
            if self.pipe_event != GameEvent::None {
                return false;
            }
            if candidate == GameEvent::DensePipes && self.phys_event != GameEvent::None {
                return false;
            }

            self.pipe_event = candidate;
            self.pipe_timer = match candidate {
                GameEvent::MovingPipesSurge => randint(280, 361) as u16,
                GameEvent::DensePipes => randint(240, 311) as u16,
                _ => randint(260, 341) as u16,
            };
            true
        } else {
            if self.phys_event != GameEvent::None {
                return false;
            }
            if self.pipe_event == GameEvent::DensePipes {
                return false;
            }

            if steep_leap_ahead && matches!(candidate, GameEvent::Tailwind | GameEvent::HighGravity)
            {
                return false;
            }

            self.phys_event = candidate;
            self.phys_warmup = match candidate {
                GameEvent::Tailwind | GameEvent::Headwind => 36,
                _ => 28,
            };
            self.phys_timer = match candidate {
                GameEvent::Tailwind | GameEvent::Headwind => randint(220, 291) as u16,
                _ => randint(200, 261) as u16,
            };
            self.phys_winddown = 0;
            true
        }
    }

    #[inline(always)]
    pub fn speed_multiplier(&self) -> f32 {
        if self.phys_event != GameEvent::None && self.phys_warmup == 0 {
            match self.phys_event {
                GameEvent::Tailwind => 1.65,
                GameEvent::Headwind => 0.60,
                _ => 1.0,
            }
        } else {
            1.0
        }
    }

    #[inline(always)]
    pub fn gravity_multiplier(&self) -> f32 {
        if self.phys_event != GameEvent::None && self.phys_warmup == 0 {
            match self.phys_event {
                GameEvent::LowGravity => 0.64,
                GameEvent::HighGravity => 1.34,
                _ => 1.0,
            }
        } else {
            1.0
        }
    }

    #[inline(always)]
    pub fn target_gap_size(&self) -> f32 {
        match self.pipe_event {
            GameEvent::NarrowGaps => 50.0,
            GameEvent::WideGaps => 84.0,
            _ => DEFAULT_GAP_SIZE,
        }
    }

    #[inline(always)]
    pub fn spacing_multiplier(&self) -> f32 {
        if self.is_dense() {
            0.68
        } else {
            1.0
        }
    }

    #[inline(always)]
    pub fn is_dense(&self) -> bool {
        self.pipe_event == GameEvent::DensePipes && self.pipe_timer > 0
    }

    #[inline(always)]
    pub fn is_surge(&self) -> bool {
        self.pipe_event == GameEvent::MovingPipesSurge
    }

    #[inline(always)]
    pub fn active_particle_event(&self) -> GameEvent {
        if self.phys_timer > 0 || self.phys_warmup > 0 {
            self.phys_event
        } else {
            GameEvent::None
        }
    }
}
