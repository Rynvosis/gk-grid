use bevy::{
    color::Color,
    prelude::{Component, Gizmos, Query, Transform, Vec3},
};

use crate::{
    bevy_ext::gizmos::draw_cell_outline,
    grid::geometry::GridGeometry,
    prelude::{Grid, TileStore, TilemapOf},
};

#[derive(Component, Debug)]
pub struct TilemapCellColorGizmo<T: TileStore + Component> {
    pub color_fn: fn(map: &T, cell: &T::Cell) -> Color,
}

pub fn draw_tilemap_cell_gizmos<S, G>(
    tilemaps: Query<(&S, &TilemapCellColorGizmo<S>, &TilemapOf)>,
    grids: Query<(&G, Option<&Transform>)>,
    mut gizmos: Gizmos,
) where
    S: TileStore + Component,
    G: Component + GridGeometry<Position = Vec3>,
    G::Grid: Grid<Cell = S::Cell>,
{
    for (store, gizmo, grid) in tilemaps.iter() {
        let Ok((grid, transform)) = grids.get(grid.0) else {
            continue;
        };
        for cell in store.cells() {
            let Some(corners) = grid.try_cell_corners(cell) else {
                continue;
            };
            let color = (gizmo.color_fn)(store, &cell);
            draw_cell_outline(&mut gizmos, transform, corners.map(|(_, local)| local), color);
        }
    }
}
