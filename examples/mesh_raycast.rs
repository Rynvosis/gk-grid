//! Mesh picking on an arbitrary mesh: the Khronos Fox's 576 triangles are the grid's cells, and the
//! surface picking backend reports the face under the cursor, which a per-frame system outlines.
//! The mesh loads asynchronously, so the grid is built the first frame it is available.
//! Orbit the camera with the arrow keys or WASD.

use bevy::{gltf::GltfAssetLabel, picking::pointer::PointerInteraction, prelude::*};
use gk_grid::prelude::*;

#[path = "common/orbit.rs"]
mod orbit;
use orbit::{Orbit, orbit_camera};

// One `()` tile per face; the picking backend keys on this store.
type FaceStore = DenseTileStore<FaceRegion, ()>;

/// The mesh arrives asynchronously, so the grid cannot be built during `Startup`.
#[derive(Resource)]
struct FoxMesh(Handle<Mesh>);

/// The Fox is ~155 units long in its own space. The rendered mesh and the grid must carry the same
/// transform, or the picking backend's world-to-local ray lands on the wrong faces.
fn fox_transform() -> Transform {
    Transform::from_scale(Vec3::splat(0.02)).with_translation(Vec3::new(0.0, -0.79, 0.21))
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SurfacePickingPlugin::<FaceStore, Mesh3DGridGeometry>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (build_grid_once_loaded, orbit_camera, highlight_hovered_face))
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    // The mesh is only rendered once its joints are stripped, so nothing spawns it here.
    let mesh: Handle<Mesh> =
        assets.load(GltfAssetLabel::Primitive { mesh: 0, primitive: 0 }.from_asset("models/fox.glb"));
    commands.insert_resource(FoxMesh(mesh));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Orbit::new(5.0),
    ));
}

/// Builds the grid, and spawns the rendered mesh, the first frame the asset has loaded. Welding
/// merges the mesh's split vertices back onto shared ids, which is what gives the faces adjacency.
///
/// The fox is a skinned mesh and we only want its bind pose. Bevy picks the skinned render pipeline
/// off the joint attributes, and a `Mesh3d` with no skeleton then binds the wrong group, which is a
/// validation error that kills the app; so the joints are stripped before anything renders it.
fn build_grid_once_loaded(
    mut commands: Commands,
    fox: Res<FoxMesh>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut built: Local<bool>,
) {
    if *built {
        return;
    }
    let Some(mut mesh) = meshes.get_mut(&fox.0) else {
        return;
    };
    mesh.remove_attribute(Mesh::ATTRIBUTE_JOINT_INDEX);
    mesh.remove_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT);

    let (grid, geometry) = GraphGrid::from_mesh(&mesh).expect("the fox welds into an edge-manifold surface");
    let region = grid.faces_region();
    let grid_entity = commands.spawn((grid, geometry, fox_transform())).id();

    // A tile per face; `PickableCells` opts every one of them into picking.
    commands.spawn((
        FaceStore::from_region(region, |_| ()),
        PickableCells::<FaceStore>::all(),
        TilemapOf(grid_entity),
    ));

    commands.spawn((
        Mesh3d(fox.0.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.4, 0.2),
            ..default()
        })),
        fox_transform(),
    ));
    *built = true;
}

/// Reads the picking hover state and outlines the face the backend reports under each pointer.
fn highlight_hovered_face(
    mut gizmos: Gizmos,
    pointers: Query<&PointerInteraction>,
    grids: Query<(&Mesh3DGridGeometry, Option<&Transform>)>,
) {
    let Ok((geometry, transform)) = grids.single() else {
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
        let Some(corners) = geometry.try_cell_corners(hit.cell) else {
            continue;
        };

        // Lift the outline along the face's own normal rather than out from the origin: the fox is
        // concave, so an origin-scaled outline would sink into the legs and muzzle. Left flush with
        // the surface it loses the depth test, so it needs the lift. Scaling the lift by the face's
        // own size keeps it under the model's local scale.
        let corners: Vec<Vec3> = corners.map(|(_, local)| local).collect();
        let [a, b, c] = corners[..] else { continue };
        let lift = (b - a).cross(c - a).normalize() * (b - a).length() * 0.005;
        let lifted = corners.iter().map(|&corner| corner + lift);

        draw_cell_outline(&mut gizmos, transform, lifted, Color::srgb(1.0, 0.5, 0.0));
    }
}
