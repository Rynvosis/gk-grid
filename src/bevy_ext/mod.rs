//! Bevy integration layer: everything gated behind the `bevy` feature (gizmos, the tilemap
//! relations, the tile reader system param, and the picking backend).

pub(crate) mod gizmos;
pub(crate) mod picking;
pub(crate) mod relations;
pub(crate) mod tiles;

use bevy::prelude::*;

/// A world-space ray mapped into a grid's local space through an optional `Transform`.
/// The direction is re-normalized, so a raycast `t` comes back as a distance in local space.
pub fn world_ray_to_local(transform: Option<&Transform>, origin: Vec3, dir: Vec3) -> (Vec3, Vec3) {
    let inverse = transform.unwrap_or(&Transform::IDENTITY).to_matrix().inverse();
    (
        inverse.transform_point3(origin),
        inverse.transform_vector3(dir).normalize(),
    )
}
