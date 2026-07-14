//! `bevy_picking` backends that make grid tilemaps first-class pickable entities and hand the
//! consumer the cell that was hit (via `HitData`'s `extra` slot), not just the entity. Two paths:
//! `GridPickingPlugin` marches a `RayCast` grid; `SurfacePickingPlugin` pierces a `Surface` grid.

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
        geometry::{GridGeometry, PointQuery, RayCast, RayHit, RayHitOf, Surface},
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

/// Picking backend for tilemaps of `S` drawn by `RayCast` geometry `G`: reports the nearest
/// pickable cell the ray crosses.
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
    G::Grid: Grid<Cell = S::Cell> + Component,
{
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, grid_picking_backend::<S, G>.in_set(PickingSystems::Backend));
    }
}

/// Picking backend for tilemaps of `S` drawn by `Surface` geometry `G`: reports the cell the ray
/// first touches (pierce, then `cells_at`).
#[derive(Debug)]
pub struct SurfacePickingPlugin<S, G>(PhantomData<(S, G)>);

impl<S, G> Default for SurfacePickingPlugin<S, G> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<S, G> Plugin for SurfacePickingPlugin<S, G>
where
    S: TileStore + Component,
    G: Component + GridGeometry<Position = Vec3> + Surface + PointQuery,
    G::Grid: Grid<Cell = S::Cell> + Component,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            surface_picking_backend::<S, G>.in_set(PickingSystems::Backend),
        );
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn grid_picking_backend<S, G>(
    ray_map: Res<RayMap>,
    cameras: Query<(&Camera, &Projection, Option<&RenderLayers>)>,
    tilemaps: Query<(Entity, &S, &TilemapOf, &PickableCells<S>, Option<&RenderLayers>)>,
    grids: Query<(&G::Grid, &G, Option<&Transform>)>,
    output: MessageWriter<PointerHits>,
) where
    S: TileStore + Component,
    G: Component + GridGeometry<Position = Vec3> + RayCast,
    G::Grid: Grid<Cell = S::Cell> + Component,
{
    run_backend(ray_map, cameras, tilemaps, grids, output, cast_ray_into_tilemap::<S, G>);
}

#[allow(clippy::type_complexity)]
pub(crate) fn surface_picking_backend<S, G>(
    ray_map: Res<RayMap>,
    cameras: Query<(&Camera, &Projection, Option<&RenderLayers>)>,
    tilemaps: Query<(Entity, &S, &TilemapOf, &PickableCells<S>, Option<&RenderLayers>)>,
    grids: Query<(&G::Grid, &G, Option<&Transform>)>,
    output: MessageWriter<PointerHits>,
) where
    S: TileStore + Component,
    G: Component + GridGeometry<Position = Vec3> + Surface + PointQuery,
    G::Grid: Grid<Cell = S::Cell> + Component,
{
    run_backend(
        ray_map,
        cameras,
        tilemaps,
        grids,
        output,
        cast_pierce_into_tilemap::<S, G>,
    );
}

/// One ray against one tilemap, with the ray already in the grid's local space.
struct Cast<'a, S: TileStore, G: GridGeometry> {
    camera: Entity,
    world_ray: Ray3d,
    /// The same ray in the grid's local space, where the geometry works. `local_dir` is deliberately
    /// left unnormalized so `t` stays a world distance.
    local_origin: Vec3,
    local_dir: Vec3,
    max_distance: f32,
    grid: &'a G::Grid,
    geometry: &'a G,
    store: &'a S,
    pickable: fn(&S, &S::Cell) -> bool,
}

/// Emits exactly one `PointerHits` per tilemap: `HoverMap` keys per entity, so multiple cells along
/// the ray would collapse anyway. Whole-column traversal stays an immediate-mode `raycast` job.
#[allow(clippy::type_complexity)]
fn run_backend<S, G>(
    ray_map: Res<RayMap>,
    cameras: Query<(&Camera, &Projection, Option<&RenderLayers>)>,
    tilemaps: Query<(Entity, &S, &TilemapOf, &PickableCells<S>, Option<&RenderLayers>)>,
    grids: Query<(&G::Grid, &G, Option<&Transform>)>,
    mut output: MessageWriter<PointerHits>,
    cast: impl Fn(&Cast<S, G>) -> Option<HitData>,
) where
    S: TileStore + Component,
    G: Component + GridGeometry<Position = Vec3>,
    G::Grid: Grid<Cell = S::Cell> + Component,
{
    for (&ray_id, &ray) in ray_map.iter() {
        let Ok((camera, projection, cam_layers)) = cameras.get(ray_id.camera) else {
            continue;
        };

        let cam_layers = cam_layers.unwrap_or_default();

        for (tilemap_entity, store, tilemap_of, pickable_component, maybe_layers) in tilemaps.iter() {
            let layers = maybe_layers.unwrap_or_default();
            if !cam_layers.intersects(layers) {
                continue;
            }

            let Ok((grid, geometry, grid_transform)) = grids.get(tilemap_of.0) else {
                continue;
            };

            let (local_origin, local_dir) = local_ray(grid_transform, ray);
            let hit = cast(&Cast {
                camera: ray_id.camera,
                world_ray: ray,
                local_origin,
                local_dir,
                // Bound the (possibly infinite) cell march at the camera far plane.
                max_distance: projection.far(),
                grid,
                geometry,
                store,
                pickable: pickable_component.pickable,
            });

            if let Some(hit_data) = hit {
                output.write(PointerHits::new(
                    ray_id.pointer,
                    vec![(tilemap_entity, hit_data)],
                    camera.order as f32,
                ));
            }
        }
    }
}

/// Maps a world ray into a grid's local space, leaving the direction unnormalized so raycast `t`
/// is a world distance (unlike the public `world_ray_to_local`, which renormalizes).
fn local_ray(grid_transform: Option<&Transform>, ray: Ray3d) -> (Vec3, Vec3) {
    let world_to_local = grid_transform
        .unwrap_or(&Transform::IDENTITY)
        .compute_affine()
        .inverse();
    (
        world_to_local.transform_point3(ray.origin),
        world_to_local.transform_vector3(ray.direction.as_vec3()),
    )
}

/// Marches the local ray and returns the nearest pickable cell within `max_distance`, its `RayHit`
/// (cell, `t`, face) carried as `HitData` extra.
fn cast_ray_into_tilemap<S, G>(cast: &Cast<S, G>) -> Option<HitData>
where
    S: TileStore,
    G: GridGeometry<Position = Vec3> + RayCast,
    G::Grid: Grid<Cell = S::Cell>,
{
    let hit = cast
        .geometry
        .raycast(cast.grid, cast.local_origin, cast.local_dir)
        .take_while(|hit| hit.t <= cast.max_distance)
        .find(|hit| (cast.pickable)(cast.store, &hit.cell))?;

    Some(HitData::new_with_extra(
        cast.camera,
        hit.t,
        Some(cast.world_ray.get_point(hit.t)),
        None,
        hit,
    ))
}

/// Pierces the local ray and returns the first-touched pickable cell within `max_distance`, a
/// `RayHit` (cell, `t`, `face: None`) carried as `HitData` extra.
///
/// A pierce touches one cell and never crosses a boundary, so it reads no topology off the grid.
fn cast_pierce_into_tilemap<S, G>(cast: &Cast<S, G>) -> Option<HitData>
where
    S: TileStore,
    G: GridGeometry<Position = Vec3> + Surface + PointQuery,
    G::Grid: Grid<Cell = S::Cell>,
{
    let (t, point) = cast.geometry.pierce(cast.local_origin, cast.local_dir)?;
    if t > cast.max_distance {
        return None;
    }
    // First touch only: an unpickable front face hides whatever sits behind it.
    let cell = cast
        .geometry
        .cells_at(point)
        .find(|cell| (cast.pickable)(cast.store, cell))?;

    let hit: RayHitOf<G::Grid> = RayHit { cell, t, face: None };
    Some(HitData::new_with_extra(
        cast.camera,
        t,
        Some(cast.world_ray.get_point(t)),
        None,
        hit,
    ))
}
