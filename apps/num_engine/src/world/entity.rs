//! High-level composite game entity binding a Body to multiple Sprites and Hitboxes.

use crate::{
    graphics::{sprite::Sprite, texture::Animation, tilemap::Tilemap},
    physics::{body::Body, hitbox::Hitbox},
};
use heapless::Vec;

pub struct Entity<'a, const TILE_SIZE: usize, const CELL_AREA: usize, const PARTS: usize = 4> {
    pub body: Option<Body>,
    pub sprites: Vec<Sprite<'a, TILE_SIZE, CELL_AREA>, PARTS>,
    pub hitboxes: Vec<Hitbox, PARTS>,
    pub active: bool,
    pub allocated: bool,
}

impl<'a, const TILE_SIZE: usize, const CELL_AREA: usize, const PARTS: usize> Default
    for Entity<'a, TILE_SIZE, CELL_AREA, PARTS>
{
    #[inline(always)]
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a, const TILE_SIZE: usize, const CELL_AREA: usize, const PARTS: usize>
    Entity<'a, TILE_SIZE, CELL_AREA, PARTS>
{
    /// Unallocated, dormant slot inside the World.
    pub const fn empty() -> Self {
        Self {
            body: None,
            sprites: Vec::new(),
            hitboxes: Vec::new(),
            active: false,
            allocated: false,
        }
    }

    /// Fresh entity ready for configuration and spawning.
    pub const fn new() -> Self {
        Self {
            body: None,
            sprites: Vec::new(),
            hitboxes: Vec::new(),
            active: true,
            allocated: false,
        }
    }

    pub const fn with_body(mut self, body: Body) -> Self {
        self.body = Some(body);
        self
    }

    pub fn with_sprite(mut self, sprite: Sprite<'a, TILE_SIZE, CELL_AREA>) -> Self {
        let _ = self.sprites.push(sprite);
        self
    }

    pub fn with_hitbox(mut self, hitbox: Hitbox) -> Self {
        let _ = self.hitboxes.push(hitbox);
        self
    }

    // --- Fluent Kinematic Accessors ---

    #[inline(always)]
    pub fn x(&self) -> f32 {
        self.body.map_or(0.0, |b| b.x)
    }

    #[inline(always)]
    pub fn y(&self) -> f32 {
        self.body.map_or(0.0, |b| b.y)
    }

    #[inline(always)]
    pub fn vx(&self) -> f32 {
        self.body.map_or(0.0, |b| b.vx)
    }

    #[inline(always)]
    pub fn vy(&self) -> f32 {
        self.body.map_or(0.0, |b| b.vy)
    }

    #[inline(always)]
    pub fn set_x(&mut self, x: f32) {
        if let Some(ref mut b) = self.body {
            b.x = x;
        }
        self.sync_sprites();
    }

    #[inline(always)]
    pub fn set_y(&mut self, y: f32) {
        if let Some(ref mut b) = self.body {
            b.y = y;
        }
        self.sync_sprites();
    }

    #[inline(always)]
    pub fn set_pos(&mut self, x: f32, y: f32) {
        if let Some(ref mut b) = self.body {
            b.x = x;
            b.y = y;
        }
        self.sync_sprites();
    }

    #[inline(always)]
    pub fn set_vx(&mut self, vx: f32) {
        if let Some(ref mut b) = self.body {
            b.vx = vx;
        }
    }

    #[inline(always)]
    pub fn set_vy(&mut self, vy: f32) {
        if let Some(ref mut b) = self.body {
            b.vy = vy;
        }
    }

    #[inline(always)]
    pub fn set_velocity(&mut self, vx: f32, vy: f32) {
        if let Some(ref mut b) = self.body {
            b.vx = vx;
            b.vy = vy;
        }
    }

    #[inline(always)]
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        for spr in self.sprites.iter_mut() {
            spr.active = active;
        }
    }

    // --- Visual & Animation Accessors ---

    #[inline(always)]
    pub fn sprite(&self) -> Option<&Sprite<'a, TILE_SIZE, CELL_AREA>> {
        self.sprites.first()
    }

    #[inline(always)]
    pub fn sprite_mut(&mut self) -> Option<&mut Sprite<'a, TILE_SIZE, CELL_AREA>> {
        self.sprites.first_mut()
    }

    #[inline(always)]
    pub fn set_animation(&mut self, animation: &'a Animation<'a>) {
        if let Some(spr) = self.sprites.first_mut() {
            spr.set_animation(animation);
        }
    }

    // --- Simulation & Synchronization ---

    pub fn update(&mut self, frame: u32) {
        if !self.active {
            return;
        }

        if let Some(ref mut body) = self.body {
            body.update();
            let anchor = body.integer_pos();

            for spr in self.sprites.iter_mut() {
                if spr.active {
                    spr.sync_to_anchor(anchor);
                    spr.update_animation(frame);
                }
            }
        } else {
            for spr in self.sprites.iter_mut() {
                if spr.active {
                    spr.update_animation(frame);
                }
            }
        }
    }

    #[inline(always)]
    pub fn sync_sprites(&mut self) {
        let anchor = self.position();
        for spr in self.sprites.iter_mut() {
            spr.sync_to_anchor(anchor);
        }
    }

    #[inline(always)]
    pub fn position(&self) -> [i16; 2] {
        if let Some(ref body) = self.body {
            body.integer_pos()
        } else if let Some(spr) = self.sprites.first() {
            spr.position
        } else {
            [0, 0]
        }
    }

    pub fn collides_with(&self, other: &Self) -> bool {
        if !self.active || !other.active {
            return false;
        }
        let pos_a = self.position();
        let pos_b = other.position();

        for ha in self.hitboxes.iter().filter(|h| h.active) {
            for hb in other.hitboxes.iter().filter(|h| h.active) {
                if ha.intersects(pos_a, hb, pos_b) {
                    return true;
                }
            }
        }
        false
    }

    pub fn collides_hitbox(&self, box_: &Hitbox, box_pos: [i16; 2]) -> bool {
        if !self.active {
            return false;
        }
        let pos = self.position();
        for h in self.hitboxes.iter().filter(|h| h.active) {
            if h.intersects(pos, box_, box_pos) {
                return true;
            }
        }
        false
    }

    pub fn collides_tilemap(&self, tilemap: &Tilemap<'_, TILE_SIZE, CELL_AREA>) -> bool {
        if !self.active {
            return false;
        }
        let pos = self.position();
        for h in self.hitboxes.iter().filter(|h| h.active) {
            if h.collides_tilemap(pos, tilemap) {
                return true;
            }
        }
        false
    }
}
