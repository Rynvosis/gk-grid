use crate::grid::{CellOf, CornerOf, Grid};

pub trait GridGeometry {
    type Grid: Grid;
    type Position: Copy + Send + Sync + 'static;

    fn try_cell_center(&self, cell: impl Into<CellOf<Self::Grid>>) -> Option<Self::Position>;
    /// Corners of a cell in a fixed winding order.
    fn try_cell_corners(
        &self,
        cell: impl Into<CellOf<Self::Grid>>,
    ) -> Option<impl Iterator<Item = (CornerOf<Self::Grid>, Self::Position)>>;
    //todo: coordinate-system type carrying orientation and corner mappings
}

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

pub trait PointQuery: GridGeometry {
    fn world_to_cell(&self, world: Self::Position) -> Option<CellOf<Self::Grid>>;
}
