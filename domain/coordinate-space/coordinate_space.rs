#[path = "global-directions/global_directions.rs"]
pub mod global_directions;

pub use global_directions::{AxisSign, GlobalDirection, WorldAxis};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldPoint {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldPoint {
    pub const fn origin() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellPoint {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl CellPoint {
    pub const fn origin() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}
