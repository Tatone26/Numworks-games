pub fn ceil(x: f32) -> f32 {
    let int_part = x as i32 as f32; // Extract the integer part
    if x > int_part {
        int_part + 1.0
    } else {
        int_part
    }
}

/// Returns the next biggest, in absolute value, integer number (ex : -3.0 from -2.7 and 3.0 from 2.7)
pub fn floor(x: f32) -> f32 {
    let int_part = x as i32 as f32; // Extract the integer part
    if x < 0.0 {
        int_part - 1.0
    } else {
        int_part
    }
}

pub fn abs(x: i32) -> u32 {
    if x < 0 {
        (-x) as u32
    } else {
        x as u32
    }
}
