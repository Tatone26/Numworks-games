//! Declarative macros for spatial boundaries, hitboxes, and kinematics.

/// Creates an axis-aligned bounding box with explicit offsets and dimensions.
#[macro_export]
macro_rules! hitbox {
    ($ox:expr, $oy:expr, $w:expr, $h:expr $(,)?) => {
        $crate::physics::Hitbox::new($ox as i16, $oy as i16, $w as u16, $h as u16)
    };
}

/// Creates a centered bounding box where (0,0) is the center of the entity.
#[macro_export]
macro_rules! centered_hitbox {
    ($w:expr, $h:expr $(,)?) => {
        $crate::physics::Hitbox::new_centered($w as u16, $h as u16)
    };
}

/// Creates a padded hitbox with symmetrical or independent horizontal/vertical margins.
#[macro_export]
macro_rules! inset_hitbox {
    ($w:expr, $h:expr, $pad:expr $(,)?) => {
        $crate::physics::Hitbox::new_inset($w as u16, $h as u16, $pad as u16, $pad as u16)
    };
    ($w:expr, $h:expr, $pad_x:expr, $pad_y:expr $(,)?) => {
        $crate::physics::Hitbox::new_inset($w as u16, $h as u16, $pad_x as u16, $pad_y as u16)
    };
}

/// Creates a static world-space trigger zone.
#[macro_export]
macro_rules! trigger_zone {
    (x: $x:expr, y: $y:expr, w: $w:expr, h: $h:expr $(,)?) => {
        $crate::physics::Hitbox::new($x as i16, $y as i16, $w as u16, $h as u16)
    };
}

/// Creates a kinematic Body with optional initial velocity.
#[macro_export]
macro_rules! body {
    ($x:expr, $y:expr $(,)?) => {
        $crate::physics::Body::new($x as f32, $y as f32)
    };
    ($x:expr, $y:expr, vx: $vx:expr, vy: $vy:expr $(,)?) => {
        $crate::physics::Body::new($x as f32, $y as f32).with_velocity($vx as f32, $vy as f32)
    };
}
