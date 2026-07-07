use crate::gizmos::GizmoLine;
use crate::grid::GridGeometry;
use crate::prelude::{TileStore, TilemapOf};
use bevy::color::Color;
use bevy::prelude::*;

#[derive(Component)]
pub struct TilemapGizmo {
    pub color: Color,
}

pub fn draw_tilemap_gizmos<S, G>(
    tilemaps: Query<(&S, &TilemapGizmo, &TilemapOf)>,
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
