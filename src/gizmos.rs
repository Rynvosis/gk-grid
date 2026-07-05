use crate::prelude::*;
use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component)]
pub struct GridGizmo {
    pub color: Color,
}

pub struct GridGizmoPlugin<S, G>(PhantomData<(S, G)>);
impl<S, G> Default for GridGizmoPlugin<S, G> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<S, G> Plugin for GridGizmoPlugin<S, G>
where
    S: TileStore + Component,
    G: Component + GridGeometry<Cell = S::Cell>,
    G::Position: GizmoLine,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_tilemap_gizmos::<S, G>);
    }
}

fn draw_tilemap_gizmos<S, G>(
    tilemaps: Query<(&S, &GridGizmo, &TilemapOf)>,
    grids: Query<&G>,
    mut gizmos: Gizmos,
) where
    S: TileStore + Component,
    G: Component + GridGeometry<Cell = S::Cell>,
    G::Position: GizmoLine,
{
    for (store, gizmo, grid) in tilemaps.iter() {
        let Ok(grid) = grids.get(grid.0) else {
            continue;
        };
        for cell in store.cells() {
            let corners: Vec<G::Position> = grid.cell_corners(cell).collect();
            for i in 0..corners.len() {
                corners[i].line(&mut gizmos, corners[(i + 1) % corners.len()], gizmo.color);
            }
        }
    }
}

trait GizmoLine: Copy {
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
