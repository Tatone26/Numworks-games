use numworks_utils::utils::randint;

use crate::pipes::DEFAULT_GAP_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameEvent {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventPhase {
    /// No event active
    Idle,
    /// Visuals/particles running as a telegraph; physics multipliers are 1.0
    Warmup,
    /// Full event physics active
    Active,
}

#[derive(Clone, Copy, Debug)]
pub struct EventConfig {
    pub frequency: u8, // 0 = Never, 1 = Rare, 2 = Normal, 3 = Frequent
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
    pub active: GameEvent,
    pub phase: EventPhase,
    pub warmup_frames: u16,
    pub duration_frames: u16,
    pub cooldown_frames: u16,
    candidates: [GameEvent; 8],
    nb_active: usize,
}

impl EventManager {
    pub fn new(config: EventConfig) -> Self {
        let initial_cooldown = match config.frequency {
            3 => randint(30, 60) as u16,   // Frequent: ~1-1.5s
            2 => randint(70, 130) as u16,  // Normal: ~2-3.5s
            1 => randint(140, 220) as u16, // Rare: ~4-6s
            _ => 0,
        };

        let mut candidates: [GameEvent; 8] = [GameEvent::None; 8];
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

        Self {
            config,
            active: GameEvent::None,
            phase: EventPhase::Idle,
            warmup_frames: 0,
            duration_frames: 0,
            cooldown_frames: initial_cooldown,
            candidates,
            nb_active: count,
        }
    }

    fn sample_cooldown(freq: u8) -> u16 {
        match freq {
            3 => randint(50, 110) as u16,  // Frequent: ~1.5 - 3s
            2 => randint(150, 260) as u16, // Normal: ~4 - 7s
            1 => randint(300, 500) as u16, // Rare: ~8 - 13s
            _ => 0,
        }
    }

    /// Advances the event timer and phase state machine.
    /// Returns `true` whenever any state transition occurs:
    /// - Idle -> Warmup (visuals turn on)
    /// - Warmup -> Active (multipliers engage)
    /// - Active -> Idle (event ends, returns to baseline)
    pub fn update(&mut self) -> bool {
        if self.config.frequency == 0 {
            return false;
        }

        match self.phase {
            EventPhase::Warmup => {
                if self.warmup_frames > 0 {
                    self.warmup_frames -= 1;
                    false
                } else {
                    self.phase = EventPhase::Active;
                    true // Multipliers engage now
                }
            }
            EventPhase::Active => {
                if self.duration_frames > 0 {
                    self.duration_frames -= 1;
                    false
                } else {
                    self.active = GameEvent::None;
                    self.phase = EventPhase::Idle;
                    self.cooldown_frames = Self::sample_cooldown(self.config.frequency);
                    true // Event ended
                }
            }
            EventPhase::Idle => {
                if self.cooldown_frames > 0 {
                    self.cooldown_frames -= 1;
                    false
                } else {
                    self.trigger_random();
                    self.active != GameEvent::None // Transitioned to Warmup
                }
            }
        }
    }

    fn trigger_random(&mut self) {
        if self.nb_active > 0 {
            let choice = randint(0, self.nb_active as u32) as usize;
            self.active = self.candidates[choice];

            // Telegraph warmup frames: gives particles time to cover ~2/3 of the display
            self.warmup_frames = match self.active {
                GameEvent::Tailwind | GameEvent::Headwind => 24, // ~400ms at ~8px/frame
                GameEvent::LowGravity | GameEvent::HighGravity => 20, // ~330ms vertical rise/fall
                _ => 0, // Surges and gap adjustments start immediately
            };

            self.duration_frames = match self.active {
                GameEvent::MovingPipesSurge => 320,
                GameEvent::Tailwind | GameEvent::Headwind => 260,
                GameEvent::NarrowGaps | GameEvent::WideGaps => 300,
                GameEvent::DensePipes => 280,
                GameEvent::LowGravity | GameEvent::HighGravity => 240,
                GameEvent::None => 0,
            };

            self.phase = if self.warmup_frames > 0 {
                EventPhase::Warmup
            } else {
                EventPhase::Active
            };
        } else {
            self.cooldown_frames = Self::sample_cooldown(self.config.frequency);
        }
    }

    /// Speed multiplier is neutral (1.0) during Warmup and applies only during Active phase.
    #[inline(always)]
    pub fn speed_multiplier(&self) -> f32 {
        if self.phase != EventPhase::Active {
            return 1.0;
        }

        match self.active {
            GameEvent::Tailwind => 1.75,
            GameEvent::Headwind => 0.55,
            _ => 1.0,
        }
    }

    /// Gravity multiplier is neutral (1.0) during Warmup and applies only during Active phase.
    #[inline(always)]
    pub fn gravity_multiplier(&self) -> f32 {
        if self.phase != EventPhase::Active {
            return 1.0;
        }

        match self.active {
            GameEvent::LowGravity => 0.6,
            GameEvent::HighGravity => 1.4,
            _ => 1.0,
        }
    }

    #[inline(always)]
    pub fn target_gap_size(&self) -> f32 {
        match self.active {
            GameEvent::NarrowGaps => 50.0,
            GameEvent::WideGaps => 84.0,
            _ => DEFAULT_GAP_SIZE,
        }
    }

    #[inline(always)]
    pub fn spacing_multiplier(&self) -> f32 {
        match self.active {
            GameEvent::DensePipes => 0.60,
            _ => 1.0,
        }
    }

    #[inline(always)]
    pub fn is_moving_surge(&self) -> bool {
        self.active == GameEvent::MovingPipesSurge && self.phase == EventPhase::Active
    }
}
