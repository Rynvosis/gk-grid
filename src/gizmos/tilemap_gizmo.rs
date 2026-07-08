use crate::gizmos::GizmoLine;
use crate::grid::geometry::GridGeometry;
use crate::prelude::{Grid, TileStore, TilemapOf};
use bevy::color::Color;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct TilemapGizmo {
    pub color: Color,
}

pub fn draw_tilemap_gizmos<S, G>(
    tilemaps: Query<(&S, &TilemapGizmo, &TilemapOf)>,
    grids: Query<&G>,
    mut gizmos: Gizmos,
) where
    S: TileStore + Component,
    G: Component + GridGeometry,
    G::Grid: Grid<Cell = S::Cell>,
    G::Position: GizmoLine,
{
    for (store, gizmo, grid) in tilemaps.iter() {
        let Ok(grid) = grids.get(grid.0) else {
            continue;
        };
        for cell in store.cells() {
            let Some(corners) = grid.try_cell_corners(cell) else {
                continue;
            };
            let corners: Vec<G::Position> = corners.map(|(_, pos)| pos).collect();
            for i in 0..corners.len() {
                corners[i].line(&mut gizmos, corners[(i + 1) % corners.len()], gizmo.color);
            }
        }
    }
}
