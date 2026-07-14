use glam::Vec3;

use crate::{
    graph::GraphGrid,
    grid::{
        CellOf, CornerOf,
        geometry::{PointQuery, Surface},
    },
    prelude::GridGeometry,
};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct Mesh3DGridGeometry {
    verts: Vec<Vec3>,
    faces: Vec<Vec<usize>>,
}
impl Mesh3DGridGeometry {
    /// Builds mesh geometry from a shared vertex pool and per-face vertex-index lists.
    pub fn new(verts: Vec<Vec3>, faces: Vec<Vec<usize>>) -> Self {
        Self { verts, faces }
    }

    fn face_normal(&self, face: &[usize]) -> Option<Vec3> {
        crate::math::face_normal(face, &self.verts)
    }

    /// Whether an on-plane `point` lies inside `face`, given the face's `normal`.
    fn face_contains(&self, point: Vec3, face: &[usize], normal: Vec3) -> bool {
        face.iter()
            .zip(face.iter().cycle().skip(1))
            // normal is -N of the winding, so an interior point tests <= 0 on every edge.
            .all(|(&a, &b)| (self.verts[b] - self.verts[a]).cross(point - self.verts[a]).dot(normal) <= 1e-4)
    }
}
impl GridGeometry for Mesh3DGridGeometry {
    type Grid = GraphGrid;
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

impl PointQuery for Mesh3DGridGeometry {
    fn cells_at(&self, local: Self::Position) -> impl Iterator<Item = CellOf<Self::Grid>> {
        self.faces.iter().enumerate().filter_map(move |(index, face)| {
            let normal = self.face_normal(face)?;
            if normal.dot(self.verts[face[0]] - local).abs() > 1e-4 {
                return None;
            }
            self.face_contains(local, face, normal).then_some(index)
        })
    }
}

// A closed mesh has no canonical projection, so it marches no ray: pierce is its only ray query.
impl Surface for Mesh3DGridGeometry {
    fn pierce(&self, origin: Vec3, dir: Vec3) -> Option<(f32, Vec3)> {
        let t = self
            .faces
            .iter()
            .filter_map(|face| {
                let normal = self.face_normal(face)?;
                let rate = normal.dot(dir);
                if rate.abs() < 1e-6 {
                    return None;
                }
                let t = normal.dot(self.verts[face[0]] - origin) / rate;
                if t < 0.0 {
                    return None;
                }
                self.face_contains(origin + t * dir, face, normal).then_some(t)
            })
            .min_by(f32::total_cmp)?;
        Some((t, origin + t * dir))
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
        let geometry = Mesh3DGridGeometry::new(verts, vec![vec![0, 1, 2]]);
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
        let geometry = Mesh3DGridGeometry::new(verts, vec![vec![0, 1, 2]]);
        assert_eq!(geometry.try_cell_center(0usize), Some(Vec3::new(1.0, 1.0, 0.0)));
        assert!(geometry.try_cell_center(9usize).is_none());
    }

    #[test]
    fn cells_at_finds_the_containing_face_on_a_non_xy_plane() {
        let verts = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 4.0),
        ];
        let geom = Mesh3DGridGeometry::new(verts, vec![vec![0, 1, 2]]);

        let inside = Vec3::new(1.0, 0.0, 1.0);
        let outside_on_plane = Vec3::new(3.0, 0.0, 3.0);
        let off_plane = Vec3::new(1.0, 1.0, 1.0);
        assert_eq!(geom.cells_at(inside).collect::<Vec<_>>(), vec![0]);
        assert!(geom.cells_at(outside_on_plane).next().is_none());
        assert!(geom.cells_at(off_plane).next().is_none());
    }

    fn xy_triangle() -> Mesh3DGridGeometry {
        let verts = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
        ];
        Mesh3DGridGeometry::new(verts, vec![vec![0, 1, 2]])
    }

    #[test]
    fn pierce_reports_the_hit_t_and_point() {
        let hit = xy_triangle().pierce(Vec3::new(1.0, 1.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(hit, Some((5.0, Vec3::new(1.0, 1.0, 0.0))));
    }

    #[test]
    fn pierce_ignores_the_surface_behind_the_origin() {
        // Cast away from the face: the intersection sits at negative t.
        assert!(
            xy_triangle()
                .pierce(Vec3::new(1.0, 1.0, 5.0), Vec3::new(0.0, 0.0, 1.0))
                .is_none()
        );
    }

    #[test]
    fn pierce_through_the_plane_outside_the_polygon_misses() {
        // Meets the face's plane at (5, 5, 0), outside the triangle.
        assert!(
            xy_triangle()
                .pierce(Vec3::new(5.0, 5.0, 5.0), Vec3::new(0.0, 0.0, -1.0))
                .is_none()
        );
    }

    #[test]
    fn pierce_parallel_to_a_face_misses() {
        // Ray lies in the face's plane: no single touch point.
        assert!(
            xy_triangle()
                .pierce(Vec3::new(1.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0))
                .is_none()
        );
    }
}
