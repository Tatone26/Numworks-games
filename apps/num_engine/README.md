# num-engine

> *A slightly unhinged, zero-allocation 2D game engine built within the brutal hardware constraints of the NumWorks graphing calculator.*

`num-engine` is a bare-metal (`no_std`), deterministic 2D game engine written in Rust for the NumWorks calculator.

Everything, this README included, as been written with the help of generative AI but re-read and optimized by human ideas. Thanks god LLMs don't know what Numworks programming is...

Its main purpose is squeezing stable 40+ FPS performance out of a 216 MHz microcontroller with 256 KB of RAM, zero dynamic heap allocations, and an SPI display bus that punishes naive drawing calls.

---

## Features

- **Zero Runtime Allocations:** Operates entirely in static buffers, fixed-size stack arrays, and slices. No heap allocators, no fragmentation, zero GC pauses.
- **Hardware Burst Fusing:** Groups contiguous runs of dirty vertical tiles into single SPI `push_rect` commands, drastically reducing display command overhead (`CASET`, `RASET`, `RAMWR`).
- **Dynamic Column Interlacing:** Automatically drops alternating columns during sub-pixel horizontal scrolling, cutting CPU compute time in half to prevent missed VSync deadlines.
- **Sub-Screen Viewport Windows:** Decouple world rendering from the physical glass (e.g., render world into a 320×200 box, leaving 40px for a standalone top HUD that never triggers dirty-tile recalculations).
- **Infinite Parallax Tilemaps:** Background layers can be tiny (e.g. 16×4 tiles) and wrap infinitely (`wrap: true`) at fractional speeds (`Parallax::from_distance(3)`).
- ETC (didn't list all, will add a lot more, please see the model for a full example)

---

## Core Concepts

### 1. Viewport vs. World Space

- **World Space ($i32$):** Your entities, tilemaps, and obstacles exist in world coordinates (e.g., $640 \times 480$ or infinite).

- **Screen Window ($u16$):** The rectangle on the physical screen (`screen_x`, `screen_y`, `screen_w`, `screen_h`) where the world is rasterized.
- **UI Space ($i16$):** `UiSprite` coordinates are relative to the viewport window.

### 2. The Frame Lifecycle

To avoid borrow conflicts and maintain correct dirty-rect tracking, follow this exact order in your main game loop:

```rust
loop {
    // 1. Scan inputs
    let keyboard_state = keyboard::scan();

    // 2. Update player & physics (sets velocity, integrates sub-pixels)
    player.set_speed([vx, vy]);
    player.update(engine.get_frame());
    
    // 3. Post-process / clamp player within world bounds
    let clamped_x = player.position[0].clamp(0, MAX_X);
    let clamped_y = player.position[1].clamp(0, MAX_Y);
    player.set_position([clamped_x, clamped_y], None);

    // 4. Update camera (scoped block so borrow drops before render)
    {
        let mut vp = engine.get_mut_viewport();
        vp.follow_deadzone(player.position[0] as i32, player.position[1] as i32, 80, 50);
        vp.clamp(0, WORLD_WIDTH_PX, 0, WORLD_HEIGHT_PX);
    }

    // 5. Build render list
    let mut render_list: Vec<GameRenderable, 16> = Vec::new();
    let _ = render_list.push(Renderable::Tilemap(&bg_map));
    let _ = render_list.push(Renderable::Tilemap(&cloud_map));
    let _ = render_list.push(Renderable::Sprite(&player));

    // 6. Flush to hardware
    let scratch = unsafe { &mut *(&raw mut SCRATCH_BUFFER) };
    engine.render_frame(&mut render_list, scratch);

    // 7. Commit state for dirty rect tracking next frame
    player.commit_frame();
}
