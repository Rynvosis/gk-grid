//! Raycast demo: an in-plane laser from the grid centre to the cursor. Solid cells always
//! render orange; the ray recolours the empty cells it crosses and stops at the first solid.

use bevy::prelude::*;
use gk_grid::prelude::{tilemap_gizmo::UniformTilemapGizmo, *};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GridGizmoPlugin::<DenseTileStore<RectRegion, bool>, QuadGridGeometry>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (trace_ray, draw_solid_cells))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    // No Transform on the grid entity: it defaults to identity, so world == local here.
    let grid = commands.spawn(QuadGridGeometry::rect(Vec2::splat(32.0))).id();
    let region = RectRegion::new(IVec2::splat(-10), UVec2::splat(20));
    commands.spawn((
        // A sparse pattern of solid cells (offset so the origin cell stays empty) for the ray to hit.
        DenseTileStore::from_region(region, |c: IVec2| c.x.rem_euclid(5) == 2 && c.y.rem_euclid(5) == 2),
        UniformTilemapGizmo {
            color: Color::srgba(1.0, 1.0, 1.0, 0.1),
        },
        TilemapOf(grid),
    ));
}

/// Recolours the empty cells the ray from the grid centre to the cursor crosses, up to the
/// first solid cell (which stops the ray).
fn trace_ray(
    mut gizmos: Gizmos,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    grids: Query<(&QuadGridGeometry, Option<&Transform>)>,
    stores: Query<(&DenseTileStore<RectRegion, bool>, &TilemapOf)>,
) {
    // Cursor -> world point. Bevy does the screen->world math.
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok(cursor_world) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        return;
    };

    // A laser from the grid centre to the cursor. `dir` is unit, so `t` is world distance.
    let origin = Vec3::ZERO;
    let target = cursor_world.extend(0.0);
    let dir = (target - origin).normalize_or_zero();
    if dir == Vec3::ZERO {
        return;
    }
    let cursor_dist = (target - origin).length();

    for (store, tilemap) in &stores {
        let Ok((geom, transform)) = grids.get(tilemap.0) else {
            continue;
        };
        let (local_origin, local_dir) = world_ray_to_local(transform, origin, dir);
        let size = geom.cell_size();

        // March, recolouring empty cells, until a solid cell stops the ray (or the cursor).
        let mut stop_t = cursor_dist;
        // The quad grid is stateless, so the march ignores it; it is in the signature because a mesh
        // grid's march cannot work out its neighbours without one.
        for hit in geom
            .raycast(&QuadGrid {}, local_origin, local_dir)
            .take_while(|h| h.t <= cursor_dist)
        {
            match store.get(hit.cell) {
                Some(&true) => {
                    stop_t = hit.t; // hit a wall: stop, leave it for the orange drawer
                    break;
                }
                Some(&false) => {
                    if let Some(center) = geom.try_cell_center(hit.cell) {
                        let world = transform.map_or(center, |t| t.transform_point(center));
                        gizmos.rect_2d(
                            Isometry2d::from_translation(world.truncate()),
                            size,
                            Color::srgb(0.2, 0.7, 1.0),
                        );
                    }
                }
                None => {} // outside the grid: the ray passes through
            }
        }

        // Draw the beam only as far as it actually reached.
        let stop = (origin + stop_t * dir).truncate();
        gizmos.line_2d(origin.truncate(), stop, Color::srgb(0.3, 0.3, 0.3));
    }
}

/// Always renders every solid cell orange.
fn draw_solid_cells(
    mut gizmos: Gizmos,
    grids: Query<(&QuadGridGeometry, Option<&Transform>)>,
    stores: Query<(&DenseTileStore<RectRegion, bool>, &TilemapOf)>,
) {
    for (store, tilemap) in &stores {
        let Ok((geom, transform)) = grids.get(tilemap.0) else {
            continue;
        };
        let size = geom.cell_size();
        for cell in store.cells() {
            if store.get(cell).copied().unwrap_or(false)
                && let Some(center) = geom.try_cell_center(cell)
            {
                let world = transform.map_or(center, |t| t.transform_point(center));
                gizmos.rect_2d(
                    Isometry2d::from_translation(world.truncate()),
                    size,
                    Color::srgb(1.0, 0.5, 0.0),
                );
            }
        }
    }
}
