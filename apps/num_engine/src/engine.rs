//! Top-level engine orchestration, frame-pacing, and dirty-boundary tracking.

use numworks_utils::{
    eadk::{
        display::{push_rect_uniform, wait_for_vblank},
        timing, Color, Rect,
    },
    graphical::fill_screen,
};

use crate::{
    debug::DebugStats,
    graphics::compositor::{Compositor, InterlaceMode, Renderable},
    graphics::viewport::Viewport,
    world::World,
};

/// Dynamic performance budgeting and sensitivity tuning for auto-interlacing.
#[derive(Clone, Copy, Debug)]
pub struct InterlaceTuning {
    /// Upper threshold before interlacing is triggered (in microseconds).
    /// Default: 21_500 µs (~21.5 ms, targeting 40-45 FPS).
    pub target_budget_us: u32,
    /// Margin below target to exit interlacing (in microseconds).
    /// Full-frame equivalent must drop below (target_budget_us - hysteresis_us) to exit.
    /// Default: 3_500 µs (exits when full frame equivalent is under 18.0 ms).
    pub hysteresis_us: u32,
    /// Consecutive slow frames required to engage interlacing (1 = instant tripwire).
    pub tripwire_frames: u8,
    /// Consecutive healthy frames required before progressive mode is restored.
    pub cooldown_frames: u8,
}

impl Default for InterlaceTuning {
    fn default() -> Self {
        Self {
            target_budget_us: 21_500,
            hysteresis_us: 3_500,
            tripwire_frames: 1,
            cooldown_frames: 5,
        }
    }
}

pub struct Engine<
    's,
    const TILE_SIZE: usize,
    const CELL_AREA: usize,
    const SCREEN_COLS: usize,
    const SCREEN_ROWS: usize,
    const R_CAP: usize = 64,
    const DEBUG: bool = false,
> {
    compositor: Compositor<TILE_SIZE, CELL_AREA, SCREEN_COLS, SCREEN_ROWS, R_CAP>,
    viewport: Viewport,
    scratch: &'s mut [Color],
    frame: u32,
    debug_stats: DebugStats,
    pub auto_interlace: bool,
    interlace_mode: InterlaceMode,
    is_interlacing_engaged: bool,
    tuning: InterlaceTuning,
    slow_streak: u8,
    fast_streak: u8,
    prev_frame_end_ms: u64,
}

impl<
        's,
        const TILE_SIZE: usize,
        const CELL_AREA: usize,
        const SCREEN_COLS: usize,
        const SCREEN_ROWS: usize,
        const R_CAP: usize,
        const DEBUG: bool,
    > Engine<'s, TILE_SIZE, CELL_AREA, SCREEN_COLS, SCREEN_ROWS, R_CAP, DEBUG>
{
    pub fn new(
        clear_color: Color,
        interlace_mode: Option<InterlaceMode>,
        scratch: &'s mut [Color],
    ) -> Self {
        fill_screen(clear_color);

        Self {
            compositor: Compositor::new(clear_color),
            viewport: Viewport::fixed(),
            scratch,
            frame: 0,
            debug_stats: DebugStats::new(),
            auto_interlace: true,
            interlace_mode: interlace_mode.unwrap_or(InterlaceMode::Checkerboard),
            is_interlacing_engaged: false,
            tuning: InterlaceTuning::default(),
            slow_streak: 0,
            fast_streak: 0,
            prev_frame_end_ms: 0,
        }
    }

    pub fn new_with_window(
        clear_color: Color,
        screen_x: u16,
        screen_y: u16,
        screen_w: u16,
        screen_h: u16,
        interlace_mode: Option<InterlaceMode>,
        scratch: &'s mut [Color],
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
            scratch,
            frame: 0,
            debug_stats: DebugStats::new(),
            auto_interlace: true,
            interlace_mode: interlace_mode.unwrap_or(InterlaceMode::Checkerboard),
            is_interlacing_engaged: false,
            tuning: InterlaceTuning::default(),
            slow_streak: 0,
            fast_streak: 0,
            prev_frame_end_ms: 0,
        }
    }

    #[inline(always)]
    pub fn set_interlace_tuning(&mut self, tuning: InterlaceTuning) {
        self.tuning = tuning;
    }

    #[inline(always)]
    pub fn set_interlacing(&mut self, mode: InterlaceMode, dynamic_adaptation: bool) {
        self.interlace_mode = mode;
        self.auto_interlace = dynamic_adaptation;
        if !dynamic_adaptation {
            self.is_interlacing_engaged = mode != InterlaceMode::None;
        }
    }

    #[inline(always)]
    pub fn render<
        'a,
        const ENT_CAP: usize,
        const MAP_CAP: usize,
        const PARTS: usize,
        const PARTICLE_CAP: usize,
        const PARTICLE_SYS_CAP: usize,
    >(
        &mut self,
        world: &mut World<
            'a,
            TILE_SIZE,
            CELL_AREA,
            ENT_CAP,
            MAP_CAP,
            PARTS,
            R_CAP,
            PARTICLE_CAP,
            PARTICLE_SYS_CAP,
        >,
    ) {
        world.render_to_engine(self, false);
    }

    #[inline(always)]
    pub fn render_progressive<
        'a,
        const ENT_CAP: usize,
        const MAP_CAP: usize,
        const PARTS: usize,
        const PARTICLE_CAP: usize,
        const PARTICLE_SYS_CAP: usize,
    >(
        &mut self,
        world: &mut World<
            'a,
            TILE_SIZE,
            CELL_AREA,
            ENT_CAP,
            MAP_CAP,
            PARTS,
            R_CAP,
            PARTICLE_CAP,
            PARTICLE_SYS_CAP,
        >,
    ) {
        world.render_to_engine(self, true);
    }

    #[inline(always)]
    pub fn get_mut_viewport(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    #[inline(always)]
    pub fn get_viewport(&self) -> &Viewport {
        &self.viewport
    }

    #[inline(always)]
    pub fn get_frame(&self) -> u32 {
        self.frame
    }

    pub fn mark_all_dirty(&mut self) {
        self.compositor.grid.mark_all();
    }

    #[inline(always)]
    fn is_item_visible<'r, 'a>(
        &self,
        item: &Renderable<'r, 'a, TILE_SIZE, CELL_AREA>,
        win_x0: i16,
        win_y0: i16,
        win_x1: i16,
        win_y1: i16,
    ) -> bool {
        match item {
            Renderable::Tilemap(_) => true,
            Renderable::Particles(ps) => {
                ps.is_visible(win_x0, win_y0, win_x1, win_y1, &self.viewport)
            }
            Renderable::Sprite(s) => {
                let w = s.pixel_width() as i16;
                let h = s.pixel_height() as i16;

                let (cx, cy) = self
                    .viewport
                    .world_to_screen(s.position[0] as i32, s.position[1] as i32);
                let curr_visible = cx + w > win_x0 && cx < win_x1 && cy + h > win_y0 && cy < win_y1;

                if curr_visible {
                    return true;
                }

                if s.moved {
                    let (px, py) = self
                        .viewport
                        .world_to_screen(s.prev_position[0] as i32, s.prev_position[1] as i32);
                    px + w > win_x0 && px < win_x1 && py + h > win_y0 && py < win_y1
                } else {
                    false
                }
            }
            Renderable::UiSprite(s) => {
                let w = s.pixel_width() as i16;
                let h = s.pixel_height() as i16;
                let local_w = win_x1 - win_x0;
                let local_h = win_y1 - win_y0;

                let cx = s.position[0];
                let cy = s.position[1];
                let curr_visible = cx + w > 0 && cx < local_w && cy + h > 0 && cy < local_h;

                if curr_visible {
                    return true;
                }

                if s.moved {
                    let px = s.prev_position[0];
                    let py = s.prev_position[1];
                    px + w > 0 && px < local_w && py + h > 0 && py < local_h
                } else {
                    false
                }
            }
        }
    }

    #[inline(always)]
    fn cull_offscreen_items<'r, 'a>(
        &self,
        items: &mut [Renderable<'r, 'a, TILE_SIZE, CELL_AREA>],
    ) -> usize {
        let win_x0 = self.viewport.screen_x as i16;
        let win_y0 = self.viewport.screen_y as i16;
        let win_x1 = win_x0 + self.viewport.screen_w as i16;
        let win_y1 = win_y0 + self.viewport.screen_h as i16;

        let mut next_visible = 0;
        for i in 0..items.len() {
            if self.is_item_visible(&items[i], win_x0, win_y0, win_x1, win_y1) {
                if i != next_visible {
                    items.swap(i, next_visible);
                }
                next_visible += 1;
            }
        }
        next_visible
    }

    #[inline(always)]
    fn resolve_interlace_mode(&self, viewport_moved: bool, sub_x: i32) -> InterlaceMode {
        if self.interlace_mode == InterlaceMode::None {
            return InterlaceMode::None;
        }

        if !self.auto_interlace {
            return self.interlace_mode;
        }

        // Sub-pixel camera scrolling requires interlacing to prevent diagonal tearing
        if viewport_moved && sub_x != 0 {
            return self.interlace_mode;
        }

        if self.is_interlacing_engaged {
            self.interlace_mode
        } else {
            InterlaceMode::None
        }
    }

    pub fn render_frame<'a>(&mut self, items: &mut [Renderable<'_, 'a, TILE_SIZE, CELL_AREA>]) {
        let viewport_moved = self.viewport.moved();
        let sub_x = self.viewport.x.rem_euclid(TILE_SIZE as i32);
        let active_interlace = self.resolve_interlace_mode(viewport_moved, sub_x);

        self.render_pipeline(items, viewport_moved, active_interlace);
    }

    pub fn render_frame_progressive<'a>(
        &mut self,
        items: &mut [Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
    ) {
        let viewport_moved = self.viewport.moved();
        self.render_pipeline(items, viewport_moved, InterlaceMode::None);
    }

    fn render_pipeline<'a>(
        &mut self,
        items: &mut [Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
        viewport_moved: bool,
        active_interlace: InterlaceMode,
    ) {
        let visible_count = self.cull_offscreen_items(items);

        if viewport_moved {
            self.compositor.grid.mark_all();
        } else {
            for item in items[..visible_count].iter_mut() {
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
                        tm.mark_sparse_dirty(
                            self.frame,
                            &mut self.compositor.grid,
                            self.viewport.x,
                            self.viewport.y,
                        );
                    }
                    Renderable::Particles(ps) => {
                        let vp = self.viewport;
                        let grid = &mut self.compositor.grid;
                        ps.mark_dirty(&vp, &mut |r| {
                            grid.mark_rect(r);
                        });
                    }
                }
            }
        }

        if DEBUG {
            self.debug_stats.tick();
            self.compositor.grid.mark_rect(Rect {
                x: 0,
                y: 0,
                width: 120,
                height: 16,
            });
        }

        wait_for_vblank();
        let t_render_start = timing::millis();

        let scratch = &mut *self.scratch;
        self.compositor.render(
            &mut items[..visible_count],
            &self.viewport,
            scratch,
            viewport_moved,
            active_interlace,
        );

        let t_now = timing::millis();
        let render_duration_ms = t_now.saturating_sub(t_render_start);
        let render_duration_us = (render_duration_ms as u32).saturating_mul(1000);

        let total_frame_duration_us = if self.prev_frame_end_ms > 0 {
            (t_now.saturating_sub(self.prev_frame_end_ms) as u32).saturating_mul(1000)
        } else {
            render_duration_us
        };
        self.prev_frame_end_ms = t_now;

        // Dynamic State Machine Transition: Fast Tripwire & Safe Exit
        if self.auto_interlace {
            if self.is_interlacing_engaged {
                // In interlaced mode, cost is ~half. Double it to estimate full progressive cost.
                let est_full_frame = render_duration_us.saturating_mul(2);
                let exit_ceiling = self
                    .tuning
                    .target_budget_us
                    .saturating_sub(self.tuning.hysteresis_us);

                if est_full_frame <= exit_ceiling {
                    self.fast_streak += 1;
                    if self.fast_streak >= self.tuning.cooldown_frames {
                        self.is_interlacing_engaged = false;
                        self.fast_streak = 0;
                        self.slow_streak = 0;
                    }
                } else {
                    self.fast_streak = 0;
                }
            } else if render_duration_us > self.tuning.target_budget_us {
                self.slow_streak += 1;
                if self.slow_streak >= self.tuning.tripwire_frames {
                    self.is_interlacing_engaged = true;
                    self.slow_streak = 0;
                    self.fast_streak = 0;
                }
            } else {
                self.slow_streak = 0;
            }
        }

        self.debug_stats.record_metrics(
            render_duration_us as u64,
            total_frame_duration_us as u64,
            active_interlace != InterlaceMode::None,
        );

        if DEBUG {
            self.debug_stats.draw_overlay();
        }

        self.viewport.commit_frame();
        self.frame = self.frame.wrapping_add(1);
        self.compositor.set_frame(self.frame);

        for item in items.iter_mut() {
            item.commit_frame();
        }
    }

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
