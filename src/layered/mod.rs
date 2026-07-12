//! A base grid stacked into layers, plus the region that bounds how many.

pub(crate) mod geometry;

use std::ops::Range;

use crate::{grid::Grid, region::Region};

/// A base cell plus which layer it sits on.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct LayeredCell<C> {
    /// The cell on the base grid.
    pub cell: C,
    /// The layer, signed so it can go below zero.
    pub layer: i32,
}

impl<C> LayeredCell<C> {
    /// A base cell on the given layer.
    pub fn new(cell: C, layer: i32) -> Self {
        Self { cell, layer }
    }
}

impl<C> From<(C, i32)> for LayeredCell<C> {
    fn from((cell, layer): (C, i32)) -> Self {
        Self { cell, layer }
    }
}

/// A move through a layered grid: along the base, or up and down a layer.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum LayeredSlot<S> {
    /// A base-grid slot, staying on the same layer.
    Base(S),
    /// One layer up.
    Up,
    /// One layer down.
    Down,
}

/// Any base grid stacked into layers. Depth is open-ended here; the region says how many.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct LayeredGrid<G> {
    base: G,
}

impl<G> LayeredGrid<G> {
    /// Stacks a base grid into layers.
    pub fn new(base: G) -> Self {
        Self { base }
    }
}

impl<G: Grid> Grid for LayeredGrid<G> {
    type Cell = LayeredCell<G::Cell>;
    type Corner = G::Corner;
    type Slot = LayeredSlot<G::Slot>;

    fn slots(&self, cell: impl Into<Self::Cell>) -> impl Iterator<Item = Self::Slot> {
        let layered_cell: Self::Cell = cell.into();
        self.base
            .slots(layered_cell.cell)
            .map(LayeredSlot::Base)
            .chain([LayeredSlot::Up, LayeredSlot::Down])
    }

    fn try_neighbour(&self, cell: impl Into<Self::Cell>, direction: impl Into<Self::Slot>) -> Option<Self::Cell> {
        let layered_cell: Self::Cell = cell.into();
        let direction: Self::Slot = direction.into();

        match direction {
            LayeredSlot::Base(slot) => self
                .base
                .try_neighbour(layered_cell.cell, slot)
                .map(|cell| LayeredCell::new(cell, layered_cell.layer)),
            LayeredSlot::Up => Some(LayeredCell::new(layered_cell.cell, layered_cell.layer + 1)),
            LayeredSlot::Down => Some(LayeredCell::new(layered_cell.cell, layered_cell.layer - 1)),
        }
    }

    fn neighbours(&self, cell: impl Into<Self::Cell>) -> impl Iterator<Item = (Self::Slot, Self::Cell)> {
        let layered_cell: Self::Cell = cell.into();
        self.base
            .neighbours(layered_cell.cell)
            .map(move |(slot, cell)| (LayeredSlot::Base(slot), LayeredCell::new(cell, layered_cell.layer)))
            .chain([
                (
                    LayeredSlot::Up,
                    LayeredCell::new(layered_cell.cell, layered_cell.layer + 1),
                ),
                (
                    LayeredSlot::Down,
                    LayeredCell::new(layered_cell.cell, layered_cell.layer - 1),
                ),
            ])
    }
}

/// A base region stacked across a range of layers.
/// Layers are half-open, so `layers.start` is the lowest layer and `layers.end` is one more than the highest layer.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct LayeredRegion<R> {
    base: R,
    layers: Range<i32>,
}

impl<R> LayeredRegion<R> {
    /// A base region across a layer range, like `-1..3` for four layers straddling zero.
    pub fn new(base: R, layers: Range<i32>) -> Self {
        Self { base, layers }
    }
}

impl<R: Region> Region for LayeredRegion<R> {
    type Cell = LayeredCell<R::Cell>;

    fn iter(&self) -> impl Iterator<Item = Self::Cell> {
        self.base
            .iter()
            .flat_map(move |cell| self.layers.clone().map(move |layer| LayeredCell::new(cell, layer)))
    }

    fn contains(&self, cell: Self::Cell) -> bool {
        self.layers.contains(&cell.layer) && self.base.contains(cell.cell)
    }

    fn index_of(&self, cell: Self::Cell) -> Option<usize> {
        let layer_index = self
            .layers
            .contains(&cell.layer)
            .then(|| (cell.layer - self.layers.start) as usize)?;
        let base_index = self.base.index_of(cell.cell)?;
        Some(base_index * self.layers.len() + layer_index)
    }

    fn len(&self) -> usize {
        self.base.len() * self.layers.len()
    }
}
