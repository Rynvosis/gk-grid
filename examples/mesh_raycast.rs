//! Mesh raycast picking: casts a ray from the cursor through the camera into an icosphere grid
//! and outlines the face under the cursor. Orbit the camera with the arrow keys or WASD.

use bevy::prelude::*;
use gk_grid::prelude::{tilemap_gizmo::UniformTilemapGizmo, *};

// Shared between the rendered sphere and the grid build, so the wireframe lands on the surface.
const RADIUS: f32 = 1.0;
const SUBDIVISIONS: u32 = 1;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GridGizmoPlugin::<DenseTileStore<FaceRegion, ()>, MeshGridGeometry>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, highlight_hovered_face))
        .run();
}

#[derive(Component)]
struct Orbit {
    yaw: f32,
    pitch: f32,
    radius: f32,
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
        Orbit {
            yaw: 0.0,
            pitch: 0.3,
            radius: 5.0,
        },
    ));

    // Build the grid from the same icosphere the sphere renders, so the wireframe lands on the surface.
    let (grid, geometry) = MeshGrid::from_mesh(meshes.get(&sphere).unwrap());
    let region = grid.faces_region();
    let grid_entity = commands.spawn((grid, geometry)).id();

    // A tile per face drawn as a faint white wireframe; the picker recolours the hovered face.
    commands.spawn((
        DenseTileStore::from_region(region, |_| ()),
        UniformTilemapGizmo {
            color: Color::srgba(1.0, 1.0, 1.0, 0.1),
        },
        TilemapOf(grid_entity),
    ));
}

/// Casts the cursor ray into the grid and outlines the nearest face it hits in orange.
fn highlight_hovered_face(
    mut gizmos: Gizmos,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    grids: Query<(&MeshGridGeometry, Option<&Transform>)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    let Ok((geom, transform)) = grids.single() else {
        return;
    };

    let (origin, dir) = world_ray_to_local(transform, ray.origin, Vec3::from(ray.direction));
    let Some(hit) = geom.raycast(origin, dir).next() else {
        return;
    };
    let Some(corners) = geom.try_cell_corners(hit.cell) else {
        return;
    };

    // lift the outline off the wireframe to avoid z-fighting; assumes the sphere is centred at the origin.
    let lifted = corners.map(|(_, local)| local * 1.01);
    draw_face_outline(&mut gizmos, transform, lifted, Color::srgb(1.0, 0.5, 0.0));
}

fn orbit_camera(keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut camera: Query<(&mut Orbit, &mut Transform)>) {
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

/// World-space ray -> the grid's local space, through an optional Transform.
fn world_ray_to_local(transform: Option<&Transform>, origin: Vec3, dir: Vec3) -> (Vec3, Vec3) {
    let inverse = transform.unwrap_or(&Transform::IDENTITY).to_matrix().inverse();
    (inverse.transform_point3(origin), inverse.transform_vector3(dir).normalize())
}

/// Draws a closed polyline through `corners` (grid-local space), transformed to world.
fn draw_face_outline(
    gizmos: &mut Gizmos,
    transform: Option<&Transform>,
    corners: impl Iterator<Item = Vec3>,
    color: Color,
) {
    let transform = transform.copied().unwrap_or_default();
    let mut corners = corners.map(|local| transform.transform_point(local));
    let Some(first) = corners.next() else {
        return;
    };
    let mut prev = first;
    for corner in corners {
        gizmos.line(prev, corner, color);
        prev = corner;
    }
    gizmos.line(prev, first, color);
}
