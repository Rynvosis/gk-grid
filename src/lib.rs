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
    pub use crate::tiles::TileReader;
    pub use crate::{
        chunk::ChunkLayout,
        grid::{
            CellOf, CornerOf, Grid, SlotOf, TotalGrid,
            geometry::{GridGeometry, PointQuery, RayCast, RayHit, RayHitOf, TotalGridGeometry, TotalPointQuery},
            swizzle::GridSwizzle,
        },
        layered::{
            LayeredCell, LayeredGrid, LayeredRegion, LayeredSlot,
            geometry::{PlanarLayeredGeometry, RadialLayeredGeometry},
        },
        mesh::{FaceRegion, MeshGrid, geometry::MeshGridGeometry},
        quad::{QuadChunkLayout, QuadCorner, QuadDir, QuadGrid, geometry::QuadGridGeometry},
        region::{RectRegion, Region},
        store::{ChunkedTileStore, DenseTileStore, SparseTileStore, TileStore},
    };
}
