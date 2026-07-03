#[cfg(feature = "bevy")]
mod gizmos;
mod grid;
mod region;
#[cfg(feature = "bevy")]
mod relations;
mod square;
mod tilemap;

pub mod prelude {
    #[cfg(feature = "bevy")]
    pub use crate::gizmos::{GridGizmo, GridGizmoPlugin};
    pub use crate::grid::{Grid, GridGeometry, GridTopology, PointQuery};
    pub use crate::region::{RectRegion, Region};
    #[cfg(feature = "bevy")]
    pub use crate::relations::{TilemapOf, Tilemaps};
    pub use crate::square::SquareGrid;
    pub use crate::tilemap::Tilemap;
}
