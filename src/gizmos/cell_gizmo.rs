use crate::gizmos::GizmoLine;
use crate::grid::GridGeometry;
use crate::prelude::{TileStore, TilemapOf};
use bevy::color::Color;
use bevy::prelude::{Component, Gizmos, Query};

#[derive(Component)]
pub struct TilemapCellGizmos<T: TileStore + Component> {
    pub color_fn: fn(map: &T, cell: &T::Cell) -> Color,
}

pub fn draw_tilemap_cell_gizmos<S, G>(
    tilemaps: Query<(&S, &TilemapCellGizmos<S>, &TilemapOf)>,
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
            let color = (gizmo.color_fn)(store, &cell);
            for i in 0..corners.len() {
                corners[i].line(&mut gizmos, corners[(i + 1) % corners.len()], color);
            }
        }
    }
}
