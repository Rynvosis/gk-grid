pub mod chunk;
#[cfg(feature = "bevy")]
mod gizmos;
mod grid;
mod layered;
mod mesh;
mod quad;
mod region;
#[cfg(feature = "bevy")]
mod relations;
mod store;
#[cfg(feature = "bevy")]
mod tiles;

pub mod prelude {
    pub use crate::chunk::ChunkLayout;
    #[cfg(feature = "bevy")]
    pub use crate::gizmos::{GridGizmoPlugin, cell_gizmo, tilemap_gizmo};
    pub use crate::grid::geometry::{GridGeometry, PointQuery, TotalGridGeometry};
    pub use crate::grid::{CellOf, CornerOf, Grid, SlotOf, TotalGrid};
    pub use crate::layered::geometry::{Extrude, LayeredGeometry};
    pub use crate::layered::{LayerSlot, Layered, LayeredCell, LayeredRegion};
    pub use crate::mesh::geometry::MeshGridGeometry;
    pub use crate::mesh::{FaceRegion, MeshGrid};
    pub use crate::quad::geometry::QuadGridGeometry;
    pub use crate::quad::{QuadChunkLayout, QuadCorners, QuadDirs, QuadGrid};
    pub use crate::region::{RectRegion, Region};
    #[cfg(feature = "bevy")]
    pub use crate::relations::{TilemapOf, Tilemaps};
    pub use crate::store::{Chunked, Dense, Sparse, TileStore};
    #[cfg(feature = "bevy")]
    pub use crate::tiles::Tiles;
}
