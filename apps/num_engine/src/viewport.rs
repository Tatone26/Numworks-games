//! Camera tracking, sub-screen window constraints, and coordinate space transformations.

use numworks_utils::eadk::display::{SCREEN_HEIGHT, SCREEN_WIDTH};

/// Controls which region of the world is projected onto the physical display.
///
/// Supports arbitrary sub-screen display windows (`screen_x`, `screen_y`, `screen_w`, `screen_h`)
/// to isolate HUD and UI regions from world overdraw.
#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    /// World X position of the camera's top-left corner
    pub x: i32,
    /// World Y position of the camera's top-left corner.
    pub y: i32,
    /// World X position during the previous frame (used to detect motion).
    pub prev_x: i32,
    /// World Y position during the previous frame (used to detect motion).
    pub prev_y: i32,

    /// Screen-space X origin on physical glass (in pixels).
    pub screen_x: u16,
    /// Screen-space Y origin on physical glass (in pixels).
    pub screen_y: u16,
    /// Pixel width of the viewport window on glass.
    pub screen_w: u16,
    /// Pixel height of the viewport window on glass.
    pub screen_h: u16,
}

impl Viewport {
    /// Creates a stationary viewport filling the entire 320x240 screen at world origin `(0, 0)`.
    pub const fn fixed() -> Self {
        Self {
            x: 0,
            y: 0,
            prev_x: 0,
            prev_y: 0,
            screen_x: 0,
            screen_y: 0,
            screen_w: SCREEN_WIDTH,
            screen_h: SCREEN_HEIGHT,
        }
    }

    /// Creates a full-screen viewport anchored at world coordinates `(x, y)`.
    pub const fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            prev_x: x,
            prev_y: y,
            screen_x: 0,
            screen_y: 0,
            screen_w: SCREEN_WIDTH,
            screen_h: SCREEN_HEIGHT,
        }
    }

    /// Creates a viewport occupying a constrained sub-window of the display glass.
    pub const fn new_window(
        x: i32,
        y: i32,
        screen_x: u16,
        screen_y: u16,
        screen_w: u16,
        screen_h: u16,
    ) -> Self {
        Self {
            x,
            y,
            prev_x: x,
            prev_y: y,
            screen_x,
            screen_y,
            screen_w,
            screen_h,
        }
    }

    /// Returns `true` if the camera moved in world space since the last [`commit_frame`](Self::commit_frame).
    #[inline(always)]
    pub fn moved(&self) -> bool {
        self.x != self.prev_x || self.y != self.prev_y
    }

    /// Latches the current camera coordinates into `(prev_x, prev_y)`.
    #[inline(always)]
    pub fn commit_frame(&mut self) {
        self.prev_x = self.x;
        self.prev_y = self.y;
    }

    /// Translates camera world coordinates by relative delta offsets `(dx, dy)`.
    #[inline(always)]
    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }

    /// Directly sets camera world coordinates.
    #[inline(always)]
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// Centers the viewport window on a target world coordinate `(target_x, target_y)`.
    pub fn look_at(&mut self, target_x: i32, target_y: i32) {
        self.x = target_x - (self.screen_w as i32 / 2);
        self.y = target_y - (self.screen_h as i32 / 2);
    }

    /// Pans the camera only when the target leaves a defined interior margin box.
    ///
    /// Avoids micro-jitter and unnecessary full-screen invalidations during minor player adjustments.
    ///
    /// # Arguments
    /// * `target_x` - World X coordinate of target.
    /// * `target_y` - World Y coordinate of target.
    /// * `margin_x` - Inward pixel padding from left and right screen borders.
    /// * `margin_y` - Inward pixel padding from top and bottom screen borders.
    pub fn follow_deadzone(&mut self, target_x: i32, target_y: i32, margin_x: i32, margin_y: i32) {
        let left_edge = self.x + margin_x;
        let right_edge = self.x + self.screen_w as i32 - margin_x;
        let top_edge = self.y + margin_y;
        let bottom_edge = self.y + self.screen_h as i32 - margin_y;

        if target_x < left_edge {
            self.x = target_x - margin_x;
        } else if target_x > right_edge {
            self.x = target_x - self.screen_w as i32 + margin_x;
        }

        if target_y < top_edge {
            self.y = target_y - margin_y;
        } else if target_y > bottom_edge {
            self.y = target_y - self.screen_h as i32 + margin_y;
        }
    }

    /// Clamps camera edges to stay within world bounding limits `[min_x..max_x, min_y..max_y]`.
    pub fn clamp(&mut self, min_x: i32, max_x: i32, min_y: i32, max_y: i32) {
        let bound_max_x = max_x - self.screen_w as i32;
        let bound_max_y = max_y - self.screen_h as i32;
        self.x = self.x.clamp(min_x, bound_max_x.max(min_x));
        self.y = self.y.clamp(min_y, bound_max_y.max(min_y));
    }

    /// Converts world coordinates `(wx, wy)` into physical display glass coordinates `(sx, sy)`.
    #[inline(always)]
    pub fn world_to_screen(&self, wx: i32, wy: i32) -> (i16, i16) {
        (
            (wx - self.x + self.screen_x as i32) as i16,
            (wy - self.y + self.screen_y as i32) as i16,
        )
    }

    /// Returns `true` if a world-space bounding box intersects the current viewport window.
    #[inline(always)]
    pub fn is_visible(&self, wx: i32, wy: i32, w: u16, h: u16) -> bool {
        let sx = wx - self.x;
        let sy = wy - self.y;
        sx + (w as i32) > 0
            && sx < self.screen_w as i32
            && sy + (h as i32) > 0
            && sy < self.screen_h as i32
    }
}
