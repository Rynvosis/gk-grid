use bevy::prelude::*;
use gk_grid::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GridGizmoPlugin::<SquareTilemap, SquareGrid>::default())
        .add_systems(Startup, setup)
        .run();
}
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    let grid = commands.spawn(SquareGrid::new(Vec2::splat(32.0))).id();
    commands.spawn((
        SquareTilemap {
            region: RectRegion::new(IVec2::splat(-10), UVec2::splat(20)),
        },
        GridGizmo {
            color: Color::WHITE,
        },
        TilemapOf(grid),
    ));
}
