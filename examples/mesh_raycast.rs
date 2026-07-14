//! Mesh picking: a grid picking backend makes the icosphere's faces pickable, and a per-frame
//! system reads the hover state to outline the face under the cursor. Orbit with the arrow keys or WASD.

use bevy::{picking::pointer::PointerInteraction, prelude::*};
use gk_grid::prelude::{tilemap_gizmo::UniformTilemapGizmo, *};

// Shared between the rendered sphere and the grid build, so the wireframe lands on the surface.
const RADIUS: f32 = 1.0;
const SUBDIVISIONS: u32 = 1;

// One `()` tile per face; the gizmo and the picking backend both key on this store.
type FaceStore = DenseTileStore<FaceRegion, ()>;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GridGizmoPlugin::<FaceStore, Mesh3DGridGeometry>::default())
        .add_plugins(SurfacePickingPlugin::<FaceStore, Mesh3DGridGeometry>::default())
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
    let (grid, geometry) = GraphGrid::from_mesh(meshes.get(&sphere).unwrap()).unwrap();
    let region = grid.faces_region();
    let grid_entity = commands.spawn((grid, geometry)).id();

    // A tile per face drawn as a faint white wireframe; `PickableCells` opts every face into picking.
    commands.spawn((
        FaceStore::from_region(region, |_| ()),
        UniformTilemapGizmo {
            color: Color::srgba(1.0, 1.0, 1.0, 0.1),
        },
        PickableCells::<FaceStore>::all(),
        TilemapOf(grid_entity),
    ));
}

/// Reads the picking hover state and outlines the face the backend reports under each pointer.
fn highlight_hovered_face(
    mut gizmos: Gizmos,
    pointers: Query<&PointerInteraction>,
    grids: Query<(&Mesh3DGridGeometry, Option<&Transform>)>,
) {
    let Ok((geom, transform)) = grids.single() else {
        return;
    };
    for interaction in &pointers {
        // Nearest hit that carries our grid's `RayHit` (skips any other backend's hits).
        let Some(hit) = interaction
            .iter()
            .find_map(|(_, hit)| hit.extra_as::<RayHitOf<GraphGrid>>())
        else {
            continue;
        };
        let Some(corners) = geom.try_cell_corners(hit.cell) else {
            continue;
        };
        // Lift the outline off the wireframe to avoid z-fighting; assumes the sphere is centred at the origin.
        let lifted = corners.map(|(_, local)| local * 1.01);
        draw_cell_outline(&mut gizmos, transform, lifted, Color::srgb(1.0, 0.5, 0.0));
    }
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
