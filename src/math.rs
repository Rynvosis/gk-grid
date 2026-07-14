//! Small geometric helpers shared across grid geometries.

use glam::Vec3;

/// Unit normal of a face from its first three corners, or `None` if they are degenerate.
pub(crate) fn face_normal(face: &[usize], verts: &[Vec3]) -> Option<Vec3> {
    let raw = (verts[face[0]] - verts[face[1]]).cross(verts[face[2]] - verts[face[1]]);
    // 1e-12 = (1e-6)^2: the is-this-zero guard in squared units.
    (raw.length_squared() >= 1e-12).then(|| raw.normalize())
}
