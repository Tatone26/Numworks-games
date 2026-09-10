# NumEngine

A lightweight 2D game engine built specifically for the Numworks calculator.

The goal was principally to make graphical development easier by astracting it to a very optimized and smart renderer system. It expanded to also contain some useful game engine-like structures and functions : unified scene entity management, tilemaps, bodies, etc.

It is based on the same Tileset system as the more basic graphical utilities you can find in my other folder, numworks_utiles.

> Please understand that, being in 2026, the code wasn't written by a human. I still spent a lot of time optimising it myself, pitching ideas and correcting the AI's code. This is NOT an agentic production.
> Some code is then pretty bad looking and not commented. That wouldn't have been different if I made it myself...

--

## Highlights

* **Zero Dynamic Allocation**: Pools, buffers, and tile grids live in fixed compile-time storage, as the calculator needs.
* **Smart Grid Drawing System**: Re-draws only areas of the screen where objects move or animate, keeping good framerates without any work.
* **Unified Composite Entities**: A single `Entity` can bind kinematic physics, multiple visual parts (using local offsets), and multiple hitboxes.
* **Automatic Scene Pacing**: `world.update()` moves all kinematic bodies, advances animation frames, steps tilemap scroll speeds, and syncs sprite positions in one line.

---

## Minimal Example

A complete game loop skeleton with a player entity and a scrolling background.

Some stuff needs to be defined, like MY_TILESET and PLAYER_ANIM.

Refer to flappy bird for some better example of game engine usage !

```rust
use num_engine::{
    define_scratch_buffer, define_tilemap_buffer,
    init_engine_window, inset_hitbox,
    world_fill_tilemap, world_spawn,
    graphics::{compositor::InterlaceMode, tilemap::{Parallax, WrapMode}, Tilemap},
    world::World,
    Engine,
};
use numworks_utils::eadk::{Color, keyboard, key};

// 1. Define game metrics
const TILE_SIZE: usize = 20;
const CELL_AREA: usize = 400;   // TILE_SIZE * TILE_SIZE
const COLS: usize = 14;         // Screen width / TILE_SIZE
const ROWS: usize = 11;         // Screen height / TILE_SIZE

const ENTITY_CAPACITY: usize = 16;
const TILEMAP_CAPACITY: usize = 2;

// 2. Allocate static memory
define_scratch_buffer!(SCRATCH, CELL_AREA, COLS, ROWS);
define_tilemap_buffer!(BG_DATA, COLS, ROWS);

pub fn run() {
    // 3. Initialize engine window & scene world
    let mut engine = init_engine_window!(
        color: Color::from_rgb888(130, 200, 255),
        window: [20, 20, 280, 220],
        interlace: Some(InterlaceMode::None),
        scratch: SCRATCH,
    );
    let mut world: World<'_, TILE_SIZE, CELL_AREA, ENTITY_CAPACITY, TILEMAP_CAPACITY> = World::new();

    // 4. Set up a background tilemap
    let _bg_id = world_fill_tilemap!(
        world: world,
        tileset: &MY_TILESET,
        buffer: BG_DATA,
        cols: COLS,
        rows: ROWS,
        z: 0,
        tile: [0, 0],
        parallax: Parallax::FIXED,
        wrap: WrapMode::None,
    );

    // 5. Spawn an entity
    let player = world_spawn!(
        world: world,
        anim: &PLAYER_ANIM,
        pos: [60.0, 100.0],
        z: 10,
        hitbox: inset_hitbox!(20, 20, 2),
    ).unwrap();

    // 6. Game Loop
    loop {
        let keys = keyboard::scan();
        let frame = engine.get_frame();

        // Input & Logic
        if keys.key_down(key::UP) {
            world[player].set_vy(-5.0);
        }
        world[player].set_vy(world[player].vy() + 0.4); // Gravity

        // Step world simulation
        world.update(frame);

        // Draw active frame
        engine.render(&mut world);
    }
}

```

---

## Macro Cheat Sheet

These are the core macros you will use in almost every game:

### 1. Memory & Engine Setup

* `define_scratch_buffer!(NAME, cell_area, cols, rows)`
Declares the static rasterization staging buffer required by the renderer.
* `define_tilemap_buffer!(NAME, cols, rows)`
Allocates a static tile array for a tilemap layer.
* `init_engine_window!(color: ..., window: [x, y, w, h], interlace: ..., scratch: BUFFER)`
Initializes the engine bound to a sub-screen window.

### 2. Assets & Animation

* `define_anim!(VIS NAME, &tileset, w, h, transparent, speed, [(tx, ty), ...])`
Builds a static animation track from tile coordinates. **All textures are considered animations, even static ones ; they just have a single frame.**
* `define_repeating_anim!(VIS NAME, &tileset, w, h, sheet_w, sheet_h, transparent, speed, [(tx, ty)])`
Creates large composite sprites by repeating a smaller tile pattern.

### 3. Tilemaps

* `world_tilemap!(world: ..., tileset: ..., buffer: BUF, cols: ..., rows: ..., offset: [x, y], z: ..., transparent: ..., parallax: ..., wrap: ...)`
Constructs and registers a custom tilemap into the world at an optional physical `offset` (world position).
* `world_fill_tilemap!(world: ..., tileset: ..., buffer: BUF, cols: ..., rows: ..., z: ..., tile: [tx, ty], parallax: ..., wrap: ...)`
Fills a static buffer with a uniform tile pattern and registers it into the world. Useful for backgrounds.
* `tile_span!(map, row: r, cols: c_start..c_end, tile: [tx, ty])`
Quickly sets a horizontal row of tiles in a registered (already created) tilemap.

### 4. Spawning & Collision

* `world_spawn!(world: ..., anim: &ANIM, pos: [x, y], z: z_index, hitbox: ...)`
Builds an entity with kinematics, graphics, and collision, and registers it into the scene pool in one call.
* `hitbox!(ox, oy, width, height)`
Creates an explicit axis-aligned bounding box.
* `inset_hitbox!(width, height, padding)`
Generates a collision box smaller than the sprite bounds for forgiving hit detection.

---

## Technical Architecture

* **Coordinate System & Layer Offsets**: `Tilemap` layers have their own physical world `offset: [i16; 2]` (where the layer sits) and `origin: (i32, i32)` (how the tiles scroll within that layer).
* **Reverse-Z Drawing**: The engine blits visible cells front-to-back using a local coverage bitmask. Opaque foreground pixels write immediately and prevent any background pixels behind them from computing, saving memory bandwidth.
* **Auto-Interlacing**: On frames with heavy draw loads, the engine can automatically interlace columns or rows to maintain the target framerate (40 FPS normally).
* **Zero Clone Policy**: Entity and sprite structs rely on references to static ROM data and lightweight integer indices, meaning scene updates never trigger stack-allocated deep copies.

> and a lot more optimisations to try to get the best out of our great calculator !

*PS: you can't imagine how slow the bus is compared to the processor... it feels like working on NES, but without the assembly optimisations power.*
