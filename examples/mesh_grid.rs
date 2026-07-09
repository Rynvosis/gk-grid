//! A mesh grid built from a Bevy icosphere, drawn as a wireframe over the rendered sphere.
//! Orbit the camera with the arrow keys or WASD.

use bevy::prelude::*;
use gk_grid::prelude::tilemap_gizmo::TilemapGizmo;
use gk_grid::prelude::*;

// Shared between the rendered sphere and the grid build, so the wireframe lands on the surface.
const RADIUS: f32 = 1.0;
const SUBDIVISIONS: u32 = 1; // base icosahedron: 20 faces, 12 verts

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GridGizmoPlugin::<Dense<FaceRegion, ()>, MeshGridGeometry>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, orbit_camera)
        .run();
}

#[derive(Component)]
struct Orbit {
    yaw: f32,
    pitch: f32,
    radius: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
        Orbit {
            yaw: 0.0,
            pitch: 0.3,
            radius: 5.0,
        },
    ));

    // Build the grid from the same icosphere the sphere renders, so the wireframe overlays it.
    let (grid, geometry) = MeshGrid::from_mesh(meshes.get(&sphere).unwrap());

    // A dense tilemap covering every face, so the gizmo draws all of them.
    let map = Dense::from_region(grid.faces_region(), |_| ());
    let grid_entity = commands.spawn((grid, geometry)).id();
    commands.spawn((
        map,
        TilemapGizmo {
            color: Color::WHITE,
        },
        TilemapOf(grid_entity),
    ));
}

fn orbit_camera(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera: Query<(&mut Orbit, &mut Transform)>,
) {
    let Ok((mut orbit, mut transform)) = camera.single_mut() else {
        return;
    };
    let speed = 1.5 * time.delta_secs();
    if keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        orbit.yaw -= speed;
    }
    if keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        orbit.yaw += speed;
    }
    if keys.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        orbit.pitch += speed;
    }
    if keys.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        orbit.pitch -= speed;
    }
    // Clamp short of the poles so `looking_at` doesn't degenerate when the view aligns with up.
    orbit.pitch = orbit.pitch.clamp(-1.5, 1.5);

    let pos = Vec3::new(
        orbit.radius * orbit.pitch.cos() * orbit.yaw.sin(),
        orbit.radius * orbit.pitch.sin(),
        orbit.radius * orbit.pitch.cos() * orbit.yaw.cos(),
    );
    *transform = Transform::from_translation(pos).looking_at(Vec3::ZERO, Vec3::Y);
}
