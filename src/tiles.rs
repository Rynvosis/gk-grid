use bevy::{ecs::system::SystemParam, prelude::*};

use crate::prelude::*;

/// Reads tile stores of type `S` from the world.
#[derive(SystemParam)]
#[allow(missing_debug_implementations)] // wraps a Query, which isn't Debug
pub struct Tiles<'w, 's, S: TileStore + Component> {
    stores: Query<'w, 's, &'static S>,
}

impl<S: TileStore + Component> Tiles<'_, '_, S> {
    /// Value at a cell of one store entity, or None if the entity or cell is missing.
    pub fn get(&self, map: Entity, cell: S::Cell) -> Option<&S::Item> {
        self.stores.get(map).ok()?.get(cell)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Map = Dense<RectRegion, i32>;

    #[derive(Resource)]
    struct Target(Entity);

    #[derive(Resource, Default)]
    struct Seen(Option<i32>);

    fn read_cell(tiles: Tiles<Map>, target: Res<Target>, mut seen: ResMut<Seen>) {
        seen.0 = tiles.get(target.0, IVec2::new(2, 1)).copied();
    }

    // The param reads a dense store: cell (2,1) of x*10+y is 21.
    #[test]
    fn tiles_param_reads_a_dense_store() {
        let mut app = App::new();
        app.init_resource::<Seen>();

        let region = RectRegion::new(IVec2::ZERO, UVec2::splat(3));
        let map = app
            .world_mut()
            .spawn(Dense::from_region(region, |c| c.x * 10 + c.y))
            .id();
        app.insert_resource(Target(map));
        app.add_systems(Update, read_cell);

        app.update();
        assert_eq!(app.world().resource::<Seen>().0, Some(21));
    }
}
