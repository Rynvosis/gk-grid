use glam::Vec3;

use crate::{
    grid::{CellOf, CornerOf},
    mesh::MeshGrid,
    prelude::GridGeometry,
};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct MeshGridGeometry {
    verts: Vec<Vec3>,
    faces: Vec<Vec<usize>>,
}
impl MeshGridGeometry {
    /// Builds mesh geometry from a shared vertex pool and per-face vertex-index lists.
    pub fn new(verts: Vec<Vec3>, faces: Vec<Vec<usize>>) -> Self {
        Self { verts, faces }
    }
}
impl GridGeometry for MeshGridGeometry {
    type Grid = MeshGrid;
    type Position = Vec3;

    fn try_cell_center(&self, cell: impl Into<CellOf<Self::Grid>>) -> Option<Self::Position> {
        self.try_cell_corners(cell).and_then(|iter| {
            let (sum, n) = iter.fold((Vec3::ZERO, 0), |(sum, n), (_, vertex)| (sum + vertex, n + 1));
            (n > 0).then(|| sum / n as f32)
        })
    }

    fn try_cell_corners(
        &self,
        cell: impl Into<CellOf<Self::Grid>>,
    ) -> Option<impl Iterator<Item = (CornerOf<Self::Grid>, Self::Position)>> {
        self.faces.get(cell.into()).map(|face| {
            face.iter()
                .enumerate()
                .map(|(corner, &vertex)| (corner, self.verts[vertex]))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_corners_yield_face_vertices_in_winding_order() {
        let verts = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
        ];
        let geometry = MeshGridGeometry::new(verts, vec![vec![0, 1, 2]]);
        assert_eq!(
            geometry.try_cell_corners(0usize).unwrap().collect::<Vec<_>>(),
            vec![
                (0, Vec3::new(0.0, 0.0, 0.0)),
                (1, Vec3::new(2.0, 0.0, 0.0)),
                (2, Vec3::new(0.0, 2.0, 0.0)),
            ]
        );
        assert!(geometry.try_cell_corners(9usize).is_none());
    }

    #[test]
    fn cell_center_is_the_corner_centroid() {
        let verts = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0, 3.0, 0.0),
        ];
        let geometry = MeshGridGeometry::new(verts, vec![vec![0, 1, 2]]);
        assert_eq!(geometry.try_cell_center(0usize), Some(Vec3::new(1.0, 1.0, 0.0)));
        assert!(geometry.try_cell_center(9usize).is_none());
    }
}
