//! Declarative macros for entity composition, scene allocation, and layer registration.

/// Unified Entity constructor: builds an independent composite Entity.
#[macro_export]
macro_rules! spawn_entity {
    // 0. Tile-grid aligned entry point (forwards to coordinate branch)
    (tile_size: $ts:expr, tile: ($tx:expr, $ty:expr), $($rest:tt)*) => {{
        let px = ($tx as i16) * ($ts as i16);
        let py = ($ty as i16) * ($ts as i16);
        $crate::spawn_entity!(pos: [px as f32, py as f32], $($rest)*)
    }};

    // 1. Full composite: Body + Sprite + Hitbox
    (anim: $anim:expr, pos: [$x:expr, $y:expr], z: $z:expr, hitbox: $hit:expr $(,)?) => {{
        let px = $x as f32;
        let py = $y as f32;
        let int_pos = [px as i16, py as i16];
        $crate::world::Entity::new()
            .with_body($crate::physics::Body::new(px, py))
            .with_sprite($crate::spawn_sprite!($anim, int_pos, $z))
            .with_hitbox($hit)
    }};

    // 2. Kinematic: Body + Sprite only
    (anim: $anim:expr, pos: [$x:expr, $y:expr], z: $z:expr $(,)?) => {{
        let px = $x as f32;
        let py = $y as f32;
        let int_pos = [px as i16, py as i16];
        $crate::world::Entity::new()
            .with_body($crate::physics::Body::new(px, py))
            .with_sprite($crate::spawn_sprite!($anim, int_pos, $z))
    }};

    // 3. Static Graphic: Sprite only
    (static_anim: $anim:expr, pos: [$x:expr, $y:expr], z: $z:expr $(,)?) => {{
        let int_pos = [$x as i16, $y as i16];
        $crate::world::Entity::new().with_sprite($crate::spawn_sprite!($anim, int_pos, $z))
    }};

    // 4. Sensor/Trigger: Body + Hitbox only
    (trigger_pos: [$x:expr, $y:expr], hitbox: $hit:expr $(,)?) => {{
        let px = $x as f32;
        let py = $y as f32;
        $crate::world::Entity::new()
            .with_body($crate::physics::Body::new(px, py))
            .with_hitbox($hit)
    }};
}

/// Spawns an entity directly into the World.
///
/// Uses macro delegation: accepts pre-built entities OR any argument pattern valid for `spawn_entity!`.
#[macro_export]
macro_rules! world_spawn {
    // Branch A: Pre-constructed Entity instance
    (world: $world:expr, entity: $ent:expr $(,)?) => {
        $world.spawn($ent)
    };

    // Branch B: Macro-in-macro delegation to spawn_entity!
    (world: $world:expr, $($rest:tt)*) => {
        $world.spawn($crate::spawn_entity!($($rest)*))
    };
}

/// Registers a static tile buffer as a Tilemap layer into World.
#[macro_export]
macro_rules! world_tilemap {
    (
        world: $world:expr,
        tileset: $tileset:expr,
        buffer: $buf:ident,
        cols: $cols:expr,
        rows: $rows:expr,
        $(offset: [$ox:expr, $oy:expr],)?
        z: $z:expr,
        transparent: $trans:expr,
        parallax: $parallax:expr,
        wrap: $wrap:expr $(,)?
    ) => {{
        let buf_ref = unsafe { &mut *(&raw mut $buf) };
        let mut map = $crate::graphics::Tilemap::new(
            $tileset,
            buf_ref,
            $cols,
            $rows,
            &[],
            $z,
            $trans,
            $parallax,
            $wrap,
        );
        $(
            map.set_offset([$ox as i16, $oy as i16]);
        )?
        $world.add_tilemap(map)
    }};
}

/// Fills a static buffer with a uniform tile and registers it into World.
#[macro_export]
macro_rules! world_fill_tilemap {
    (
        world: $world:expr,
        tileset: $tileset:expr,
        buffer: $buf:ident,
        cols: $cols:expr,
        rows: $rows:expr,
        $(offset: [$ox:expr, $oy:expr],)?
        z: $z:expr,
        tile: $tile:expr,
        parallax: $parallax:expr,
        wrap: $wrap:expr $(,)?
    ) => {{
        let buf_ref = unsafe { &mut *(&raw mut $buf) };
        for slot in buf_ref.iter_mut() {
            *slot = Some($tile);
        }
        let mut map = $crate::graphics::Tilemap::new(
            $tileset,
            buf_ref,
            $cols,
            $rows,
            &[],
            $z,
            false,
            $parallax,
            $wrap,
        );
        $(
            map.set_offset([$ox as i16, $oy as i16]);
        )?
        $world.add_tilemap(map)
    }};
}

/// Parses ASCII art layouts into a static buffer and registers it directly into World.
#[macro_export]
macro_rules! world_ascii_tilemap {
    (
        world: $world:expr,
        tileset: $tileset:expr,
        buffer: $buf:ident,
        cols: $cols:expr,
        rows: $rows:expr,
        $(offset: [$ox:expr, $oy:expr],)?
        z: $z:expr,
        transparent: $trans:expr,
        parallax: $parallax:expr,
        wrap: $wrap:expr,
        mapping: { $( $char:pat => $tile:expr ),* $(,)? },
        layout: $layout:expr $(,)?
    ) => {{
        let buf_ref = unsafe { &mut *(&raw mut $buf) };
        let map = $crate::ascii_tilemap!(
            tileset: $tileset,
            buffer: buf_ref,
            cols: $cols,
            rows: $rows,
            $(offset: [$ox, $oy],)?
            z: $z,
            transparent: $trans,
            parallax: $parallax,
            wrap: $wrap,
            mapping: { $( $char => $tile ),* },
            layout: $layout,
        );
        $world.add_tilemap(map)
    }};
}

/// Instantiates and registers a [`ParticleSystem`](crate::graphics::particles::ParticleSystem) into a World.
#[macro_export]
macro_rules! world_particles {
    // 1. Initialized with a pre-configured auto-emitter (simplified, no z needed)
    (world: $world:expr, kind: $kind:expr, emitter: $emitter:expr $(,)?) => {{
        let mut sys = $crate::graphics::particles::ParticleSystem::new(0, $kind);
        sys.set_emitter($emitter);
        $world.add_particle_system(sys)
    }};

    // 2. Standalone system with kind (simplified, no z needed)
    (world: $world:expr, kind: $kind:expr $(,)?) => {{
        let sys = $crate::graphics::particles::ParticleSystem::new(0, $kind);
        $world.add_particle_system(sys)
    }};

    // 3. Backward-compatible branches accepting legacy z: $z
    (world: $world:expr, z: $z:expr, kind: $kind:expr, emitter: $emitter:expr $(,)?) => {{
        let mut sys = $crate::graphics::particles::ParticleSystem::new($z, $kind);
        sys.set_emitter($emitter);
        $world.add_particle_system(sys)
    }};

    (world: $world:expr, z: $z:expr, kind: $kind:expr $(,)?) => {{
        let sys = $crate::graphics::particles::ParticleSystem::new($z, $kind);
        $world.add_particle_system(sys)
    }};

    (world: $world:expr, z: $z:expr $(,)?) => {{
        let sys = $crate::graphics::particles::ParticleSystem::new(
            $z,
            $crate::graphics::particles::ParticleKind::Pixel,
        );
        $world.add_particle_system(sys)
    }};

    // 4. Fallback default
    (world: $world:expr $(,)?) => {{
        let sys = $crate::graphics::particles::ParticleSystem::new(
            0,
            $crate::graphics::particles::ParticleKind::Pixel,
        );
        $world.add_particle_system(sys)
    }};
}
