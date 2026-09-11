//! Zero-allocation particle engine using hardware-accelerated f32 kinematics.

use numworks_utils::eadk::{Color, Rect};

use crate::graphics::viewport::Viewport;

pub const DEFAULT_PARTICLE_CAP: usize = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ParticleKind {
    #[default]
    HorizontalStreak, // Length on X, thickness on Y
    VerticalStreak, // Length on Y, thickness on X
    Pixel,          // 1x1 dot
    Square,         // w == h (randomized together)
    Rectangle,      // Independent [min_w..max_w] and [min_h..max_h]
}

impl ParticleKind {
    #[inline(always)]
    pub fn dimensions(self, size: u8) -> (i16, i16) {
        match self {
            Self::HorizontalStreak => (size as i16, 1),
            Self::VerticalStreak => (1, size as i16),
            Self::Pixel => (1, 1),
            Self::Square | Self::Rectangle => (size as i16, size as i16),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub prev_x: i16,
    pub prev_y: i16,
    pub color: Color,
    pub life: u8,
    pub w: u8,
    pub h: u8,
}

impl Particle {
    pub const fn empty() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            prev_x: i16::MIN,
            prev_y: i16::MIN,
            color: Color::BLACK,
            life: 0,
            w: 1,
            h: 1,
        }
    }

    #[inline(always)]
    pub fn is_active(&self) -> bool {
        self.life > 0
    }

    #[inline(always)]
    pub fn needs_clear(&self) -> bool {
        self.life == 0 && self.prev_x != i16::MIN
    }
}

#[derive(Clone, Copy)]
pub struct Emitter {
    pub area: [i16; 4],
    pub rate: u8,
    pub interval: u8,
    pub vx_range: [f32; 2],
    pub vy_range: [f32; 2],
    pub size_x_range: [u8; 2],
    pub size_y_range: [u8; 2],
    pub life_range: [u8; 2],
    pub color: Color,
    pub enabled: bool,
    timer: u8,
}

impl Emitter {
    pub const fn empty() -> Self {
        Self {
            area: [0, 0, 0, 0],
            rate: 0,
            interval: 1,
            vx_range: [0.0, 0.0],
            vy_range: [0.0, 0.0],
            size_x_range: [1, 1],
            size_y_range: [1, 1],
            life_range: [30, 30],
            color: Color::WHITE,
            enabled: false,
            timer: 0,
        }
    }
}

/// Fully dyn-compatible renderer trait for heterogeneous particle systems.
pub trait ParticleRenderer<const TILE_SIZE: usize, const CELL_AREA: usize> {
    fn z(&self) -> u8;
    fn is_visible(
        &self,
        win_x0: i16,
        win_y0: i16,
        win_x1: i16,
        win_y1: i16,
        viewport: &Viewport,
    ) -> bool;
    fn screen_bounds(&self, viewport: &Viewport) -> Option<[i16; 4]>;
    fn mark_dirty(&mut self, viewport: &Viewport, mark_rect: &mut dyn FnMut(Rect));
    fn blit_to_cell(
        &self,
        cell_pos: [i16; 2],
        viewport: &Viewport,
        buffer: &mut [Color; CELL_AREA],
        covered: &mut [bool; CELL_AREA],
        pixels_covered: &mut usize,
    );
    fn commit_frame(&mut self);
}

pub struct ParticleSystem<const CAP: usize = DEFAULT_PARTICLE_CAP> {
    pub particles: [Particle; CAP],
    pub emitter: Option<Emitter>,
    pub kind: ParticleKind,
    pub gravity_x: f32,
    pub gravity_y: f32,
    pub z: u8,
    pub active: bool,
    rng: u32,
}

impl<const CAP: usize> ParticleSystem<CAP> {
    pub fn new(z: u8, kind: ParticleKind) -> Self {
        let hw_seed = numworks_utils::eadk::random();
        Self {
            particles: [Particle::empty(); CAP],
            emitter: None,
            kind,
            gravity_x: 0.0,
            gravity_y: 0.0,
            z,
            active: true,
            rng: if hw_seed == 0 { 0x9E37_79B9 } else { hw_seed },
        }
    }

    #[inline(always)]
    pub fn set_gravity(&mut self, gx: f32, gy: f32) {
        self.gravity_x = gx;
        self.gravity_y = gy;
    }

    #[inline(always)]
    pub fn set_emitter(&mut self, emitter: Emitter) {
        self.emitter = Some(emitter);
    }

    #[inline(always)]
    pub fn enable_emitter(&mut self, enabled: bool) {
        if let Some(ref mut em) = self.emitter {
            em.enabled = enabled;
        }
    }

    // --- Inlined 3-Cycle PRNG ---

    #[inline(always)]
    fn rand(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }

    #[inline(always)]
    fn rand_range_i16(&mut self, min: i16, max: i16) -> i16 {
        if min >= max {
            return min;
        }
        let span = (max - min + 1) as u32;
        min + (self.rand() % span) as i16
    }

    #[inline(always)]
    fn rand_range_f32(&mut self, min: f32, max: f32) -> f32 {
        if min >= max {
            return min;
        }
        let norm = (self.rand() & 0xFFFF) as f32 * (1.0 / 65535.0);
        min + norm * (max - min)
    }

    #[inline(always)]
    fn rand_range_u8(&mut self, min: u8, max: u8) -> u8 {
        if min >= max {
            return min;
        }
        let span = (max - min + 1) as u32;
        min + (self.rand() % span) as u8
    }

    // --- Inherent Screen Bounds ---

    #[inline(always)]
    pub fn screen_bounds(&self, viewport: &Viewport) -> Option<[i16; 4]> {
        let mut min_x = i16::MAX;
        let mut min_y = i16::MAX;
        let mut max_x = i16::MIN;
        let mut max_y = i16::MIN;
        let mut any = false;

        for p in self.particles.iter() {
            if p.is_active() || p.needs_clear() {
                let x = if p.is_active() { p.x as i16 } else { p.prev_x };
                let y = if p.is_active() { p.y as i16 } else { p.prev_y };
                let (sx, sy) = viewport.world_to_screen(x as i32, y as i32);
                let sx1 = sx + p.w as i16;
                let sy1 = sy + p.h as i16;

                min_x = min_x.min(sx);
                min_y = min_y.min(sy);
                max_x = max_x.max(sx1);
                max_y = max_y.max(sy1);
                any = true;
            }
        }

        if any {
            Some([min_x, min_y, max_x, max_y])
        } else {
            None
        }
    }

    // --- Emission Methods ---

    pub fn emit(
        &mut self,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        w: u8,
        h: u8,
        color: Color,
        life: u8,
    ) -> bool {
        for p in self.particles.iter_mut() {
            if p.life == 0 && p.prev_x == i16::MIN {
                p.x = x;
                p.y = y;
                p.vx = vx;
                p.vy = vy;
                p.prev_x = x as i16;
                p.prev_y = y as i16;
                p.color = color;
                p.w = w.max(1);
                p.h = h.max(1);
                p.life = life;
                return true;
            }
        }
        false
    }

    pub fn burst(
        &mut self,
        pos: [f32; 2],
        count: usize,
        speed: [f32; 2],
        size: [u8; 2],
        color: Color,
        life: [u8; 2],
    ) {
        for _ in 0..count {
            let vx = self.rand_range_f32(speed[0], speed[1]);
            let vy = self.rand_range_f32(speed[0], speed[1]);
            let l = self.rand_range_u8(life[0], life[1]);
            if !self.emit(pos[0], pos[1], vx, vy, size[0], size[1], color, l) {
                break;
            }
        }
    }

    pub fn emit_streak(
        &mut self,
        pos: [f32; 2],
        speed: f32,
        length: u8,
        thickness: u8,
        color: Color,
        life: u8,
    ) -> bool {
        self.emit(
            pos[0],
            pos[1],
            speed,
            0.0,
            length,
            thickness.max(1),
            color,
            life,
        )
    }

    pub fn clear(&mut self) {
        for p in self.particles.iter_mut() {
            if p.life > 0 {
                p.life = 0;
            }
        }
    }

    // --- Simulation Step ---

    pub fn update(&mut self) {
        if let Some(mut em) = self.emitter {
            if em.enabled {
                if em.timer == 0 {
                    em.timer = em.interval.saturating_sub(1);
                    for _ in 0..em.rate {
                        let x = self.rand_range_i16(em.area[0], em.area[0] + em.area[2].max(1) - 1)
                            as f32;
                        let y = self.rand_range_i16(em.area[1], em.area[1] + em.area[3].max(1) - 1)
                            as f32;
                        let vx = self.rand_range_f32(em.vx_range[0], em.vx_range[1]);
                        let vy = self.rand_range_f32(em.vy_range[0], em.vy_range[1]);

                        let (w, h) = match self.kind {
                            ParticleKind::Pixel => (1, 1),
                            ParticleKind::Square => {
                                let s = self.rand_range_u8(em.size_x_range[0], em.size_x_range[1]);
                                (s, s)
                            }
                            ParticleKind::HorizontalStreak => {
                                let w = self.rand_range_u8(em.size_x_range[0], em.size_x_range[1]);
                                let h = self.rand_range_u8(em.size_y_range[0], em.size_y_range[1]);
                                (w, h.max(1))
                            }
                            ParticleKind::VerticalStreak => {
                                let w = self.rand_range_u8(em.size_x_range[0], em.size_x_range[1]);
                                let h = self.rand_range_u8(em.size_y_range[0], em.size_y_range[1]);
                                (w.max(1), h)
                            }
                            ParticleKind::Rectangle => {
                                let w = self.rand_range_u8(em.size_x_range[0], em.size_x_range[1]);
                                let h = self.rand_range_u8(em.size_y_range[0], em.size_y_range[1]);
                                (w.max(1), h.max(1))
                            }
                        };

                        let life = self.rand_range_u8(em.life_range[0], em.life_range[1]);

                        if !self.emit(x, y, vx, vy, w, h, em.color, life) {
                            break;
                        }
                    }
                } else {
                    em.timer -= 1;
                }
            }
            self.emitter = Some(em);
        }

        for p in self.particles.iter_mut() {
            if p.life > 0 {
                p.prev_x = p.x as i16;
                p.prev_y = p.y as i16;

                p.vx += self.gravity_x;
                p.vy += self.gravity_y;
                p.x += p.vx;
                p.y += p.vy;

                p.life -= 1;
            }
        }
    }
}

impl<const CAP: usize, const TILE_SIZE: usize, const CELL_AREA: usize>
    ParticleRenderer<TILE_SIZE, CELL_AREA> for ParticleSystem<CAP>
{
    #[inline(always)]
    fn z(&self) -> u8 {
        self.z
    }

    #[inline(always)]
    fn is_visible(
        &self,
        win_x0: i16,
        win_y0: i16,
        win_x1: i16,
        win_y1: i16,
        viewport: &Viewport,
    ) -> bool {
        self.screen_bounds(viewport).map_or(false, |b| {
            !(b[0] > win_x1 || b[2] < win_x0 || b[1] > win_y1 || b[3] < win_y0)
        })
    }

    #[inline(always)]
    fn screen_bounds(&self, viewport: &Viewport) -> Option<[i16; 4]> {
        ParticleSystem::screen_bounds(self, viewport)
    }

    #[inline(always)]
    fn mark_dirty(&mut self, viewport: &Viewport, mark_rect: &mut dyn FnMut(Rect)) {
        let win_x0 = viewport.screen_x as i16;
        let win_y0 = viewport.screen_y as i16;
        let win_x1 = win_x0 + viewport.screen_w as i16;
        let win_y1 = win_y0 + viewport.screen_h as i16;

        for p in self.particles.iter_mut() {
            let w = p.w as i16;
            let h = p.h as i16;

            if p.is_active() {
                if let Some(r) = clamped_rect(
                    viewport, p.prev_x, p.prev_y, w, h, win_x0, win_y0, win_x1, win_y1,
                ) {
                    mark_rect(r);
                }
                if let Some(r) = clamped_rect(
                    viewport, p.x as i16, p.y as i16, w, h, win_x0, win_y0, win_x1, win_y1,
                ) {
                    mark_rect(r);
                }
            } else if p.needs_clear() {
                if let Some(r) = clamped_rect(
                    viewport, p.prev_x, p.prev_y, w, h, win_x0, win_y0, win_x1, win_y1,
                ) {
                    mark_rect(r);
                }
                p.prev_x = i16::MIN;
            }
        }
    }

    #[inline(always)]
    fn blit_to_cell(
        &self,
        cell_pos: [i16; 2],
        viewport: &Viewport,
        buffer: &mut [Color; CELL_AREA],
        covered: &mut [bool; CELL_AREA],
        pixels_covered: &mut usize,
    ) {
        let [cell_x, cell_y] = cell_pos;
        let cell_x1 = cell_x + TILE_SIZE as i16;
        let cell_y1 = cell_y + TILE_SIZE as i16;

        for p in self.particles.iter() {
            if !p.is_active() || *pixels_covered >= CELL_AREA {
                continue;
            }

            let (sx, sy) = viewport.world_to_screen(p.x as i32, p.y as i32);
            let px1 = sx + p.w as i16;
            let py1 = sy + p.h as i16;

            if px1 <= cell_x || sx >= cell_x1 || py1 <= cell_y || sy >= cell_y1 {
                continue;
            }

            let x0 = sx.max(cell_x);
            let y0 = sy.max(cell_y);
            let x1 = px1.min(cell_x1);
            let y1 = py1.min(cell_y1);

            for y in y0..y1 {
                let row = (y - cell_y) as usize * TILE_SIZE;
                for x in x0..x1 {
                    let idx = row + (x - cell_x) as usize;
                    if !covered[idx] {
                        buffer[idx] = p.color;
                        covered[idx] = true;
                        *pixels_covered += 1;
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn commit_frame(&mut self) {
        for p in self.particles.iter_mut() {
            if p.is_active() {
                p.prev_x = p.x as i16;
                p.prev_y = p.y as i16;
            }
        }
    }
}

#[inline(always)]
fn clamped_rect(
    viewport: &Viewport,
    wx: i16,
    wy: i16,
    w: i16,
    h: i16,
    win_x0: i16,
    win_y0: i16,
    win_x1: i16,
    win_y1: i16,
) -> Option<Rect> {
    let (sx, sy) = viewport.world_to_screen(wx as i32, wy as i32);
    let x0 = sx.max(win_x0);
    let y0 = sy.max(win_y0);
    let x1 = (sx + w).min(win_x1);
    let y1 = (sy + h).min(win_y1);

    if x1 > x0 && y1 > y0 {
        Some(Rect {
            x: (x0 - win_x0) as u16,
            y: (y0 - win_y0) as u16,
            width: (x1 - x0) as u16,
            height: (y1 - y0) as u16,
        })
    } else {
        None
    }
}
