use crate::gizmos::GizmoLine;
use crate::grid::geometry::GridGeometry;
use crate::prelude::{Grid, TileStore, TilemapOf};
use bevy::color::Color;
use bevy::prelude::{Component, Gizmos, Query};

#[derive(Component, Debug)]
pub struct TilemapCellGizmos<T: TileStore + Component> {
    pub color_fn: fn(map: &T, cell: &T::Cell) -> Color,
}

pub fn draw_tilemap_cell_gizmos<S, G>(
    tilemaps: Query<(&S, &TilemapCellGizmos<S>, &TilemapOf)>,
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
            let color = (gizmo.color_fn)(store, &cell);
            for i in 0..corners.len() {
                corners[i].line(&mut gizmos, corners[(i + 1) % corners.len()], color);
            }
        }
    }
}
