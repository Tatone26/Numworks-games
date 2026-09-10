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

/// Target budget ceiling for 40-45 Hz display hardware (~22.2-25.0 ms period).
/// Leaves 5.5 ms of headroom for game logic and keyboard scanning before VBlank.
const RENDER_BUDGET_CEILING_US: u32 = 19_500;

/// Normalized full-frame threshold for disengaging interlacing.
/// Requiring estimated full-frame duration (measured * 2) < 16_000 us ensures stable hysteresis.
const RENDER_BUDGET_FLOOR_US: u32 = 16_000;

/// Number of consecutive cool frames required before returning to progressive rendering.
const INTERLACE_COOLDOWN_FRAMES: u8 = 10;

/// Number of initial frames ignored during level startup to let the pipeline stabilize.
const WARMUP_FRAMES: u32 = 5;

pub struct Engine<
    const TILE_SIZE: usize,
    const CELL_AREA: usize,
    const SCREEN_COLS: usize,
    const SCREEN_ROWS: usize,
    const DEBUG: bool = false,
> {
    compositor: Compositor<TILE_SIZE, CELL_AREA, SCREEN_COLS, SCREEN_ROWS>,
    viewport: Viewport,
    frame: u32,
    debug_stats: DebugStats,
    pub auto_interlace: bool,
    interlace_mode: InterlaceMode,
    smoothed_render_us: u32,
    is_interlacing_engaged: bool,
    cooldown_frames: u8,
    /// Hardware timestamp in milliseconds of the previous frame completion.
    prev_frame_end_ms: u64,
}

impl<
        const TILE_SIZE: usize,
        const CELL_AREA: usize,
        const SCREEN_COLS: usize,
        const SCREEN_ROWS: usize,
        const DEBUG: bool,
    > Engine<TILE_SIZE, CELL_AREA, SCREEN_COLS, SCREEN_ROWS, DEBUG>
{
    pub fn new(clear_color: Color, interlace_mode: Option<InterlaceMode>) -> Self {
        fill_screen(clear_color);

        Self {
            compositor: Compositor::new(clear_color),
            viewport: Viewport::fixed(),
            frame: 0,
            debug_stats: DebugStats::new(),
            auto_interlace: true,
            interlace_mode: interlace_mode.unwrap_or(InterlaceMode::Columns),
            smoothed_render_us: 10_000,
            is_interlacing_engaged: false,
            cooldown_frames: 0,
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
            interlace_mode: interlace_mode.unwrap_or(InterlaceMode::Columns),
            smoothed_render_us: 10_000,
            is_interlacing_engaged: false,
            cooldown_frames: 0,
            prev_frame_end_ms: 0,
        }
    }

    #[inline(always)]
    pub fn set_interlacing(&mut self, mode: InterlaceMode, dynamic_adaptation: bool) {
        self.interlace_mode = mode;
        self.auto_interlace = dynamic_adaptation;
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

        if self.frame < WARMUP_FRAMES {
            return InterlaceMode::None;
        }

        if self.is_interlacing_engaged {
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

    #[inline(always)]
    pub fn render_frame<'a>(
        &mut self,
        items: &mut [Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
        scratch: &mut [Color],
    ) {
        let viewport_moved = self.viewport.moved();
        let sub_x = self.viewport.x.rem_euclid(TILE_SIZE as i32);
        let active_interlace = self.resolve_interlace_mode(viewport_moved, sub_x);

        self.render_pipeline(items, scratch, viewport_moved, active_interlace);
    }

    #[inline(always)]
    pub fn render_frame_progressive<'a>(
        &mut self,
        items: &mut [Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
        scratch: &mut [Color],
    ) {
        let viewport_moved = self.viewport.moved();
        self.render_pipeline(items, scratch, viewport_moved, InterlaceMode::None);
    }

    pub fn render_frame_full<'a>(
        &mut self,
        items: &mut [Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
        scratch: &mut [Color],
    ) {
        self.mark_all_dirty();
        self.render_pipeline(items, scratch, true, InterlaceMode::None);
    }

    fn render_pipeline<'a>(
        &mut self,
        items: &mut [Renderable<'_, 'a, TILE_SIZE, CELL_AREA>],
        scratch: &mut [Color],
        viewport_moved: bool,
        active_interlace: InterlaceMode,
    ) {
        // 1. In-Place Frustum Culling
        let visible_count = self.cull_offscreen_items(items);

        // 2. Invalidate moving entities and viewports
        if viewport_moved {
            self.compositor.grid.mark_all();
        } else {
            for item in items[..visible_count].iter() {
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

        // 3. Hardware synchronization and timing capture
        wait_for_vblank();
        let t_render_start = timing::millis();

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

        // Total frame duration measured from previous completion (Logic + VBlank Wait + Render)
        let total_frame_duration_us = if self.prev_frame_end_ms > 0 {
            (t_now.saturating_sub(self.prev_frame_end_ms) as u32).saturating_mul(1000)
        } else {
            render_duration_us
        };
        self.prev_frame_end_ms = t_now;

        // 4. Update Exponential Moving Average: EMA = (3 * prev + 1 * curr) / 4
        if self.frame >= WARMUP_FRAMES {
            self.smoothed_render_us = ((self.smoothed_render_us * 3) + render_duration_us) >> 2;
        }

        self.debug_stats.record_metrics(
            render_duration_us as u64,
            total_frame_duration_us as u64,
            active_interlace != InterlaceMode::None,
        );

        if DEBUG {
            self.debug_stats.draw_overlay();
        }

        // 5. Commit viewport position and advance frame sequence
        self.viewport.commit_frame();
        self.frame = self.frame.wrapping_add(1);
        self.compositor.set_frame(self.frame);

        // 6. Automatic commit across all items submitted to the frame
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
