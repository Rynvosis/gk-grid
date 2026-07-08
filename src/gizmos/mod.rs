pub mod cell_gizmo;
pub mod tilemap_gizmo;

use crate::gizmos::cell_gizmo::draw_tilemap_cell_gizmos;
use crate::gizmos::tilemap_gizmo::draw_tilemap_gizmos;
use crate::prelude::*;
use bevy::prelude::*;
use std::marker::PhantomData;

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
    G: Component + GridGeometry,
    G::Grid: Grid<Cell = S::Cell>,
    G::Position: GizmoLine,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                draw_tilemap_gizmos::<S, G>,
                draw_tilemap_cell_gizmos::<S, G>,
            ),
        );
    }
}

pub trait GizmoLine: Copy {
    fn line(self, gizmos: &mut Gizmos, to: Self, color: Color);
}
impl GizmoLine for Vec2 {
    fn line(self, gizmos: &mut Gizmos, to: Self, color: Color) {
        gizmos.line_2d(self, to, color);
    }
}
impl GizmoLine for Vec3 {
    fn line(self, gizmos: &mut Gizmos, to: Self, color: Color) {
        gizmos.line(self, to, color);
    }
}

//todo: draw_volumetric_tilemap_gizmos for 3d/mesh grids
