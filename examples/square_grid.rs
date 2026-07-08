use bevy::prelude::*;
use gk_grid::prelude::tilemap_gizmo::TilemapGizmo;
use gk_grid::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GridGizmoPlugin::<Dense<RectRegion, ()>, QuadGridGeometry>::default())
        .add_systems(Startup, setup)
        .run();
}
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    let grid = commands.spawn(QuadGridGeometry::rect_grid(Vec2::splat(32.0))).id();
    let region = RectRegion::new(IVec2::splat(-10), UVec2::splat(20));
    commands.spawn((
        Dense::from_region(region, |_| ()),
        TilemapGizmo {
            color: Color::WHITE,
        },
        TilemapOf(grid),
    ));
}
