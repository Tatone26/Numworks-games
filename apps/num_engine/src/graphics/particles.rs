//! Zero-allocation particle engine using hardware-accelerated f32 kinematics and math overlays.

use numworks_utils::eadk::{Color, Rect};

use crate::graphics::{dirty_grid::DirtyGrid, viewport::Viewport};

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

pub struct ParticleSystem<const CAP: usize = DEFAULT_PARTICLE_CAP> {
    pub particles: [Particle; CAP],
    pub emitter: Option<Emitter>,
    pub kind: ParticleKind,
    pub gravity_x: f32,
    pub gravity_y: f32,
    pub active: bool,
    rng: u32,
}

impl<const CAP: usize> ParticleSystem<CAP> {
    pub fn new(_z_ignored: u8, kind: ParticleKind) -> Self {
        let hw_seed = numworks_utils::eadk::random();
        Self {
            particles: [Particle::empty(); CAP],
            emitter: None,
            kind,
            gravity_x: 0.0,
            gravity_y: 0.0,
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

    #[allow(clippy::too_many_arguments)]
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
                p.vx += self.gravity_x;
                p.vy += self.gravity_y;
                p.x += p.vx;
                p.y += p.vy;

                p.life -= 1;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn mark_rect<const TILE_SIZE: usize, const COLS: usize, const ROWS: usize>(
        grid: &mut DirtyGrid<TILE_SIZE, COLS, ROWS>,
        viewport: &Viewport,
        wx: i16,
        wy: i16,
        w: i16,
        h: i16,
        win_x0: i16,
        win_y0: i16,
        win_x1: i16,
        win_y1: i16,
    ) {
        let sx = wx as i32 - viewport.x;
        let sy = wy as i32 - viewport.y;
        let x0 = (sx as i16).max(win_x0);
        let y0 = (sy as i16).max(win_y0);
        let x1 = ((sx + w as i32) as i16).min(win_x1);
        let y1 = ((sy + h as i32) as i16).min(win_y1);

        if x1 > x0 && y1 > y0 {
            grid.mark_rect(Rect {
                x: (x0 - win_x0) as u16,
                y: (y0 - win_y0) as u16,
                width: (x1 - x0) as u16,
                height: (y1 - y0) as u16,
            });
        }
    }

    pub(crate) fn mark_dirty<const TILE_SIZE: usize, const COLS: usize, const ROWS: usize>(
        &self,
        viewport: &Viewport,
        grid: &mut DirtyGrid<TILE_SIZE, COLS, ROWS>,
    ) {
        let win_x0 = viewport.screen_x as i16;
        let win_y0 = viewport.screen_y as i16;
        let win_x1 = win_x0 + viewport.screen_w as i16;
        let win_y1 = win_y0 + viewport.screen_h as i16;

        for p in self.particles.iter() {
            if p.is_active() {
                let w = p.w as i16;
                let h = p.h as i16;
                if p.prev_x != i16::MIN {
                    Self::mark_rect(
                        grid, viewport, p.prev_x, p.prev_y, w, h, win_x0, win_y0, win_x1, win_y1,
                    );
                }
                Self::mark_rect(
                    grid, viewport, p.x as i16, p.y as i16, w, h, win_x0, win_y0, win_x1, win_y1,
                );
            } else if p.needs_clear() {
                Self::mark_rect(
                    grid, viewport, p.prev_x, p.prev_y, p.w as i16, p.h as i16, win_x0, win_y0,
                    win_x1, win_y1,
                );
            }
        }
    }

    /// Zero-cost post-process mathematical overlay. Bypasses opacity masks.
    #[inline(always)]
    pub fn blit_to_cell<const TILE_SIZE: usize, const CELL_AREA: usize>(
        &self,
        cell_pos: [i16; 2],
        viewport: &Viewport,
        buffer: &mut [Color; CELL_AREA],
    ) {
        let [cell_x, cell_y] = cell_pos;
        let cell_x1 = cell_x + TILE_SIZE as i16;
        let cell_y1 = cell_y + TILE_SIZE as i16;

        for p in self.particles.iter() {
            if !p.is_active() {
                continue;
            }

            let sx = p.x as i32 - viewport.x;
            let sy = p.y as i32 - viewport.y;
            let px1 = sx + p.w as i32;
            let py1 = sy + p.h as i32;

            let sx_i = sx as i16;
            let sy_i = sy as i16;
            let px1_i = px1 as i16;
            let py1_i = py1 as i16;

            if px1_i <= cell_x || sx_i >= cell_x1 || py1_i <= cell_y || sy_i >= cell_y1 {
                continue;
            }

            let x0 = sx_i.max(cell_x);
            let y0 = sy_i.max(cell_y);
            let x1 = px1_i.min(cell_x1);
            let y1 = py1_i.min(cell_y1);

            for y in y0..y1 {
                let row = (y - cell_y) as usize * TILE_SIZE;
                for x in x0..x1 {
                    buffer[row + (x - cell_x) as usize] = p.color;
                }
            }
        }
    }

    pub fn commit_frame(&mut self) {
        for p in self.particles.iter_mut() {
            if p.is_active() {
                p.prev_x = p.x as i16;
                p.prev_y = p.y as i16;
            } else if p.needs_clear() {
                p.prev_x = i16::MIN;
            }
        }
    }
}
