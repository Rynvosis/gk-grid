use bevy::prelude::*;
use gk_grid::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GridGizmoPlugin::<Dense<RectRegion, ()>, SquareGrid>::default())
        .add_systems(Startup, setup)
        .run();
}
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    let grid = commands.spawn(SquareGrid::new(Vec2::splat(16.0))).id();
    let layout = SquareChunkLayout::new(UVec2::splat(16));
    for (chunk_coord, color) in [
        (IVec2::new(-1, 0), bevy::color::palettes::css::RED),
        (IVec2::new(-1, -1), bevy::color::palettes::css::BLUE),
        (IVec2::new(0, -1), bevy::color::palettes::css::GREEN),
        (IVec2::new(0, 0), bevy::color::palettes::css::YELLOW),
    ] {
        commands.spawn((
            Dense::from_region(layout.chunk_region(chunk_coord), |_| ()),
            GridGizmo {
                color: Color::Srgba(color),
            },
            TilemapOf(grid),
        ));
    }
}
