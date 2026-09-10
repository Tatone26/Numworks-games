//! Visual rendering, dirty grids, tilemaps, and camera viewports.

pub mod compositor;
pub mod dirty_grid;
#[macro_use]
pub mod macros;
pub mod sprite;
pub mod texture;
pub mod tilemap;
pub mod viewport;

// Re-export Sprite so it can be found directly under `crate::graphics::Sprite`
pub use compositor::{InterlaceMode, Renderable};
pub use sprite::Sprite;
pub use texture::{Animation, FrameCoord, TextureDescriptor};
pub use tilemap::{Parallax, Tilemap, WrapMode};
pub use viewport::Viewport;
