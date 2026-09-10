//! # NumEngine (`num_engine`)
//!
//! A zero-allocation, deterministic 2D game engine written in `no_std` Rust for the
//! NumWorks graphing calculator (STM32F730V8 / ST7789 display controller).
//!
//! ## Key Architectural Features
//!
//! * **Sub-Screen Viewport Windows:** Decouple world rendering from physical screen boundaries.
//!   Confining game rendering to a sub-rectangle (e.g., 320x200) prevents dirty-tile invalidations
//!   and SPI transfer costs from bleeding into dedicated HUD and text areas.
//! * **Hardware SPI Burst Fusing:** Groups contiguous vertical spans of dirty cells into single
//!   `push_rect` commands, minimizing ST7789 command overhead (`CASET`, `RASET`, `RAMWR`).
//! * **Dynamic Column Interlacing:** Automatically drops alternating columns during sub-pixel
//!   horizontal camera panning to halve CPU calculation load and maintain $\ge 40\text{ FPS}$.
//! * **Zero-Copy Parallax Layers:** Backed by independent dimensions, offsets, and rational
//!   scroll ratios (`Parallax`). Background ribbons can repeat endlessly with `wrap: true`
//!   using tiny in-memory slices rather than full-world allocations.
//! * **Opaque Background Fast-Path:** Automatically bypasses `buffer.fill(clear_color)` when
//!   the bottom-most tilemap is marked `transparent = false`.
//!
//! ## Standard Execution Lifecycle
//!
//! ```text
//! 1. Input Handling       (keyboard::scan())
//! 2. Viewport Navigation  (viewport.follow_deadzone(...), viewport.clamp(...))
//! 3. Sprite Update        (sprite.set_speed(...), sprite.update(frame))
//! 4. Physics & Clamping   (sprite.set_position(...))
//! 5. Render Assembly      (render_list.push(Renderable::...))
//! 6. Engine Render        (engine.render_frame(&mut render_list, scratch))
//! 7. State Commit         (sprite.commit_frame(), ui_sprite.commit_frame())
//! 8. Direct HUD Overlay   (draw_string(...))
//! ```

#![no_std]
#![no_main]

pub mod debug;
pub mod engine;
pub mod graphics;
pub mod physics;
pub mod world;

#[macro_use]
pub mod macros;

pub use engine::Engine;
