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
    #[cfg(feature = "bevy")]
    pub use crate::gizmos::{GridGizmoPlugin, cell_gizmo, tilemap_gizmo};
    #[cfg(feature = "bevy")]
    pub use crate::relations::{TilemapOf, Tilemaps};
    #[cfg(feature = "bevy")]
    pub use crate::tiles::Tiles;
    pub use crate::{
        chunk::ChunkLayout,
        grid::{
            CellOf, CornerOf, Grid, SlotOf, TotalGrid,
            geometry::{GridGeometry, PointQuery, TotalGridGeometry},
            swizzle::GridSwizzle,
        },
        layered::{
            LayerSlot, Layered, LayeredCell, LayeredRegion,
            geometry::{Extrude, LayeredGeometry},
        },
        mesh::{FaceRegion, MeshGrid, geometry::MeshGridGeometry},
        quad::{QuadChunkLayout, QuadCorners, QuadDirs, QuadGrid, geometry::QuadGridGeometry},
        region::{RectRegion, Region},
        store::{Chunked, Dense, Sparse, TileStore},
    };
}
