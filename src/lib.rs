#[cfg(feature = "bevy")]
mod bevy_ext;
pub mod chunk;
mod graph;
mod grid;
mod layered;
mod math;
mod quad;
mod region;
mod store;

pub mod prelude {
    #[cfg(feature = "bevy")]
    pub use crate::bevy_ext::{
        gizmos::{GridGizmoPlugin, cell_gizmo, draw_cell_outline, tilemap_gizmo},
        picking::{GridPickingPlugin, PickableCells},
        relations::{TilemapOf, Tilemaps},
        tiles::TileReader,
        world_ray_to_local,
    };
    pub use crate::{
        chunk::ChunkLayout,
        graph::{FaceRegion, GraphGrid, NonManifoldError, geometry::Mesh3DGridGeometry, merge_coplanar},
        grid::{
            CellOf, CornerOf, Grid, SlotOf, TotalGrid,
            geometry::{GridGeometry, PointQuery, RayCast, RayHit, RayHitOf, TotalGridGeometry, TotalPointQuery},
            swizzle::GridSwizzle,
        },
        layered::{
            LayeredCell, LayeredGrid, LayeredRegion, LayeredSlot,
            geometry::{PlanarLayeredGeometry, RadialLayeredGeometry},
        },
        quad::{QuadChunkLayout, QuadCorner, QuadDir, QuadGrid, geometry::QuadGridGeometry},
        region::{RectRegion, Region},
        store::{ChunkedTileStore, DenseTileStore, SparseTileStore, TileStore},
    };
}
