//! Top-level engine orchestration, frame-pacing, and dirty-boundary tracking.

use numworks_utils::{
    eadk::{
        display::{push_rect_uniform, wait_for_vblank},
        timing, Color, Rect,
    },
    graphical::fill_screen,
};

use crate::{
    compositor::{Compositor, InterlaceMode, Renderable},
    debug::DebugStats,
    viewport::Viewport,
};

/// Threshold in microseconds for tripping into dynamic interlacing (20.0 ms).
///
/// Leaves approximately 2.2 ms of headroom before the standard 22.2 ms (60 Hz)
/// VBlank boundary, preventing dropped frames caused by keyboard scanning or OS interrupts.
const RENDER_BUDGET_CEILING_US: u32 = 20_000;

/// Normalized full-frame threshold in microseconds for disengaging interlacing (17.5 ms).
///
/// When interlacing is active, the measured frame duration is roughly halved.
/// Normalizing back to the estimated full-frame equivalent (`measured * 2`) and
/// requiring it to drop below 17,500 µs (i.e. `< 8,750 µs` measured) ensures strong
/// hysteresis and avoids rapid mode-flapping.
const RENDER_BUDGET_FLOOR_US: u32 = 17_500;

/// Number of consecutive cool frames required before returning to progressive rendering.
const INTERLACE_COOLDOWN_FRAMES: u8 = 8;

/// Number of initial frames ignored during level startup to let the EMA stabilize.
const WARMUP_FRAMES: u32 = 5;

/// The primary coordinator for rendering, camera movement, and frame pacing.
///
/// Manages the compositor pipeline, dirty-rect tracking across layers, sub-screen viewport
/// boundaries, and closed-loop dynamic interlacing governed by wall-clock hardware benchmarks.
///
/// # Generics
/// * `TILE_SIZE` - Width and height of one square tile in pixels (e.g. `20`).
/// * `CELL_AREA` - Total pixel count of one tile (`TILE_SIZE * TILE_SIZE`).
/// * `SCREEN_COLS` - Total columns contained within the viewport grid.
/// * `SCREEN_ROWS` - Total rows contained within the viewport grid.
/// * `DEBUG` - Compile-time toggle for performance tracking overlays and telemetry.
pub struct Engine<
    const TILE_SIZE: usize,
    const CELL_AREA: usize,
    const SCREEN_COLS: usize,
    const SCREEN_ROWS: usize,
    const DEBUG: bool = false,
> {
    /// Layered cell compositor and dirty bitmask grid.
    compositor: Compositor<TILE_SIZE, CELL_AREA, SCREEN_COLS, SCREEN_ROWS>,
    /// Active camera viewport tracking world and screen window bounds.
    viewport: Viewport,
    /// Monotonically increasing frame sequence counter.
    frame: u32,
    /// On-screen telemetry and frame timing collector.
    debug_stats: DebugStats,
    /// When true, interlacing is dynamically engaged based on render load and sub-pixel scrolling.
    pub auto_interlace: bool,
    /// The target interlacing pattern applied when interlacing is engaged.
    interlace_mode: InterlaceMode,
    /// Exponential Moving Average (EMA) of the rasterization duration in microseconds.
    smoothed_render_us: u32,
    /// Whether adaptive interlacing is currently engaged by the runtime governor.
    is_interlacing_engaged: bool,
    /// Cooldown frame counter preventing rapid flapping when transitioning back to progressive mode.
    cooldown_frames: u8,
}

impl<
        const TILE_SIZE: usize,
        const CELL_AREA: usize,
        const SCREEN_COLS: usize,
        const SCREEN_ROWS: usize,
        const DEBUG: bool,
    > Engine<TILE_SIZE, CELL_AREA, SCREEN_COLS, SCREEN_ROWS, DEBUG>
{
    /// Constructs a full-screen engine instance.
    ///
    /// Clears the physical display glass with `clear_color` and initializes a full-display viewport.
    ///
    /// # Arguments
    /// * `clear_color` - Solid background color to clear the display glass with.
    /// * `interlace_mode` - Optional fallback pattern when dynamic interlacing is engaged.
    ///   Defaults to [`InterlaceMode::Rows`].
    pub fn new(clear_color: Color, interlace_mode: Option<InterlaceMode>) -> Self {
        fill_screen(clear_color);

        Self {
            compositor: Compositor::new(clear_color),
            viewport: Viewport::fixed(),
            frame: 0,
            debug_stats: DebugStats::new(),
            auto_interlace: true,
            interlace_mode: interlace_mode.unwrap_or(InterlaceMode::Rows),
            smoothed_render_us: 10_000,
            is_interlacing_engaged: false,
            cooldown_frames: 0,
        }
    }

    /// Constructs an engine instance constrained to an explicit sub-screen window.
    ///
    /// Clears the designated rectangular area with `clear_color` and limits dirty invalidation,
    /// camera bounds, and rendering strictly to `(screen_x, screen_y, screen_w, screen_h)`.
    ///
    /// # Arguments
    /// * `clear_color` - Background fill color for the window rectangle.
    /// * `screen_x` - Horizontal pixel origin on physical display glass.
    /// * `screen_y` - Vertical pixel origin on physical display glass.
    /// * `screen_w` - Total physical width of the viewport window in pixels.
    /// * `screen_h` - Total physical height of the viewport window in pixels.
    /// * `interlace_mode` - Optional fallback pattern when dynamic interlacing is engaged.
    ///   Defaults to [`InterlaceMode::Rows`].
    pub fn new_with_window(
        clear_color: Color,
        screen_x: u16,
        screen_y: u16,
        screen_w: u16,
        screen_h: u16,
        interlace_mode: Option<InterlaceMode>,
    ) -> Self {
        push_rect_uniform(
            Rect {
                x: screen_x,
                y: screen_y,
                width: screen_w,
                height: screen_h,
            },
            clear_color,
        );

        Self {
            compositor: Compositor::new(clear_color),
            viewport: Viewport::new_window(0, 0, screen_x, screen_y, screen_w, screen_h),
            frame: 0,
            debug_stats: DebugStats::new(),
            auto_interlace: true,
            interlace_mode: interlace_mode.unwrap_or(InterlaceMode::Rows),
            smoothed_render_us: 10_000,
            is_interlacing_engaged: false,
            cooldown_frames: 0,
        }
    }

    /// Explicitly updates the interlacing pattern and controls adaptive mode switching.
    ///
    /// # Arguments
    /// * `mode` - Interlacing pattern to apply when engaged ([`InterlaceMode::Rows`],
    ///   [`InterlaceMode::Columns`], or [`InterlaceMode::Checkerboard`]).
    /// * `dynamic_adaptation` - When `true`, automatically engages `mode` only when sub-pixel
    ///   scrolling or when frame render benchmarks exceed budget thresholds. When `false`,
    ///   locks the engine permanently into `mode`.
    #[inline(always)]
    pub fn set_interlacing(&mut self, mode: InterlaceMode, dynamic_adaptation: bool) {
        self.interlace_mode = mode;
        self.auto_interlace = dynamic_adaptation;
    }

    /// Returns a mutable reference to the camera viewport for positioning and scrolling.
    #[inline(always)]
    pub fn get_mut_viewport(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    /// Returns an immutable reference to the active camera viewport.
    #[inline(always)]
    pub fn get_viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Returns the current monotonic frame sequence counter.
    #[inline(always)]
    pub fn get_frame(&self) -> u32 {
        self.frame
    }

    /// Flags the entire cell grid as dirty, forcing every tile on screen to re-render next frame.
    ///
    /// Useful after unpausing, restoring from full-screen dialogs, or following scene transitions.
    pub fn mark_all_dirty(&mut self) {
        self.compositor.grid.mark_all();
    }

    /// Resolves the active interlacing pattern for the current frame using closed-loop metrics.
    ///
    /// Evaluates camera sub-pixel offsets and the Exponential Moving Average of measured
    /// hardware render times against [`RENDER_BUDGET_CEILING_US`] and [`RENDER_BUDGET_FLOOR_US`].
    #[inline(always)]
    fn resolve_interlace_mode(&mut self, viewport_moved: bool, sub_x: i32) -> InterlaceMode {
        if self.interlace_mode == InterlaceMode::None {
            return InterlaceMode::None;
        }

        if !self.auto_interlace {
            return self.interlace_mode;
        }

        // Sub-pixel hardware scrolling trigger: always interlace while camera glides horizontally
        if viewport_moved && sub_x != 0 {
            return self.interlace_mode;
        }

        // Warmup guard: ignore early pipeline initialization spikes
        if self.frame < WARMUP_FRAMES {
            return InterlaceMode::None;
        }

        // Closed-loop runtime budget evaluation
        if self.is_interlacing_engaged {
            // When already interlacing, measured render duration drops by ~50%.
            // Normalize duration back to estimated full-frame equivalent before evaluating release:
            let estimated_full_frame_us = self.smoothed_render_us.saturating_mul(2);

            if estimated_full_frame_us < RENDER_BUDGET_FLOOR_US {
                if self.cooldown_frames == 0 {
                    self.is_interlacing_engaged = false;
                } else {
                    self.cooldown_frames -= 1;
                }
            } else {
                self.cooldown_frames = INTERLACE_COOLDOWN_FRAMES;
            }
        } else if self.smoothed_render_us >= RENDER_BUDGET_CEILING_US {
            self.is_interlacing_engaged = true;
            self.cooldown_frames = INTERLACE_COOLDOWN_FRAMES;
        }

        if self.is_interlacing_engaged {
            self.interlace_mode
        } else {
            InterlaceMode::None
        }
    }

    /// Renders all registered visual entities, synchronizes to VBlank, and tracks performance metrics.
    ///
    /// Calculates dirty rectangles for moving entities, determines whether interlacing is required,
    /// executes the compositing pipeline, measures wall-clock duration, updates moving averages,
    /// and advances internal state counters.
    ///
    /// # Arguments
    /// * `items` - Slice of renderable entities (tilemaps, sprites, UI sprites) to composite.
    /// * `scratch` - Fast scratch pixel buffer utilized during compositing and line assembly.
    pub fn render_frame<'a>(
        &mut self,
        items: &mut [Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
        scratch: &mut [Color],
    ) {
        let viewport_moved = self.viewport.moved();
        let sub_x = self.viewport.x.rem_euclid(TILE_SIZE as i32);

        // 1. Invalidate moving entities and viewports
        if viewport_moved {
            self.compositor.grid.mark_all();
        } else {
            for item in items.iter() {
                match item {
                    Renderable::Sprite(s) => {
                        if s.moved {
                            let (px, py) = self.viewport.world_to_screen(
                                s.prev_position[0] as i32,
                                s.prev_position[1] as i32,
                            );
                            let (cx, cy) = self
                                .viewport
                                .world_to_screen(s.position[0] as i32, s.position[1] as i32);

                            self.mark_screen_rect(px, py, s.pixel_width(), s.pixel_height());
                            self.mark_screen_rect(cx, cy, s.pixel_width(), s.pixel_height());
                        }
                    }
                    Renderable::UiSprite(s) => {
                        if s.moved {
                            self.mark_screen_rect(
                                s.prev_position[0],
                                s.prev_position[1],
                                s.pixel_width(),
                                s.pixel_height(),
                            );
                            self.mark_screen_rect(
                                s.position[0],
                                s.position[1],
                                s.pixel_width(),
                                s.pixel_height(),
                            );
                        }
                    }
                    Renderable::Tilemap(tm) => {
                        tm.mark_animated_dirty(
                            self.frame,
                            &mut self.compositor.grid,
                            self.viewport.x,
                            self.viewport.y,
                        );
                    }
                }
            }
        }

        // 2. Select interlacing strategy using closed-loop metrics
        let active_interlace = self.resolve_interlace_mode(viewport_moved, sub_x);

        if DEBUG {
            self.debug_stats.tick();
            self.compositor.grid.mark_rect(Rect {
                x: 0,
                y: 0,
                width: 90,
                height: 16,
            });
        }

        // 3. Hardware synchronization and timing capture
        wait_for_vblank();
        let t_start = timing::millis();

        self.compositor.render(
            items,
            &self.viewport,
            scratch,
            viewport_moved,
            active_interlace,
        );

        let t_end = timing::millis();
        let elapsed_ms = t_end.saturating_sub(t_start);
        let frame_duration_us = (elapsed_ms as u32).saturating_mul(1000);

        // 4. Update Exponential Moving Average: EMA = (3 * prev + 1 * curr) / 4
        if self.frame >= WARMUP_FRAMES {
            self.smoothed_render_us = ((self.smoothed_render_us * 3) + frame_duration_us) >> 2;
        }

        self.debug_stats.record_render_time(
            frame_duration_us as u64,
            active_interlace != InterlaceMode::None,
        );

        if DEBUG {
            self.debug_stats.draw_overlay();
        }

        // 5. Commit viewport position and advance frame sequence
        self.viewport.commit_frame();
        self.frame = self.frame.wrapping_add(1);
        self.compositor.set_frame(self.frame);
    }

    /// Marks a screen-space rectangle as dirty within the active viewport window.
    ///
    /// Clips the coordinates against the viewport boundary before setting bits in the grid.
    #[inline(always)]
    fn mark_screen_rect(&mut self, sx: i16, sy: i16, w: u16, h: u16) {
        let win_x0 = self.viewport.screen_x as i16;
        let win_y0 = self.viewport.screen_y as i16;
        let win_x1 = win_x0 + self.viewport.screen_w as i16;
        let win_y1 = win_y0 + self.viewport.screen_h as i16;

        let x0 = sx.max(win_x0);
        let y0 = sy.max(win_y0);
        let x1 = (sx + w as i16).min(win_x1);
        let y1 = (sy + h as i16).min(win_y1);

        if x1 > x0 && y1 > y0 {
            self.compositor.grid.mark_rect(Rect {
                x: (x0 - win_x0) as u16,
                y: (y0 - win_y0) as u16,
                width: (x1 - x0) as u16,
                height: (y1 - y0) as u16,
            });
        }
    }

    /// Invalidate an entire grid row across the viewport.
    ///
    /// Useful for horizontal scrolling tilemaps (e.g. ground or parallax layers) that
    /// advance horizontally while the viewport camera itself remains stationary.
    ///
    /// # Arguments
    /// * `row` - Zero-indexed row within `0..SCREEN_ROWS`.
    #[inline(always)]
    pub fn mark_row_dirty(&mut self, row: usize) {
        if row < SCREEN_ROWS {
            let full_row_mask = if SCREEN_COLS >= 32 {
                u32::MAX
            } else {
                (1u32 << SCREEN_COLS) - 1
            };
            self.compositor.grid.curr[row] |= full_row_mask;
        }
    }
}
