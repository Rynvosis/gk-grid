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

/// The boundary between two cells, seen from one side.
///
/// A slot names a boundary from the cell it leaves; the cell on the other side names the same
/// boundary by a slot of its own. A connection carries both, so a caller crossing a boundary knows
/// where it landed and which way it came in.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct Connection<C, S> {
    /// The cell across the boundary.
    pub cell: C,
    /// The slot that leads back, as `cell` numbers it. Not a slot on the cell that was left.
    pub back: S,
}

impl<C, S> Connection<C, S> {
    /// A boundary leading to `cell`, which leads back through `back`.
    pub fn new(cell: C, back: S) -> Self {
        Self { cell, back }
    }
}

/// A [`Connection`] keyed on a grid's cell and slot types.
pub type ConnectionOf<G> = Connection<CellOf<G>, SlotOf<G>>;

pub trait Grid {
    type Cell: GridCell;
    type Corner: GridCorner;
    type Slot: GridSlot;

    /// The connection slots available at a cell, in winding order. Slot `i` is the boundary between
    /// corner `i` and corner `i + 1` of the geometry paired with this grid.
    fn slots(&self, cell: impl Into<Self::Cell>) -> impl Iterator<Item = Self::Slot>;

    /// Attempts to cross the boundary a slot names, reporting where it leads and the way back.
    /// Warning: None can mean A. Cell not found, B. Slot not valid, C. Slot valid but doesn't have a cell on the other side
    fn try_connection(
        &self,
        cell: impl Into<Self::Cell>,
        direction: impl Into<Self::Slot>,
    ) -> Option<ConnectionOf<Self>>;

    /// Attempts to find the neighbouring cell in the specified direction.
    fn try_neighbour(&self, cell: impl Into<Self::Cell>, direction: impl Into<Self::Slot>) -> Option<Self::Cell> {
        self.try_connection(cell, direction).map(|connection| connection.cell)
    }

    /// Every slot of a cell that leads somewhere, paired with the cell it leads to.
    fn neighbours(&self, cell: impl Into<Self::Cell>) -> impl Iterator<Item = (Self::Slot, Self::Cell)> {
        let cell = cell.into();
        self.slots(cell)
            .filter_map(move |slot| Some((slot, self.try_neighbour(cell, slot)?)))
    }
}

/// A grid whose slots always resolve, so its neighbour lookups are infallible.
pub trait TotalGrid: Grid {
    fn neighbour(&self, cell: impl Into<Self::Cell>, direction: impl Into<Self::Slot>) -> Self::Cell {
        self.try_neighbour(cell, direction).unwrap()
    }
}
