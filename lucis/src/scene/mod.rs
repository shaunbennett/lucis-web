// Scene Module
// - material
// - texturing
// - scene trees

mod color;
mod intersection;
mod light;
mod node;

pub use self::color::Color;
pub use self::intersection::Intersection;
pub use self::light::Light;
pub use self::node::{Intersect, Material, SceneNode};
