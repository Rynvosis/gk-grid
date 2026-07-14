use std::collections::HashMap;

use bevy::mesh::{Mesh, PrimitiveTopology};
use glam::Vec3;

use crate::graph::{GraphGrid, NonManifoldError, geometry::Mesh3DGridGeometry};

impl GraphGrid {
    /// Builds a mesh grid and its geometry from a Bevy triangle mesh, welding vertices that share a position.
    ///
    /// Panics if the mesh is not a `TriangleList` or lacks `Float32x3` positions;
    /// returns [`NonManifoldError`] if the welded mesh is not edge-manifold.
    pub fn from_mesh(mesh: &Mesh) -> Result<(GraphGrid, Mesh3DGridGeometry), NonManifoldError> {
        assert_eq!(
            mesh.primitive_topology(),
            PrimitiveTopology::TriangleList,
            "from_mesh expects a TriangleList mesh",
        );
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|attr| attr.as_float3())
            .expect("mesh has Float32x3 positions");
        // A mesh may be non-indexed (a triangle soup), in which case its corners are the positions
        // in order. Welding merges them either way, so the index buffer is only a shortcut.
        let corners: Vec<usize> = match mesh.indices() {
            Some(indices) => indices.iter().collect(),
            None => (0..positions.len()).collect(),
        };

        // Weld vertices that share a position (by exact bits) onto one id, so triangles meeting at a
        // vertex reference the same id and from_faces can actually detect their shared edges.
        let mut verts: Vec<Vec3> = Vec::new();
        let mut vertex_ids: HashMap<[u32; 3], usize> = HashMap::new();
        let mut welded: Vec<usize> = Vec::with_capacity(corners.len());
        for corner in corners {
            let position = positions[corner];
            let key = [position[0].to_bits(), position[1].to_bits(), position[2].to_bits()];
            let id = *vertex_ids.entry(key).or_insert_with(|| {
                verts.push(Vec3::from_array(position));
                verts.len() - 1
            });
            welded.push(id);
        }

        let faces: Vec<Vec<usize>> = welded.chunks(3).map(|tri| tri.to_vec()).collect();
        let grid = Self::from_faces(&faces)?;
        Ok((grid, Mesh3DGridGeometry::new(verts, faces)))
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::prelude::Grid;

    // A cube duplicates its corner vertices per face; welding must merge them or the faces share no
    // ids and every edge reads as a boundary. A closed surface has no boundary edges.
    #[test]
    fn welds_a_cube_into_a_closed_surface() {
        let mesh = Mesh::from(Cuboid::default());
        let num_faces = mesh.indices().unwrap().len() / 3;
        let (grid, _geometry) = GraphGrid::from_mesh(&mesh).unwrap();
        for face in 0..num_faces {
            assert_eq!(grid.slots(face).count(), grid.neighbours(face).count());
        }
    }

    // The icosphere is already welded; from_mesh should keep it a closed surface.
    #[test]
    fn icosphere_is_a_closed_surface() {
        let mesh = Sphere::new(1.0).mesh().ico(0).unwrap();
        let num_faces = mesh.indices().unwrap().len() / 3;
        let (grid, _geometry) = GraphGrid::from_mesh(&mesh).unwrap();
        for face in 0..num_faces {
            assert_eq!(grid.slots(face).count(), grid.neighbours(face).count());
        }
    }
}
