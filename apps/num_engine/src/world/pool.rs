//! Scene manager holding entities and tilemaps, executing simulation and render dispatch.

use crate::{
    graphics::{compositor::Renderable, tilemap::Tilemap},
    world::Entity,
    Engine,
};
use core::ops::{Index, IndexMut};
use heapless::Vec;

pub struct World<
    'a,
    const TILE_SIZE: usize,
    const CELL_AREA: usize,
    const ENT_CAP: usize,
    const MAP_CAP: usize,
    const PARTS_PER_ENT: usize = 4,
    const R_CAP: usize = 36,
> {
    pub entities: [Entity<'a, TILE_SIZE, CELL_AREA, PARTS_PER_ENT>; ENT_CAP],
    pub tilemaps: [Option<Tilemap<'a, TILE_SIZE, CELL_AREA>>; MAP_CAP],
    active_ent_count: usize,
}

impl<
        'a,
        const TILE_SIZE: usize,
        const CELL_AREA: usize,
        const ENT_CAP: usize,
        const MAP_CAP: usize,
        const PARTS_PER_ENT: usize,
        const R_CAP: usize,
    > World<'a, TILE_SIZE, CELL_AREA, ENT_CAP, MAP_CAP, PARTS_PER_ENT, R_CAP>
{
    pub fn new() -> Self {
        Self {
            entities: core::array::from_fn(|_| Entity::empty()),
            tilemaps: core::array::from_fn(|_| None),
            active_ent_count: 0,
        }
    }

    pub fn add_tilemap(&mut self, map: Tilemap<'a, TILE_SIZE, CELL_AREA>) -> Option<usize> {
        for (i, slot) in self.tilemaps.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(map);
                return Some(i);
            }
        }
        None
    }

    #[inline(always)]
    pub fn tilemap_mut(&mut self, id: usize) -> &mut Tilemap<'a, TILE_SIZE, CELL_AREA> {
        self.tilemaps[id].as_mut().expect("Invalid tilemap index")
    }

    /// Spawns an Entity into the first unallocated slot.
    pub fn spawn(
        &mut self,
        mut entity: Entity<'a, TILE_SIZE, CELL_AREA, PARTS_PER_ENT>,
    ) -> Option<usize> {
        for (i, slot) in self.entities.iter_mut().enumerate() {
            if !slot.allocated {
                entity.allocated = true;
                *slot = entity;
                if i >= self.active_ent_count {
                    self.active_ent_count = i + 1;
                }
                return Some(i);
            }
        }
        None
    }

    /// Marks the slot unallocated and frees it for subsequent spawns.
    #[inline(always)]
    pub fn despawn(&mut self, id: usize) {
        if id < self.active_ent_count {
            self.entities[id].allocated = false;
            self.entities[id].active = false;

            if id + 1 == self.active_ent_count {
                while self.active_ent_count > 0
                    && !self.entities[self.active_ent_count - 1].allocated
                {
                    self.active_ent_count -= 1;
                }
            }
        }
    }

    pub fn update(&mut self, frame: u32) {
        for map_opt in self.tilemaps.iter_mut() {
            if let Some(map) = map_opt.as_mut() {
                map.update();
            }
        }

        for ent in self.entities[..self.active_ent_count].iter_mut() {
            if ent.allocated && ent.active {
                ent.update(frame);
            }
        }
    }

    #[inline(always)]
    pub fn collides(&self, id_a: usize, id_b: usize) -> bool {
        if id_a >= self.active_ent_count || id_b >= self.active_ent_count {
            return false;
        }
        let a = &self.entities[id_a];
        let b = &self.entities[id_b];
        if !a.allocated || !a.active || !b.allocated || !b.active {
            return false;
        }
        a.collides_with(b)
    }

    #[inline(always)]
    pub fn collides_tilemap(&self, ent_id: usize, map_id: usize) -> bool {
        if ent_id >= self.active_ent_count || map_id >= MAP_CAP {
            return false;
        }
        let ent = &self.entities[ent_id];
        if !ent.allocated || !ent.active {
            return false;
        }
        if let Some(ref map) = self.tilemaps[map_id] {
            ent.collides_tilemap(map)
        } else {
            false
        }
    }

    pub fn render_to_engine<const SC: usize, const SR: usize, const DBG: bool>(
        &mut self,
        engine: &mut Engine<'_, TILE_SIZE, CELL_AREA, SC, SR, DBG>,
        progressive: bool,
    ) {
        let mut list: Vec<Renderable<'_, 'a, TILE_SIZE, CELL_AREA>, R_CAP> = Vec::new();

        for map_opt in self.tilemaps.iter_mut() {
            if let Some(map) = map_opt.as_mut() {
                let _ = list.push(Renderable::Tilemap(map));
            }
        }

        for ent in self.entities[..self.active_ent_count].iter_mut() {
            if ent.allocated && ent.active {
                for spr in ent.sprites.iter_mut() {
                    if spr.active {
                        let _ = list.push(Renderable::Sprite(spr));
                    }
                }
            }
        }

        if progressive {
            engine.render_frame_progressive(&mut list);
        } else {
            engine.render_frame(&mut list);
        }
    }
}

impl<
        'a,
        const TILE_SIZE: usize,
        const CELL_AREA: usize,
        const ENT_CAP: usize,
        const MAP_CAP: usize,
        const PARTS_PER_ENT: usize,
        const R_CAP: usize,
    > Index<usize> for World<'a, TILE_SIZE, CELL_AREA, ENT_CAP, MAP_CAP, PARTS_PER_ENT, R_CAP>
{
    type Output = Entity<'a, TILE_SIZE, CELL_AREA, PARTS_PER_ENT>;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.entities[index]
    }
}

impl<
        'a,
        const TILE_SIZE: usize,
        const CELL_AREA: usize,
        const ENT_CAP: usize,
        const MAP_CAP: usize,
        const PARTS_PER_ENT: usize,
        const R_CAP: usize,
    > IndexMut<usize> for World<'a, TILE_SIZE, CELL_AREA, ENT_CAP, MAP_CAP, PARTS_PER_ENT, R_CAP>
{
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entities[index]
    }
}

impl<
        'a,
        const TILE_SIZE: usize,
        const CELL_AREA: usize,
        const ENT_CAP: usize,
        const MAP_CAP: usize,
        const PARTS_PER_ENT: usize,
        const R_CAP: usize,
    > Default for World<'a, TILE_SIZE, CELL_AREA, ENT_CAP, MAP_CAP, PARTS_PER_ENT, R_CAP>
{
    fn default() -> Self {
        Self::new()
    }
}
