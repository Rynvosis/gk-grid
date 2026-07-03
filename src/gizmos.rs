use crate::prelude::*;
use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component)]
pub struct GridGizmo {
    pub color: Color,
}

pub struct GridGizmoPlugin<T, G>(PhantomData<(T, G)>);
impl<T, G> Default for GridGizmoPlugin<T, G> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<T, G> Plugin for GridGizmoPlugin<T, G>
where
    T: Component + Tilemap,
    G: Component + GridGeometry<Cell = <T::TilemapRegion as Region>::Cell>,
    G::Position: GizmoLine,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_tilemap_gizmos::<T, G>);
    }
}

fn draw_tilemap_gizmos<T, G>(
    tilemaps: Query<(&T, &GridGizmo, &TilemapOf)>,
    grids: Query<&G>,
    mut gizmos: Gizmos,
) where
    T: Component + Tilemap,
    G: Component + GridGeometry<Cell = <T::TilemapRegion as Region>::Cell>,
    G::Position: GizmoLine,
{
    for (tilemap, gizmo, grid) in tilemaps.iter() {
        let Ok(grid) = grids.get(grid.0) else {
            continue;
        };
        for cell in tilemap.region().iter() {
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

//todo: draw_volumetric_tilemap_gizmos for grids with 3d cells, cells are a mesh
