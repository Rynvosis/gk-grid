use std::fmt::Debug;
use std::hash::Hash;

/// Marker for anything usable as a grid cell coordinate
pub trait GridCell: Copy + Eq + Hash + Debug + Send + Sync + 'static {}
impl<T: Copy + Eq + Hash + Debug + Send + Sync + 'static> GridCell for T {}

pub trait Grid {
    type Cell: GridCell;
}

pub trait GridTopology: Grid {
    fn neighbours(&self, cell: Self::Cell) -> impl Iterator<Item = Self::Cell>;
}

pub trait GridGeometry: Grid {
    type Position: Copy + Send + Sync + 'static;
    fn cell_to_world(&self, cell: Self::Cell) -> Self::Position;
    //corners of a cell in a fixed canonical winding order
    fn cell_corners(&self, cell: Self::Cell) -> impl Iterator<Item = Self::Position>;
    //todo: consider if coordinatesystem typesystem, baking orientability into it with things like cornermappings
}

pub trait PointQuery: GridGeometry {
    fn world_to_cell(&self, world: Self::Position) -> Option<Self::Cell>;
}
