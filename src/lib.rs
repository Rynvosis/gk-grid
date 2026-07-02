mod gizmos;
mod grid;
mod region;
mod relations;
#[cfg(feature = "bevy")] mod square;
#[cfg(feature = "bevy")] mod tilemap;

pub mod prelude {
    pub use crate::grid::{Grid, GridGeometry, GridTopology, PointQuery};
    pub use crate::region::{RectRegion, Region};
    pub use crate::square::SquareGrid;
    pub use crate::tilemap::Tilemap;                              // core
    #[cfg(feature = "bevy")] pub use crate::relations::{TilemapOf, Tilemaps};
    #[cfg(feature = "bevy")] pub use crate::gizmos::{GridGizmo, GridGizmoPlugin};
}