use bevy::prelude::*;
use gk_grid::prelude::cell_gizmo::TilemapCellGizmos;
use gk_grid::prelude::*;

type ColorMap = Dense<RectRegion, Color>;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GridGizmoPlugin::<ColorMap, QuadGridGeometry>::default())
        .add_systems(Startup, setup)
        .run();
}
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    let grid = commands
        .spawn(QuadGridGeometry::rect_grid(Vec2::splat(16.0)))
        .id();
    let layout = QuadChunkLayout::new(UVec2::splat(16));
    for (chunk_coord, color) in [
        (IVec2::new(-1, 0), bevy::color::palettes::css::RED),
        (IVec2::new(-1, -1), bevy::color::palettes::css::BLUE),
        (IVec2::new(0, -1), bevy::color::palettes::css::GREEN),
        (IVec2::new(0, 0), bevy::color::palettes::css::YELLOW),
    ] {
        commands.spawn((
            Dense::from_region(layout.chunk_region(chunk_coord), |cell| {
                Color::Srgba(color * cell_value(cell))
            }),
            TilemapCellGizmos::<ColorMap> {
                color_fn: |map, cell| *map.get(*cell).unwrap(),
            },
            TilemapOf(grid),
        ));
    }
}

// 0..1 sine field, seamless across chunk borders since cells are global
fn cell_value(global_cell: IVec2) -> f32 {
    (((global_cell.x + global_cell.y) as f32 / 9.0 * std::f32::consts::TAU).sin() + 1.0) * 0.5
}
