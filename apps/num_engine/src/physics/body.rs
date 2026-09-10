//! Kinematic states, velocity integration, and impulse forces.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Body {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub active: bool,
}

impl Body {
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            active: true,
        }
    }

    pub const fn with_velocity(mut self, vx: f32, vy: f32) -> Self {
        self.vx = vx;
        self.vy = vy;
        self
    }

    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    // --- Simulation & Stepping ---

    /// Advances position by velocity when active.
    #[inline(always)]
    pub fn update(&mut self) {
        if self.active {
            self.x += self.vx;
            self.y += self.vy;
        }
    }

    /// Truncates sub-pixel coordinates to integer screen space.
    #[inline(always)]
    pub fn integer_pos(&self) -> [i16; 2] {
        [self.x as i16, self.y as i16]
    }

    // --- Impulses, Forces & Damping ---

    #[inline(always)]
    pub fn apply_impulse(&mut self, ix: f32, iy: f32) {
        self.vx += ix;
        self.vy += iy;
    }

    #[inline(always)]
    pub fn apply_gravity(&mut self, gravity: f32, max_fall_speed: f32) {
        self.vy = (self.vy + gravity).min(max_fall_speed);
    }

    #[inline(always)]
    pub fn apply_damping(&mut self, factor: f32) {
        self.vx *= factor;
        self.vy *= factor;
    }

    #[inline(always)]
    pub fn apply_friction_x(&mut self, friction: f32) {
        if self.vx > 0.0 {
            self.vx = (self.vx - friction).max(0.0);
        } else if self.vx < 0.0 {
            self.vx = (self.vx + friction).min(0.0);
        }
    }

    #[inline(always)]
    pub fn clamp_velocity(&mut self, max_vx: f32, max_vy: f32) {
        self.vx = self.vx.clamp(-max_vx, max_vx);
        self.vy = self.vy.clamp(-max_vy, max_vy);
    }

    #[inline(always)]
    pub fn stop(&mut self) {
        self.vx = 0.0;
        self.vy = 0.0;
    }

    #[inline(always)]
    pub fn stop_x(&mut self) {
        self.vx = 0.0;
    }

    #[inline(always)]
    pub fn stop_y(&mut self) {
        self.vy = 0.0;
    }

    // --- Spatial Queries ---

    #[inline(always)]
    pub fn distance_squared_to(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx) + (dy * dy)
    }

    #[inline(always)]
    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    #[inline(always)]
    pub fn set_velocity(&mut self, vx: f32, vy: f32) {
        self.vx = vx;
        self.vy = vy;
    }
}
