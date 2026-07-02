use crate::grid::GridGeometry;
use crate::prelude::SquareGrid;
use crate::region::Region;
use crate::relations::TilemapOf;
use crate::tilemap::Tilemap;
use bevy::prelude::*;

#[derive(Component)]
pub struct GridGizmo {
    pub color: Color,
}

pub struct GridGizmoPlugin;
impl Plugin for GridGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_tilemap_gizmos);
    }
}

fn draw_tilemap_gizmos(
    tilemaps: Query<(&Tilemap, &GridGizmo, &TilemapOf)>,
    grids: Query<&SquareGrid>,
    mut gizmos: Gizmos,
) {
    for (tilemap, gizmo, grid) in tilemaps.iter() {
        let Ok(grid) = grids.get(grid.0) else {
            continue;
        };
        for cell in tilemap.region.iter() {
            let corners: Vec<Vec2> = grid.cell_corners(cell).collect();
            for i in 0..corners.len() {
                gizmos.line_2d(corners[i], corners[(i + 1) % corners.len()], gizmo.color);
            }
        }
    }
}
