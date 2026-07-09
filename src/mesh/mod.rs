#[cfg(feature = "bevy")]
mod from_mesh;
pub(crate) mod geometry;

use crate::mesh::geometry::MeshGridGeometry;
use crate::prelude::Grid;
use crate::region::Region;
use glam::Vec3;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct MeshGrid {
    adjacency: Vec<Vec<Option<usize>>>,
}

impl MeshGrid {
    /// Builds a mesh grid and its geometry from per-face vertex-index lists and vertex positions.
    /// Panics if mesh is not manifold.
    pub fn from_faces(faces: Vec<Vec<usize>>, verts: Vec<Vec3>) -> (MeshGrid, MeshGridGeometry) {
        let adjacency = Self::adjacency_from_faces(&faces);
        (MeshGrid { adjacency }, MeshGridGeometry::new(verts, faces))
    }

    /// Derives face-to-face adjacency by matching edges shared between faces.
    fn adjacency_from_faces(faces: &[Vec<usize>]) -> Vec<Vec<Option<usize>>> {
        let mut adjacency: Vec<Vec<Option<usize>>> =
            faces.iter().map(|f| vec![None; f.len()]).collect();
        let mut edge_map: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut closed_edges: HashSet<(usize, usize)> = HashSet::new();

        for (face_idx, face) in faces.iter().enumerate() {
            for (i, (&a, &b)) in face.iter().zip(face.iter().cycle().skip(1)).enumerate() {
                let edge = (a.min(b), a.max(b));
                match edge_map.entry(edge) {
                    Entry::Vacant(e) => {
                        if closed_edges.contains(&edge) {
                            panic!(
                                "Non-manifold mesh: 3rd vertex pair detected for edge {:?}",
                                edge
                            );
                        }
                        e.insert((face_idx, i));
                    }
                    Entry::Occupied(e) => {
                        let (other_face, other_i) = *e.get();
                        e.remove();
                        closed_edges.insert(edge);
                        adjacency[face_idx][i] = Some(other_face);
                        adjacency[other_face][other_i] = Some(face_idx);
                    }
                }
            }
        }

        adjacency
    }

    /// A region covering every face of this mesh.
    pub fn faces_region(&self) -> FaceRegion {
        FaceRegion::new(self.adjacency.len())
    }
}

impl Grid for MeshGrid {
    type Cell = usize;
    type Corner = usize;
    type Slot = usize;

    fn slots(&self, cell: impl Into<Self::Cell>) -> impl Iterator<Item = Self::Slot> {
        self.adjacency
            .get(cell.into())
            .map(|adj| 0..adj.len())
            .into_iter()
            .flatten()
    }

    fn try_neighbour(
        &self,
        cell: impl Into<Self::Cell>,
        direction: impl Into<Self::Slot>,
    ) -> Option<Self::Cell> {
        *self.adjacency.get(cell.into())?.get(direction.into())?
    }

    fn neighbours(
        &self,
        cell: impl Into<Self::Cell>,
    ) -> impl Iterator<Item = (Self::Slot, Self::Cell)> {
        self.adjacency
            .get(cell.into())
            .map(|adj| {
                adj.iter()
                    .enumerate()
                    .filter_map(|(slot, neighbour)| neighbour.map(|neighbour| (slot, neighbour)))
            })
            .into_iter()
            .flatten()
    }
}

/// A region over a mesh's faces, addressed by face id from zero.
#[derive(Clone, Copy, Debug)]
pub struct FaceRegion {
    len: usize,
}

impl FaceRegion {
    /// A region covering face ids `0..len`.
    pub fn new(len: usize) -> Self {
        Self { len }
    }
}

impl Region for FaceRegion {
    type Cell = usize;

    fn iter(&self) -> impl Iterator<Item = usize> {
        0..self.len
    }

    fn contains(&self, cell: usize) -> bool {
        cell < self.len
    }

    fn index_of(&self, cell: usize) -> Option<usize> {
        (cell < self.len).then_some(cell)
    }

    fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A triangle strip 0-1-2: face 0 shares an edge with 1, face 1 with 2; 0 and 2 are not adjacent.
    fn strip() -> MeshGrid {
        MeshGrid {
            adjacency: vec![
                vec![Some(1), None, None],
                vec![Some(0), Some(2), None],
                vec![Some(1), None, None],
            ],
        }
    }

    #[test]
    fn slots_enumerate_every_edge_including_boundaries() {
        let grid = strip();
        assert_eq!(grid.slots(0usize).collect::<Vec<_>>(), vec![0, 1, 2]);
        assert_eq!(grid.slots(1usize).count(), 3);
        assert_eq!(grid.slots(99usize).count(), 0);
    }

    #[test]
    fn try_neighbour_distinguishes_missing_face_bad_slot_and_boundary() {
        let grid = strip();
        assert_eq!(grid.try_neighbour(1usize, 0usize), Some(0));
        assert_eq!(grid.try_neighbour(1usize, 1usize), Some(2));
        assert_eq!(grid.try_neighbour(1usize, 2usize), None); // boundary edge
        assert_eq!(grid.try_neighbour(1usize, 7usize), None); // slot out of range
        assert_eq!(grid.try_neighbour(99usize, 0usize), None); // face out of range
    }

    #[test]
    fn neighbours_yield_only_present_edges_in_slot_order() {
        let grid = strip();
        assert_eq!(
            grid.neighbours(1usize).collect::<Vec<_>>(),
            vec![(0, 0), (1, 2)]
        );
        assert_eq!(grid.neighbours(0usize).collect::<Vec<_>>(), vec![(0, 1)]);
    }

    // Two triangles sharing edge {1,2}; every other edge is a boundary.
    #[test]
    fn from_faces_links_faces_that_share_an_edge() {
        let verts = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
        ];
        let faces = vec![vec![0, 1, 2], vec![2, 1, 3]];
        let (grid, _geometry) = MeshGrid::from_faces(faces, verts);

        assert_eq!(grid.slots(0usize).count(), 3);
        assert_eq!(grid.slots(1usize).count(), 3);
        assert!(grid.neighbours(0usize).any(|(_, c)| c == 1));
        assert!(grid.neighbours(1usize).any(|(_, c)| c == 0));
        assert_eq!(grid.neighbours(0usize).count(), 1);
        assert_eq!(grid.neighbours(1usize).count(), 1);
    }

    #[test]
    fn face_region_indexes_faces_by_identity() {
        let region = FaceRegion::new(5);
        assert_eq!(region.iter().collect::<Vec<_>>(), vec![0, 1, 2, 3, 4]);
        assert_eq!(region.len(), 5);
        assert_eq!(region.index_of(3), Some(3));
        assert_eq!(region.index_of(5), None);
        assert!(region.contains(4));
        assert!(!region.contains(5));
    }

    #[test]
    fn faces_region_covers_all_faces() {
        assert_eq!(strip().faces_region().len(), 3);
    }
}
