//! Declarative macros for top-level engine orchestration and memory initialization.

/// Statically allocates a scratch staging buffer tailored to viewport dimensions.
#[macro_export]
macro_rules! define_scratch_buffer {
    ($vis:vis $name:ident, $cell_area:expr, $cols:expr, $rows:expr) => {
        $vis static mut $name: [::numworks_utils::eadk::Color; {
            let max_chunks = if $cols > $rows { $cols } else { $rows };
            $cell_area + (max_chunks * $cell_area)
        }] = [::numworks_utils::eadk::Color::BLACK; {
            let max_chunks = if $cols > $rows { $cols } else { $rows };
            $cell_area + (max_chunks * $cell_area)
        }];
    };
}

/// Instantiates an Engine bound to full screen.
#[macro_export]
macro_rules! init_engine {
    (
        color: $color:expr,
        interlace: $interlace:expr,
        scratch: $scratch:ident $(,)?
    ) => {{
        let scratch_ref = unsafe { &mut *(&raw mut $scratch) };
        $crate::engine::Engine::new($color, $interlace, scratch_ref)
    }};
}

/// Instantiates an Engine bound to a sub-screen window.
#[macro_export]
macro_rules! init_engine_window {
    (
        color: $color:expr,
        window: [$x:expr, $y:expr, $w:expr, $h:expr],
        interlace: $interlace:expr,
        scratch: $scratch:ident $(,)?
    ) => {{
        let scratch_ref = unsafe { &mut *(&raw mut $scratch) };
        $crate::engine::Engine::new_with_window(
            $color,
            $x as u16,
            $y as u16,
            $w as u16,
            $h as u16,
            $interlace,
            scratch_ref,
        )
    }};
}
