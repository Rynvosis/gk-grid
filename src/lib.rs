#[cfg(feature = "bevy")]
mod gizmos;
mod grid;
mod region;
#[cfg(feature = "bevy")]
mod relations;
mod square;
pub mod tile_storage;
mod tilemap;
pub mod chunk;

pub mod prelude {
    #[cfg(feature = "bevy")]
    pub use crate::gizmos::{GridGizmo, GridGizmoPlugin};
    pub use crate::chunk::ChunkLayout;
    pub use crate::grid::{Grid, GridGeometry, GridTopology, PointQuery};
    pub use crate::region::{RectRegion, Region};
    #[cfg(feature = "bevy")]
    pub use crate::relations::{TilemapOf, Tilemaps};
    pub use crate::square::{SquareChunkLayout, SquareGrid, SquareTilemap};
    pub use crate::tile_storage::TileStorage;
    pub use crate::tilemap::Tilemap;
}
