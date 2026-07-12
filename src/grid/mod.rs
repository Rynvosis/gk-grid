use std::{fmt::Debug, hash::Hash};
pub(crate) mod geometry;
pub(crate) mod swizzle;

/// Marker for anything usable as a grid cell coordinate.
pub trait GridCell: Copy + Eq + Hash + Debug + Send + Sync + 'static {}
impl<T: Copy + Eq + Hash + Debug + Send + Sync + 'static> GridCell for T {}
pub type CellOf<G> = <G as Grid>::Cell;

pub trait GridCorner: Copy + Eq + Hash + Debug + Send + Sync + 'static {}
impl<T: Copy + Eq + Hash + Debug + Send + Sync + 'static> GridCorner for T {}
pub type CornerOf<G> = <G as Grid>::Corner;

pub trait GridSlot: Copy + Eq + Hash + Debug + Send + Sync + 'static {}
impl<T: Copy + Eq + Hash + Debug + Send + Sync + 'static> GridSlot for T {}
pub type SlotOf<G> = <G as Grid>::Slot;

pub trait Grid {
    type Cell: GridCell;
    type Corner: GridCorner;
    type Slot: GridSlot;

    /// The connection slots available at a cell, in winding order.
    fn slots(&self, cell: impl Into<Self::Cell>) -> impl Iterator<Item = Self::Slot>;

    /// Attempts to find the neighboring cell in the specified direction.
    /// Warning: None can mean A) Cell not found, B) Slot not valid, C) Slot valid but doesn't have a cell on the other side
    fn try_neighbour(&self, cell: impl Into<Self::Cell>, direction: impl Into<Self::Slot>) -> Option<Self::Cell>;
    fn neighbours(&self, cell: impl Into<Self::Cell>) -> impl Iterator<Item = (Self::Slot, Self::Cell)>;

    //todo: consider making a better connection return type than Option<Cell> with things like which slot you moved through on the connecting cell
}

/// A grid whose slots always resolve, so its neighbour lookups are infallible.
pub trait TotalGrid: Grid {
    fn neighbour(&self, cell: impl Into<Self::Cell>, direction: impl Into<Self::Slot>) -> Self::Cell {
        self.try_neighbour(cell, direction).unwrap()
    }
}
