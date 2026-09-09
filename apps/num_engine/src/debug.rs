//! Real-time diagnostic metrics, frame pacing monitors, and on-screen HUD profiling.

use numworks_utils::{
    eadk::{timing, Color, Point},
    graphical::{draw_string_cfg, ColorConfig},
    utils::string_from_u16,
};

/// Rolling window diagnostic tracker measuring frame rate and rasterization duration.
///
/// Accumulates hardware millisecond and microsecond metrics over a 500 ms window
/// to produce smoothed average FPS and render timings.
pub(crate) struct DebugStats {
    /// Timestamp in milliseconds of the last metrics window flush.
    last_sample_time: u64,
    /// Total frames elapsed within the active sampling window.
    frame_accumulator: u32,
    /// Total microseconds spent inside `Compositor::render` during the active window.
    render_time_accumulator_us: u64,

    /// Smoothed frames per second calculated over the last 500 ms sample.
    pub current_fps: u16,
    /// Mean duration of the composite pass per frame in milliseconds.
    pub avg_render_ms: u16,
    /// Indicates whether dynamic interlacing (half-column skipping) was active during the last pass.
    pub is_throttled: bool,
}

impl DebugStats {
    /// Creates a zeroed debug tracker.
    pub const fn new() -> Self {
        Self {
            last_sample_time: 0,
            frame_accumulator: 0,
            render_time_accumulator_us: 0,
            current_fps: 0,
            avg_render_ms: 0,
            is_throttled: false,
        }
    }

    /// Registers the elapsed duration of the current frame's compositor pass.
    ///
    /// # Arguments
    /// * `duration_us` - Microseconds spent strictly inside [`Compositor::render`](crate::compositor::Compositor::render).
    /// * `throttled` - `true` if column interlacing dropped half the screen tiles this frame.
    #[inline(always)]
    pub fn record_render_time(&mut self, duration_us: u64, throttled: bool) {
        self.render_time_accumulator_us =
            self.render_time_accumulator_us.saturating_add(duration_us);
        self.is_throttled = throttled;
    }

    // Advances the frame counter and updates metrics averages if 500 ms have elapsed.
    ///
    /// Must be invoked once per game loop tick.
    pub fn tick(&mut self) {
        let now = timing::millis();
        self.frame_accumulator = self.frame_accumulator.saturating_add(1);

        if self.last_sample_time == 0 {
            self.last_sample_time = now;
            return;
        }

        let delta = now.saturating_sub(self.last_sample_time);
        if delta >= 500 {
            self.current_fps = ((self.frame_accumulator as u64 * 1000) / delta) as u16;

            if self.frame_accumulator > 0 {
                // Average render time per frame in milliseconds
                self.avg_render_ms = (self.render_time_accumulator_us
                    / (self.frame_accumulator as u64 * 1000))
                    as u16;
            }

            self.last_sample_time = now;
            self.frame_accumulator = 0;
            self.render_time_accumulator_us = 0;
        }
    }

    /// Paints the debug status badge directly onto the display glass via EADK syscalls.
    ///
    /// Output format: `"<FPS>F <RenderTime>ms"` (e.g., `"40F 8ms"`), appending an asterisk
    /// `*` when hardware interlacing is actively dropping alternating columns.
    pub fn draw_overlay(&self) {
        const BG_COLOR: Color = Color::from_rgb888(30, 30, 30);
        const TEXT_COLOR: Color = Color::from_rgb888(50, 230, 50);
        const WARN_COLOR: Color = Color::from_rgb888(255, 180, 0);

        let cfg = ColorConfig {
            text: if self.is_throttled {
                WARN_COLOR
            } else {
                TEXT_COLOR
            },
            bckgrd: BG_COLOR,
            alt: TEXT_COLOR,
        };

        let mut text = heapless::String::<24>::new();
        // FPS
        let _ = text.push_str(
            string_from_u16(self.current_fps)
                .as_str()
                .trim_matches('\0'),
        );
        let _ = text.push_str("F ");
        // Render work duration in ms
        let _ = text.push_str(
            string_from_u16(self.avg_render_ms)
                .as_str()
                .trim_matches('\0'),
        );
        let _ = text.push_str("ms");
        if self.is_throttled {
            let _ = text.push('*'); // Indicator that half-frame interlacing is active
        }
        let _ = text.push('\0');

        draw_string_cfg(&text, Point::new(4, 3), false, &cfg, false);
    }
}
