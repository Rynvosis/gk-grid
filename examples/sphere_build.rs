//! Sphere building demo: an icosphere grid stacked into radial shells over a rendered sphere, blocks
//! placed on the surface. Ground-as-surface: the store starts empty, so the base sphere itself is the
//! ground a first block lands on. Left click places, right click deletes. Orbit with arrow keys or WASD.

use bevy::{picking::pointer::PointerInteraction, prelude::*};
use gk_grid::prelude::{tilemap_gizmo::UniformTilemapGizmo, *};

#[path = "common/orbit.rs"]
mod orbit;
use orbit::{Orbit, orbit_camera};

const RADIUS: f32 = 1.0;
// Bevy's `.ico()` is hexasphere too, so the same count makes the grid faces the rendered triangles.
const SUBDIVISIONS: u32 = 1;
const SHELL_THICKNESS: f32 = 0.25;

// Only filled cells live in the store; presence there is what makes a cell solid, drawn, and pickable.
type BlockStore = SparseTileStore<LayeredCell<usize>, ()>;
type SphereGeometry = LayeredGeometry<RadialMeshGeometry>;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GridGizmoPlugin::<BlockStore, SphereGeometry>::default())
        .add_plugins(GridPickingPlugin::<BlockStore, SphereGeometry>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, highlight_hovered_cell, build_on_click))
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let sphere = meshes.add(Sphere::new(RADIUS).mesh().ico(SUBDIVISIONS).unwrap());
    commands.spawn((
        Mesh3d(sphere),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.8),
            ..default()
        })),
        Transform::default(),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Orbit::new(5.0),
    ));

    // Generate the base sphere grid straight from parameters, then stack it into shells.
    let (base_grid, base_geometry) = RadialMeshGeometry::ico_sphere(Vec3::ZERO, RADIUS, SUBDIVISIONS);
    let grid = LayeredGrid::new(base_grid);
    let geometry = SphereGeometry::new(base_geometry, SHELL_THICKNESS);
    let grid_entity = commands.spawn((grid, geometry)).id();

    // Ground-as-surface: no prefilled ground layer; only placed blocks live in the store.
    commands.spawn((
        BlockStore::new(),
        UniformTilemapGizmo {
            color: Color::srgba(1.0, 1.0, 1.0, 0.4),
        },
        PickableCells::<BlockStore> {
            pickable: |store, cell| store.get(*cell).is_some(),
        },
        TilemapOf(grid_entity),
    ));
}

/// Outlines the cell under each pointer from the hit the picking backend reports.
fn highlight_hovered_cell(
    mut gizmos: Gizmos,
    pointers: Query<&PointerInteraction>,
    grids: Query<(&SphereGeometry, Option<&Transform>)>,
) {
    let Ok((geometry, transform)) = grids.single() else {
        return;
    };
    for interaction in &pointers {
        let Some(hit) = interaction
            .iter()
            .find_map(|(_, hit)| hit.extra_as::<RayHitOf<LayeredGrid<GraphGrid>>>())
        else {
            continue;
        };
        let Some(corners) = geometry.try_cell_corners(hit.cell) else {
            continue;
        };
        // Lift the outline clear of the block's own faces, or it z-fights them. The base geometry's
        // normal field is what "outward" means here, so it does the lifting.
        let base = geometry.base();
        let lifted = corners.map(|(_, local)| base.lift(local, RADIUS * 0.01));
        draw_cell_outline(&mut gizmos, transform, lifted, Color::srgb(1.0, 0.5, 0.0));
    }
}

/// Places a block against the face clicked, or deletes the block clicked.
///
/// The picking backend only sees cells in the store, so with the store empty there is nothing to hit
/// and the first click has to come from somewhere else. That is the ground: the base sphere is a
/// surface, so piercing it names the face under the cursor, and layer 0 sits on top of it.
fn build_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    pointers: Query<&PointerInteraction>,
    ray_map: Res<bevy::picking::backend::ray::RayMap>,
    grids: Query<(&LayeredGrid<GraphGrid>, &SphereGeometry, Option<&Transform>)>,
    mut stores: Query<&mut BlockStore>,
) {
    let place = mouse.just_pressed(MouseButton::Left);
    let delete = mouse.just_pressed(MouseButton::Right);
    if !place && !delete {
        return;
    }
    let (Ok((grid, geometry, transform)), Ok(mut store)) = (grids.single(), stores.single_mut()) else {
        return;
    };

    // A block under the cursor: place against the face the ray came in through, delete the block itself.
    let block_hit = pointers.iter().find_map(|interaction| {
        interaction
            .iter()
            .find_map(|(_, hit)| hit.extra_as::<RayHitOf<LayeredGrid<GraphGrid>>>())
    });
    if let Some(hit) = block_hit {
        if delete {
            store.remove(hit.cell);
        } else if let Some(against) = hit.face.and_then(|slot| grid.try_neighbour(hit.cell, slot)) {
            store.insert(against, ());
        }
        return;
    }

    // Ground: no block in the way, so the ray falls through to the sphere itself.
    if !place {
        return;
    }
    for (_, ray) in ray_map.iter() {
        // Geometry works in the grid's local space, so the ray has to be brought into it, exactly as
        // the picking backend does for the hits above.
        let (origin, dir) = world_ray_to_local(transform, ray.origin, *ray.direction);
        let Some((_, point)) = geometry.base().pierce(origin, dir) else {
            continue;
        };
        if let Some(face) = geometry.base().cells_at(point).next() {
            store.insert(LayeredCell::new(face, 0), ());
        }
    }
}
