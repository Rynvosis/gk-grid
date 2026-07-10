//! Geometry for a layered grid: the base geometry with every point lifted to its layer.

use crate::{
    grid::{CellOf, CornerOf},
    layered::Layered,
    prelude::GridGeometry,
};

/// Maps a base point at layer 0 to where it sits on a target layer.
/// One lift drives all the geometry: corners and centres are just the base's, lifted.
/// Operates entirely in the base geometry's local space, never world space.
pub trait Extrude<C, P> {
    /// Lifts a layer-0 point to the given layer, told which base cell it belongs to.
    fn lift(&self, point: P, cell: C, layer: i32) -> P;
    // todo: lower(point, cell) -> layer, the inverse, for picking in phase 5.
}

/// A base geometry stacked into layers by an extrusion.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct LayeredGeometry<Geo, E> {
    base: Geo,
    extrude: E,
}

impl<Geo, E> LayeredGeometry<Geo, E> {
    /// Stacks a base geometry into layers with the given extrusion.
    pub fn new(base: Geo, extrude: E) -> Self {
        Self { base, extrude }
    }
}

impl<Geo, E> GridGeometry for LayeredGeometry<Geo, E>
where
    Geo: GridGeometry,
    E: Extrude<CellOf<Geo::Grid>, Geo::Position>,
{
    type Grid = Layered<Geo::Grid>;
    type Position = Geo::Position;

    fn try_cell_center(&self, cell: impl Into<CellOf<Self::Grid>>) -> Option<Self::Position> {
        let cell = cell.into();
        let base_center = self.base.try_cell_center(cell.cell)?;
        Some(self.extrude.lift(base_center, cell.cell, cell.layer))
    }

    fn try_cell_corners(
        &self,
        cell: impl Into<CellOf<Self::Grid>>,
    ) -> Option<impl Iterator<Item = (CornerOf<Self::Grid>, Self::Position)>> {
        let cell = cell.into();
        let base_corners = self.base.try_cell_corners(cell.cell)?;
        Some(
            base_corners
                .map(move |(corner, base_corner)| (corner, self.extrude.lift(base_corner, cell.cell, cell.layer))),
        )
    }
}
