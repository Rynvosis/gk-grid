pub mod chunk;
#[cfg(feature = "bevy")]
mod gizmos;
mod grid;
mod region;
#[cfg(feature = "bevy")]
mod relations;
mod square;
mod store;
#[cfg(feature = "bevy")]
mod tiles;

pub mod prelude {
    pub use crate::chunk::ChunkLayout;
    #[cfg(feature = "bevy")]
    pub use crate::gizmos::{GridGizmoPlugin, cell_gizmo, tilemap_gizmo};
    pub use crate::grid::{Grid, GridGeometry, GridTopology, PointQuery};
    pub use crate::region::{RectRegion, Region};
    #[cfg(feature = "bevy")]
    pub use crate::relations::{TilemapOf, Tilemaps};
    pub use crate::square::{SquareChunkLayout, SquareGrid};
    pub use crate::store::{Chunked, Dense, Sparse, TileStore};
    #[cfg(feature = "bevy")]
    pub use crate::tiles::Tiles;
}
