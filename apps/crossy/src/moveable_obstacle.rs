use numworks_utils::eadk::Point;

use crate::frog::Direction;

// the code necessary for moveable obstacles is here
// smooth moving, size, speed, direction, etc.

pub struct MoveableObstacle {
    pub pos: Point,
    pub width: u8,
    pub height: u8, // should almost always be 1 ?
    pub speed: f32,
    pub direction: Direction,
}
