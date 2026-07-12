pub mod cell_gizmo;
pub mod tilemap_gizmo;

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    bevy_ext::gizmos::{cell_gizmo::draw_tilemap_cell_gizmos, tilemap_gizmo::draw_tilemap_gizmos},
    prelude::*,
};

#[derive(Debug)]
pub struct GridGizmoPlugin<S, G>(PhantomData<(S, G)>);
impl<S, G> Default for GridGizmoPlugin<S, G> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<S, G> Plugin for GridGizmoPlugin<S, G>
where
    S: TileStore + Component,
    G: Component + GridGeometry<Position = Vec3>,
    G::Grid: Grid<Cell = S::Cell>,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_tilemap_gizmos::<S, G>, draw_tilemap_cell_gizmos::<S, G>));
    }
}

/// Draws a closed polyline through `corners` (in the grid's local space),
/// transformed by `transform` (identity if absent).
pub fn draw_cell_outline(
    gizmos: &mut Gizmos,
    transform: Option<&Transform>,
    corners: impl Iterator<Item = Vec3>,
    color: Color,
) {
    let transform = transform.copied().unwrap_or_default();
    let mut corners = corners.map(|local| transform.transform_point(local));
    let Some(first) = corners.next() else {
        return;
    };
    let mut prev = first;
    for corner in corners {
        gizmos.line(prev, corner, color);
        prev = corner;
    }
    gizmos.line(prev, first, color);
}

//todo: draw_volumetric_tilemap_gizmos for 3d/mesh grids
