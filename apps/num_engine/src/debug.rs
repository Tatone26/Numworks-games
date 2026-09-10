//! Real-time diagnostic metrics, frame pacing monitors, and on-screen HUD profiling.

use numworks_utils::{
    eadk::{timing, Color, Point},
    graphical::{draw_string_cfg, ColorConfig},
    utils::string_from_u16,
};

/// Rolling window diagnostic tracker measuring frame rate, pure rasterization duration,
/// and total end-to-end frame duration.
pub(crate) struct DebugStats {
    /// Timestamp in milliseconds of the last metrics window flush.
    last_sample_time: u64,
    /// Total frames elapsed within the active sampling window.
    frame_accumulator: u32,
    /// Total microseconds spent inside `Compositor::render`.
    render_time_accumulator_us: u64,
    /// Total microseconds elapsed between consecutive frame submissions (Logic + VBlank + Render).
    total_frame_time_accumulator_us: u64,

    /// Smoothed frames per second calculated over the last 500 ms sample.
    pub current_fps: u16,
    /// Mean duration of the pure rasterization/blitting pass in milliseconds.
    pub avg_render_ms: u16,
    /// Mean total frame duration (including game logic, simulation, and sync stalls) in milliseconds.
    pub avg_total_ms: u16,
    /// Indicates whether dynamic interlacing (half-column skipping) was active.
    pub is_throttled: bool,
}

impl DebugStats {
    /// Creates a zeroed debug tracker.
    pub const fn new() -> Self {
        Self {
            last_sample_time: 0,
            frame_accumulator: 0,
            render_time_accumulator_us: 0,
            total_frame_time_accumulator_us: 0,
            current_fps: 0,
            avg_render_ms: 0,
            avg_total_ms: 0,
            is_throttled: false,
        }
    }

    /// Registers the elapsed metrics for the frame.
    ///
    /// # Arguments
    /// * `render_us` - Microseconds spent strictly inside [`Compositor::render`](crate::compositor::Compositor::render).
    /// * `total_us` - Microseconds elapsed between consecutive frame submissions.
    /// * `throttled` - `true` if column interlacing dropped half the screen tiles this frame.
    #[inline(always)]
    pub fn record_metrics(&mut self, render_us: u64, total_us: u64, throttled: bool) {
        self.render_time_accumulator_us = self.render_time_accumulator_us.saturating_add(render_us);
        self.total_frame_time_accumulator_us = self
            .total_frame_time_accumulator_us
            .saturating_add(total_us);
        self.is_throttled = throttled;
    }

    /// Advances the frame counter and updates metrics averages if 500 ms have elapsed.
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
                let frames = self.frame_accumulator as u64;
                self.avg_render_ms = (self.render_time_accumulator_us / (frames * 1000)) as u16;
                self.avg_total_ms = (self.total_frame_time_accumulator_us / (frames * 1000)) as u16;
            }

            self.last_sample_time = now;
            self.frame_accumulator = 0;
            self.render_time_accumulator_us = 0;
            self.total_frame_time_accumulator_us = 0;
        }
    }

    /// Paints the debug status badge directly onto the display glass.
    ///
    /// Output format: `"<FPS>F <Render>ms/<Total>ms[*]"`.
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

        let mut text = heapless::String::<32>::new();
        // FPS
        let _ = text.push_str(
            string_from_u16(self.current_fps)
                .as_str()
                .trim_matches('\0'),
        );
        let _ = text.push_str("F ");
        // Pure render duration in ms
        let _ = text.push_str(
            string_from_u16(self.avg_render_ms)
                .as_str()
                .trim_matches('\0'),
        );
        let _ = text.push_str("ms/");
        // Total frame duration in ms
        let _ = text.push_str(
            string_from_u16(self.avg_total_ms)
                .as_str()
                .trim_matches('\0'),
        );
        let _ = text.push_str("ms");
        if self.is_throttled {
            let _ = text.push('*');
        }
        let _ = text.push('\0');

        draw_string_cfg(&text, Point::new(4, 3), false, &cfg, false);
    }
}
