//! A `bevy_picking` backend that makes grid tilemaps first-class pickable entities and hands the
//! consumer the cell that was hit (via `HitData`'s `extra` slot), not just the entity.

use std::marker::PhantomData;

use bevy::{
    camera::visibility::RenderLayers,
    picking::{
        PickingSystems,
        backend::{HitData, PointerHits, ray::RayMap},
    },
    prelude::*,
};

use crate::{
    grid::{
        Grid,
        geometry::{GridGeometry, RayCast},
    },
    prelude::{TileStore, TilemapOf},
};

/// Marks a tilemap as pickable and selects which of its cells can be hit.
#[derive(Component, Debug)]
pub struct PickableCells<S: TileStore + Component> {
    /// Whether a given cell of the store may be picked.
    pub pickable: fn(&S, &S::Cell) -> bool,
}

impl<S: TileStore + Component> PickableCells<S> {
    /// Every cell is pickable, so the front-most cell the ray crosses wins.
    pub fn all() -> Self {
        Self { pickable: |_, _| true }
    }
}

/// Adds the grid picking backend for tilemaps whose data is `S` and whose grid is drawn by geometry `G`.
#[derive(Debug)]
pub struct GridPickingPlugin<S, G>(PhantomData<(S, G)>);

impl<S, G> Default for GridPickingPlugin<S, G> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<S, G> Plugin for GridPickingPlugin<S, G>
where
    S: TileStore + Component,
    G: Component + GridGeometry<Position = Vec3> + RayCast,
    G::Grid: Grid<Cell = S::Cell>,
{
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, grid_picking_backend::<S, G>.in_set(PickingSystems::Backend));
    }
}

/// Reports the nearest pickable cell under each pointer as a `bevy_picking` hit, carrying the core
/// `RayHit` as `HitData` extra so an observer can recover it with `hit.extra_as::<RayHitOf<G::Grid>>()`.
///
/// Emits exactly ONE hit per tilemap: the hover state (`HoverMap`) keys per entity, so multiple
/// cells along the ray for the same tilemap would collapse anyway. Whole-column traversal stays an
/// immediate-mode `raycast` job.
#[allow(clippy::type_complexity)]
pub(crate) fn grid_picking_backend<S, G>(
    ray_map: Res<RayMap>,
    cameras: Query<(&Camera, &Projection, Option<&RenderLayers>)>,
    tilemaps: Query<(Entity, &S, &TilemapOf, &PickableCells<S>, Option<&RenderLayers>)>,
    grids: Query<(&G, Option<&Transform>)>,
    mut output: MessageWriter<PointerHits>,
) where
    S: TileStore + Component,
    G: Component + GridGeometry<Position = Vec3> + RayCast,
    G::Grid: Grid<Cell = S::Cell>,
{
    for (&ray_id, &ray) in ray_map.iter() {
        let Ok((camera, projection, cam_layers)) = cameras.get(ray_id.camera) else {
            continue;
        };

        let cam_layers = cam_layers.unwrap_or_default();
        // Bound the (possibly infinite) cell march at the camera far plane.
        let max_distance = projection.far();

        for (tilemap_entity, store, tilemap_of, pickable_component, maybe_layers) in tilemaps.iter() {
            let layers = maybe_layers.unwrap_or_default();
            if !cam_layers.intersects(layers) {
                continue;
            }

            let Ok((geometry, maybe_grid_transform)) = grids.get(tilemap_of.0) else {
                continue;
            };

            if let Some(hit_data) = cast_ray_into_tilemap(
                ray_id.camera,
                ray,
                max_distance,
                geometry,
                maybe_grid_transform,
                store,
                pickable_component.pickable,
            ) {
                output.write(PointerHits::new(
                    ray_id.pointer,
                    vec![(tilemap_entity, hit_data)],
                    camera.order as f32,
                ));
            }
        }
    }
}

/// Casts a world-space ray into one tilemap's grid and returns the picking hit for the nearest
/// pickable cell within `max_distance`, with the whole `RayHit` (cell, `t`, face) carried as `HitData` extra.
fn cast_ray_into_tilemap<S, G>(
    camera: Entity,
    ray: Ray3d,
    max_distance: f32,
    geometry: &G,
    grid_transform: Option<&Transform>,
    store: &S,
    pickable: fn(&S, &S::Cell) -> bool,
) -> Option<HitData>
where
    S: TileStore,
    G: GridGeometry<Position = Vec3> + RayCast,
    G::Grid: Grid<Cell = S::Cell>,
{
    let world_to_local = grid_transform
        .unwrap_or(&Transform::IDENTITY)
        .compute_affine()
        .inverse();
    let local_origin = world_to_local.transform_point3(ray.origin);
    let local_dir = world_to_local.transform_vector3(ray.direction.as_vec3());

    // The local dir is not renormalized, so raycast `t` is the world distance along the ray, which is
    // both the `max_distance` bound (the march may be infinite) and the picking depth.
    let hit = geometry
        .raycast(local_origin, local_dir)
        .take_while(|hit| hit.t <= max_distance)
        .find(|hit| pickable(store, &hit.cell))?;

    Some(HitData::new_with_extra(
        camera,
        hit.t,
        Some(ray.get_point(hit.t)),
        None,
        hit,
    ))
}
