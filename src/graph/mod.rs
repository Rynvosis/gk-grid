#[cfg(feature = "bevy")]
mod from_mesh;
pub(crate) mod geometry;

use std::collections::{HashMap, HashSet, VecDeque, hash_map::Entry};

use glam::Vec3;

use crate::{graph::geometry::Mesh3DGridGeometry, prelude::Grid, region::Region};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct GraphGrid {
    adjacency: Vec<Vec<Option<usize>>>,
}

/// The mesh is not edge-manifold: an edge is shared by three or more faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonManifoldError {
    pub edge: (usize, usize),
}
impl std::fmt::Display for NonManifoldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the given graph grid contains a non-manifold region (edge {:?} is shared by 3+ faces)",
            self.edge
        )
    }
}
impl std::error::Error for NonManifoldError {}

impl GraphGrid {
    /// Builds a mesh grid and its geometry from per-face vertex-index lists and vertex positions.
    /// Returns [`NonManifoldError`] if an edge is shared by three or more faces.
    pub fn from_faces(
        faces: Vec<Vec<usize>>,
        verts: Vec<Vec3>,
    ) -> Result<(GraphGrid, Mesh3DGridGeometry), NonManifoldError> {
        let adjacency = Self::adjacency_from_faces(&faces)?;
        Ok((GraphGrid { adjacency }, Mesh3DGridGeometry::new(verts, faces)))
    }

    /// Derives face-to-face adjacency by matching edges shared between faces.
    fn adjacency_from_faces(faces: &[Vec<usize>]) -> Result<Vec<Vec<Option<usize>>>, NonManifoldError> {
        let mut adjacency: Vec<Vec<Option<usize>>> = faces.iter().map(|f| vec![None; f.len()]).collect();
        let mut edge_map: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut closed_edges: HashSet<(usize, usize)> = HashSet::new();

        for (face_idx, face) in faces.iter().enumerate() {
            for (i, (&a, &b)) in face.iter().zip(face.iter().cycle().skip(1)).enumerate() {
                let edge = (a.min(b), a.max(b));
                match edge_map.entry(edge) {
                    Entry::Vacant(e) => {
                        if closed_edges.contains(&edge) {
                            return Err(NonManifoldError { edge });
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

        Ok(adjacency)
    }

    /// A region covering every face of this mesh.
    pub fn faces_region(&self) -> FaceRegion {
        FaceRegion::new(self.adjacency.len())
    }
}

/// Merges adjacent triangles whose normals agree within `max_angle` radians into n-gon faces, passing
/// non-mergeable faces through unchanged.
/// Returns [`NonManifoldError`] if an edge is shared by three or more faces.
pub fn merge_coplanar(
    mut faces: Vec<Vec<usize>>,
    verts: &[Vec3],
    max_angle: f32,
) -> Result<Vec<Vec<usize>>, NonManifoldError> {
    let adjacency = GraphGrid::adjacency_from_faces(&faces)?;

    // None = degenerate or non-triangle; neither becomes a merge candidate.
    let normals: Vec<Option<Vec3>> = faces
        .iter()
        .map(|f| (f.len() == 3).then(|| crate::math::face_normal(f, verts)).flatten())
        .collect();
    let min_dot = max_angle.cos() - 1e-6;

    // BFS-group faces whose normals agree with the seed's within max_angle, recording each face's
    // group so boundaries can be read straight off the adjacency below.
    let mut face_group: Vec<Option<usize>> = vec![None; faces.len()];
    let mut groups: Vec<Vec<usize>> = Vec::new();

    for i in 0..faces.len() {
        if face_group[i].is_some() {
            continue;
        }
        let Some(normal_i) = normals[i] else { continue };

        let g = groups.len();
        face_group[i] = Some(g);
        let mut group = vec![i];
        let mut queue = VecDeque::from([i]);
        while let Some(current) = queue.pop_front() {
            for &neighbor in adjacency[current].iter().flatten() {
                if face_group[neighbor].is_some() {
                    continue;
                }
                let Some(normal_n) = normals[neighbor] else { continue };
                if normal_i.dot(normal_n) >= min_dot {
                    face_group[neighbor] = Some(g);
                    group.push(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
        groups.push(group);
    }

    // Read each group's boundary edges off the adjacency (an edge is a group boundary when the face
    // across it isn't in the same group) and walk them into one loop. If that isn't a single simple
    // loop, pass the group's faces through unmerged; policing mesh correctness isn't our job.
    let mut merged_faces = Vec::new();
    let mut boundary_edges: Vec<(usize, usize)> = Vec::new();

    for (g, group) in groups.iter().enumerate() {
        boundary_edges.clear();
        for &f in group {
            let face = &faces[f];
            for (i, (&a, &b)) in face.iter().zip(face.iter().cycle().skip(1)).enumerate() {
                let interior = adjacency[f][i].is_some_and(|nbr| face_group[nbr] == Some(g));
                if !interior {
                    boundary_edges.push((a, b));
                }
            }
        }

        match boundary_loop(&boundary_edges) {
            Some(face) => merged_faces.push(face),
            None => merged_faces.extend(group.iter().map(|&f| std::mem::take(&mut faces[f]))),
        }
    }

    // Faces that never became merge candidates (non-triangles / degenerate triangles) pass through
    // unchanged: this merges the residual triangles and leaves everything else as it came in.
    for (face, normal) in faces.iter_mut().zip(&normals) {
        if normal.is_none() {
            merged_faces.push(std::mem::take(face));
        }
    }

    Ok(merged_faces)
}

/// Walks directed boundary edges into one vertex loop, or `None` if they aren't a single simple loop.
fn boundary_loop(edges: &[(usize, usize)]) -> Option<Vec<usize>> {
    if edges.is_empty() {
        return None;
    }
    // Each boundary vertex has exactly one outgoing edge; a repeat means the boundary pinches.
    let mut next: HashMap<usize, usize> = HashMap::with_capacity(edges.len());
    for &(a, b) in edges {
        if next.insert(a, b).is_some() {
            return None;
        }
    }
    let start = edges[0].0;
    let mut loop_verts = vec![start];
    let mut current = next[&start];
    while current != start {
        loop_verts.push(current);
        current = *next.get(&current)?;
    }
    // A hole leaves edges unwalked; a face needs at least three vertices.
    (loop_verts.len() == edges.len() && loop_verts.len() >= 3).then_some(loop_verts)
}

impl Grid for GraphGrid {
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

    fn try_neighbour(&self, cell: impl Into<Self::Cell>, direction: impl Into<Self::Slot>) -> Option<Self::Cell> {
        *self.adjacency.get(cell.into())?.get(direction.into())?
    }

    fn neighbours(&self, cell: impl Into<Self::Cell>) -> impl Iterator<Item = (Self::Slot, Self::Cell)> {
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
    fn strip() -> GraphGrid {
        GraphGrid {
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
        assert_eq!(grid.neighbours(1usize).collect::<Vec<_>>(), vec![(0, 0), (1, 2)]);
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
        let (grid, _geometry) = GraphGrid::from_faces(faces, verts).unwrap();

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

    // Two coplanar triangles tiling a unit square merge into one quad, in boundary order.
    #[test]
    fn merge_coplanar_fuses_two_triangles_into_a_quad() {
        let verts = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];
        assert_eq!(merge_coplanar(faces, &verts, 0.01).unwrap(), vec![vec![0, 1, 2, 3]]);
    }

    // A fold across the shared edge keeps both triangles: their normals disagree past max_angle.
    #[test]
    fn merge_coplanar_keeps_a_fold_as_two_triangles() {
        let verts = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 1.0),
        ];
        let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];
        assert_eq!(
            merge_coplanar(faces, &verts, 0.01).unwrap(),
            vec![vec![0, 1, 2], vec![0, 2, 3]]
        );
    }

    // A lone triangle passes through unchanged.
    #[test]
    fn merge_coplanar_passes_a_lone_triangle_through() {
        let verts = vec![Vec3::ZERO, Vec3::X, Vec3::Y];
        assert_eq!(
            merge_coplanar(vec![vec![0, 1, 2]], &verts, 0.01).unwrap(),
            vec![vec![0, 1, 2]]
        );
    }

    // A non-triangle face is never a merge candidate and passes straight through.
    #[test]
    fn merge_coplanar_passes_a_non_triangle_through() {
        let verts = vec![Vec3::ZERO, Vec3::X, Vec3::new(1.0, 1.0, 0.0), Vec3::Y];
        assert_eq!(
            merge_coplanar(vec![vec![0, 1, 2, 3]], &verts, 0.01).unwrap(),
            vec![vec![0, 1, 2, 3]]
        );
    }

    // A unit cube as 12 triangles (two per face sharing a diagonal): each coplanar pair fuses back into
    // its quad while the perpendicular faces stay separate, so a closed triangle cube becomes six quads.
    #[test]
    fn merge_coplanar_fuses_a_cube_into_six_quads() {
        let verts = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(0.0, 1.0, 1.0),
        ];
        let faces = vec![
            vec![0, 1, 2], vec![0, 2, 3], // z = 0
            vec![4, 5, 6], vec![4, 6, 7], // z = 1
            vec![0, 1, 5], vec![0, 5, 4], // y = 0
            vec![3, 2, 6], vec![3, 6, 7], // y = 1
            vec![0, 3, 7], vec![0, 7, 4], // x = 0
            vec![1, 2, 6], vec![1, 6, 5], // x = 1
        ];
        assert_eq!(
            merge_coplanar(faces, &verts, 0.01).unwrap(),
            vec![
                vec![0, 1, 2, 3],
                vec![4, 5, 6, 7],
                vec![0, 1, 5, 4],
                vec![3, 2, 6, 7],
                vec![0, 3, 7, 4],
                vec![1, 2, 6, 5],
            ]
        );
    }

    #[test]
    fn boundary_loop_walks_a_simple_square() {
        assert_eq!(boundary_loop(&[(0, 1), (1, 2), (2, 3), (3, 0)]), Some(vec![0, 1, 2, 3]));
    }

    // Two triangles meeting only at vertex 0: vertex 0 has two outgoing boundary edges (a bowtie).
    #[test]
    fn boundary_loop_rejects_a_pinch() {
        assert_eq!(boundary_loop(&[(0, 1), (1, 2), (2, 0), (0, 3), (3, 4), (4, 0)]), None);
    }

    // Two disjoint loops (a hole): walking one leaves the other's edges unwalked.
    #[test]
    fn boundary_loop_rejects_a_hole() {
        assert_eq!(boundary_loop(&[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3)]), None);
    }

    #[test]
    fn boundary_loop_rejects_empty() {
        assert_eq!(boundary_loop(&[]), None);
    }
}
