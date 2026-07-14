use glam::Vec3;

use crate::grid::{CellOf, CornerOf, Grid, SlotOf};

pub trait GridGeometry {
    type Grid: Grid;
    /// Local space: relative to the grid entity's own origin, not world space.
    /// Converting to world space is a `Transform` multiply, done by whatever
    /// consumer actually needs world coordinates.
    type Position: Copy + Send + Sync + 'static;

    fn try_cell_center(&self, cell: impl Into<CellOf<Self::Grid>>) -> Option<Self::Position>;
    /// Corners of a cell in a fixed winding order.
    fn try_cell_corners(
        &self,
        cell: impl Into<CellOf<Self::Grid>>,
    ) -> Option<impl Iterator<Item = (CornerOf<Self::Grid>, Self::Position)>>;
}

/// Geometry where every cell has a center and corners, so its lookups are infallible.
pub trait TotalGridGeometry: GridGeometry {
    fn cell_center(&self, cell: impl Into<CellOf<Self::Grid>>) -> Self::Position {
        self.try_cell_center(cell).expect("total geometry")
    }
    fn cell_corners(
        &self,
        cell: impl Into<CellOf<Self::Grid>>,
    ) -> impl Iterator<Item = (CornerOf<Self::Grid>, Self::Position)> {
        self.try_cell_corners(cell).expect("total geometry")
    }
}

/// Maps a local-space point to the cells that contain it.
pub trait PointQuery: GridGeometry {
    /// Cells whose area contains `local`: empty if none, one for a flat grid,
    /// several where the geometry folds or self-overlaps.
    fn cells_at(&self, local: Self::Position) -> impl Iterator<Item = CellOf<Self::Grid>>;
}

/// A point query that lands every point in exactly one cell, so its lookups are infallible.
pub trait TotalPointQuery: PointQuery {
    /// The single cell containing `local`. Only valid where `cells_at` always yields exactly one.
    fn cell_at(&self, local: Self::Position) -> CellOf<Self::Grid> {
        self.cells_at(local).next().expect("total point query")
    }
}

/// A cell a ray passes through, with the ray parameter `t` at the crossing.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RayHit<C, S> {
    /// The cell the ray entered.
    pub cell: C,
    /// Ray parameter at the crossing: the hit point is `origin + t * dir`.
    pub t: f32,
    /// The face crossed to enter this cell; `None` for the first cell or a flat-surface pierce.
    pub face: Option<S>,
}

/// A [`RayHit`] keyed on a grid's cell and slot types.
pub type RayHitOf<G> = RayHit<CellOf<G>, SlotOf<G>>;

/// Casts a ray through the grid, marching the cells it sweeps.
pub trait RayCast: GridGeometry {
    /// The cells the ray `origin + t * dir` sweeps through, in nondecreasing `t`, each
    /// edge-adjacent to the last with the entry slot in `face` (`None` for the first cell).
    /// A cell may recur where the geometry folds. The stream may be unbounded, so consumers
    /// bound it (for example `take_while` on `t`).
    fn raycast(&self, origin: Self::Position, dir: Self::Position) -> impl Iterator<Item = RayHitOf<Self::Grid>>;
}

/// A geometry whose cells are 2D patches embedded in 3D space.
pub trait Surface: GridGeometry<Position = Vec3> {
    /// Ray parameter and point where the ray first touches the surface, or `None` on a miss.
    fn pierce(&self, origin: Vec3, dir: Vec3) -> Option<(f32, Vec3)>;
}

/// A surface whose surroundings split into layers: a normal field with a global height
/// coordinate, valid within a band.
pub trait Layerable: Surface + RayCast {
    /// Moves a point `offset` along the normal field.
    fn lift(&self, point: Vec3, offset: f32) -> Vec3;
    /// Signed height of a point above the layer-zero surface.
    fn height(&self, point: Vec3) -> f32;
    /// Ray parameters where the ray crosses layer boundaries spaced `spacing` apart, in
    /// nondecreasing `t`, with `1` entering the layer above and `-1` the layer below.
    fn layer_crossings(&self, origin: Vec3, dir: Vec3, spacing: f32) -> impl Iterator<Item = (f32, i32)>;
}
