//! A mesh grid built from a Bevy icosphere, drawn as a wireframe over the rendered sphere.
//! Orbit the camera with the arrow keys or WASD.

use bevy::prelude::*;
use gk_grid::prelude::{tilemap_gizmo::UniformTilemapGizmo, *};

#[path = "common/orbit.rs"]
mod orbit;
use orbit::{Orbit, orbit_camera};

// Shared between the rendered sphere and the grid build, so the wireframe lands on the surface.
const RADIUS: f32 = 1.0;
const SUBDIVISIONS: u32 = 1; // base icosahedron: 20 faces, 12 verts

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GridGizmoPlugin::<DenseTileStore<FaceRegion, ()>, Mesh3DGridGeometry>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, orbit_camera)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let sphere = meshes.add(Sphere::new(RADIUS).mesh().ico(SUBDIVISIONS).unwrap());
    commands.spawn((
        Mesh3d(sphere.clone()),
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

    // Build the grid from the same icosphere the sphere renders, so the wireframe lands on the surface.
    let (grid, geometry) = GraphGrid::from_mesh(meshes.get(&sphere).unwrap()).unwrap();
    let region = grid.faces_region();

    // A dense tilemap over every face, so the gizmo draws the whole surface.
    let map = DenseTileStore::from_region(region, |_| ());
    let grid_entity = commands.spawn((grid, geometry)).id();
    commands.spawn((map, UniformTilemapGizmo { color: Color::WHITE }, TilemapOf(grid_entity)));
}
