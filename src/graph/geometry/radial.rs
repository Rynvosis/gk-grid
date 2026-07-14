//! Radial surface geometry: a mesh whose vertices all sit on one sphere, stored as unit directions
//! plus a shared radius so equidistance holds by construction. The analytic sphere precedes the
//! mesh: the layer machinery evaluates the field, the mesh only supplies connectivity and corners.
use glam::Vec3;
use hexasphere::shapes::IcoSphere;

use crate::{
    graph::GraphGrid,
    grid::{
        CellOf, CornerOf, Grid,
        geometry::{GridGeometry, Layerable, PointQuery, RayCast, RayHit, RayHitOf, Surface},
    },
};

/// Mesh geometry on a sphere: unit vertex directions around `center`, all at `radius`.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct RadialMeshGeometry {
    center: Vec3,
    radius: f32,
    dirs: Vec<Vec3>,
    faces: Vec<Vec<usize>>,
}

impl RadialMeshGeometry {
    /// Builds radial geometry from unit vertex directions, the faces indexing them, and the sphere they lie on.
    pub fn new(center: Vec3, radius: f32, dirs: Vec<Vec3>, faces: Vec<Vec<usize>>) -> Self {
        Self {
            center,
            radius,
            dirs,
            faces,
        }
    }

    /// Generates an icosphere grid and geometry. `subdivisions` is the number of extra points along
    /// each icosahedron edge, so each of the 20 base faces becomes `(subdivisions + 1)^2` triangles.
    pub fn ico_sphere(center: Vec3, radius: f32, subdivisions: u32) -> (GraphGrid, RadialMeshGeometry) {
        // hexasphere shares the points along each edge between adjacent base faces, so the vertices
        // come out welded and the faces carry the shared indices that give the grid its adjacency.
        let sphere: IcoSphere<Vec3> = IcoSphere::new(subdivisions as usize, Vec3::from);
        let dirs = sphere.raw_data().to_vec();
        let faces: Vec<Vec<usize>> = sphere
            .get_all_indices()
            .chunks(3)
            .map(|triangle| triangle.iter().map(|&index| index as usize).collect())
            .collect();

        let grid = GraphGrid::from_faces(&faces).expect("an icosphere is edge-manifold");
        (grid, RadialMeshGeometry::new(center, radius, dirs, faces))
    }

    /// Whether a unit `direction` from the centre falls inside `face`'s spherical polygon.
    fn face_contains(&self, direction: Vec3, face: &[usize]) -> bool {
        face.iter()
            .zip(face.iter().cycle().skip(1))
            // Each edge spans a plane through the centre, whose normal points inwards for
            // counter-clockwise winding, so an interior direction tests positive on every one. The
            // slack is negative so a direction on a shared edge is kept by both its faces instead of
            // dropped from both.
            .all(|(&a, &b)| self.dirs[a].cross(self.dirs[b]).dot(direction) >= -1e-4)
    }

    /// Ray parameter where the ray's central projection leaves `face` through edge `slot`, or `None`
    /// if it never crosses that edge outwards.
    ///
    /// A ray straight through the centre has no projection there, so its great circle degenerates.
    fn edge_exit(&self, origin: Vec3, dir: Vec3, face: &[usize], slot: usize) -> Option<f32> {
        let (a, b) = (face[slot], face[(slot + 1) % face.len()]);
        // The edge spans a plane through the centre, so the side test is affine in `t` and its root
        // is the crossing. The rate is the plane test applied to `dir`: negative means the ray is on
        // its way out through this edge, and anything else is an entry or a parallel graze.
        let edge_normal = self.dirs[a].cross(self.dirs[b]);
        let rate = dir.dot(edge_normal);
        if rate > -1e-6 {
            return None;
        }
        Some((self.center - origin).dot(edge_normal) / rate)
    }

    /// The part of the ray-sphere solve every shell about the centre shares, or `None` if `dir` is
    /// degenerate.
    fn shell_setup(&self, origin: Vec3, dir: Vec3) -> Option<ShellSetup> {
        let center_to_origin = origin - self.center;
        let dir_sq = dir.length_squared();
        if dir_sq < 1e-6 {
            return None;
        }
        // `dir` is not normalized (raycast `t` is a world distance), so `dir_sq` stays in.
        let origin_along_dir = center_to_origin.dot(dir);
        Some(ShellSetup {
            dir_sq,
            origin_along_dir,
            closest_sq: center_to_origin.length_squared() - origin_along_dir * origin_along_dir / dir_sq,
        })
    }
}

/// The ray-sphere solve with the shell radius factored out, so a foliation solves each shell without
/// redoing the shared work.
#[derive(Copy, Clone, Debug)]
struct ShellSetup {
    dir_sq: f32,
    origin_along_dir: f32,
    closest_sq: f32,
}

impl ShellSetup {
    /// Ray parameters where the ray enters and leaves the sphere of `shell_radius`, or `None` if it
    /// misses.
    fn roots(&self, shell_radius: f32) -> Option<(f32, f32)> {
        // The closest approach is the same for every shell, which turns the discriminant into
        // dir_sq * (shell_radius^2 - closest_sq), so it doubles as a "does the ray reach it" test.
        let discriminant = self.dir_sq * (shell_radius * shell_radius - self.closest_sq);
        (discriminant >= 0.0).then(|| {
            let root = discriminant.sqrt();
            (
                (-self.origin_along_dir - root) / self.dir_sq,
                (-self.origin_along_dir + root) / self.dir_sq,
            )
        })
    }
}

impl GridGeometry for RadialMeshGeometry {
    type Grid = GraphGrid;
    type Position = Vec3;

    fn try_cell_center(&self, cell: impl Into<CellOf<Self::Grid>>) -> Option<Self::Position> {
        self.try_cell_corners(cell).and_then(|iter| {
            let (sum, n) = iter.fold((Vec3::ZERO, 0), |(sum, n), (_, vertex)| (sum + vertex, n + 1));
            (n > 0).then(|| self.center + self.radius * (sum / n as f32 - self.center).normalize())
        })
    }

    fn try_cell_corners(
        &self,
        cell: impl Into<CellOf<Self::Grid>>,
    ) -> Option<impl Iterator<Item = (CornerOf<Self::Grid>, Self::Position)>> {
        self.faces.get(cell.into()).map(|face| {
            face.iter()
                .enumerate()
                .map(|(corner, &dir)| (corner, self.center + self.dirs[dir] * self.radius))
        })
    }
}

impl Surface for RadialMeshGeometry {
    fn pierce(&self, origin: Vec3, dir: Vec3) -> Option<(f32, Vec3)> {
        // The near root is behind an origin already inside the sphere, so the first touch is
        // whichever root the ray reaches first without going backwards.
        let (near, far) = self.shell_setup(origin, dir)?.roots(self.radius)?;
        let t = if near >= 0.0 { near } else { far };
        (t >= 0.0).then(|| (t, origin + t * dir))
    }
}

impl RayCast for RadialMeshGeometry {
    /// Marches the ray's central projection: the great circle it sweeps across the sphere. The
    /// projection leaves a face through exactly one edge, and the neighbour across that edge is the
    /// next cell, so the march is a walk over the adjacency with the exit edge naming each step.
    fn raycast(
        &self,
        grid: &Self::Grid,
        origin: Self::Position,
        dir: Self::Position,
    ) -> impl Iterator<Item = RayHitOf<Self::Grid>> {
        // Deliberately one seed: a point on an edge names both its faces, but the march has to start
        // somewhere and the crossing that follows re-enters the other one anyway.
        let seed = self.cells_at(origin).next().map(|cell| RayHit {
            cell,
            t: 0.0,
            face: None,
        });

        std::iter::successors(seed, move |hit| {
            // The next cell is whichever edge the projection leaves through first. `edge_exit`
            // reports only the edges it actually leaves through, so the earliest is the exit.
            let face = &self.faces[hit.cell];
            let (slot, t) = (0..face.len())
                .filter_map(|slot| self.edge_exit(origin, dir, face, slot).map(|t| (slot, t)))
                // An exit at or before the current `t` is the edge just entered through, read back
                // from the far side. Only a strictly later one is a step forwards.
                .filter(|&(_, t)| t > hit.t)
                .min_by(|(_, a), (_, b)| a.total_cmp(b))?;

            let connection = grid.try_connection(hit.cell, slot)?;
            Some(RayHit {
                cell: connection.cell,
                t,
                // The slot the ray entered *by*: the same edge, as the new face numbers it.
                face: Some(connection.back),
            })
        })
    }
}

impl Layerable for RadialMeshGeometry {
    fn lift(&self, point: Vec3, offset: f32) -> Vec3 {
        point + (point - self.center).normalize() * offset
    }

    fn height(&self, point: Vec3) -> f32 {
        (point - self.center).length() - self.radius
    }

    fn layer_crossings(&self, origin: Vec3, dir: Vec3, spacing: f32) -> impl Iterator<Item = (f32, i32)> {
        let center_to_origin = origin - self.center;
        // A degenerate ray has no setup, so `roots` yields nothing and the march ends on its own.
        let setup = self.shell_setup(origin, dir);

        // Shell n sits at radius + n * spacing, so its crossings are the roots of the ray-sphere
        // quadratic. The foliation stops at the centre: below it the shell radius goes negative, and
        // its square would happily keep reporting crossings, walking `t` back on itself.
        let roots = move |shell: i32| {
            let shell_radius = self.radius + shell as f32 * spacing;
            (shell_radius > 0.0).then_some(())?;
            setup?.roots(shell_radius)
        };

        // Bands are half-open, so the boundary at or below the origin is the first one a descending
        // ray crosses, as on the planar base. Height falls to the closest approach, crossing shells
        // inward (step -1), then rises forever, crossing them back outward (step +1), so walking the
        // boundaries down and then up visits them in nondecreasing `t` without any sorting.
        let origin_shell = ((center_to_origin.length() - self.radius) / spacing).floor() as i32;
        // The ray descends toward the centre exactly when the origin is on the near side of the
        // closest approach, which is when it points back toward the centre.
        let mut descending = center_to_origin.dot(dir) < 0.0;
        // A receding ray never descends, so it starts one boundary up: the one it sits on is the
        // floor of the layer it is already in, and half-open bands do not re-cross it.
        let mut shell = if descending { origin_shell } else { origin_shell + 1 };

        std::iter::from_fn(move || {
            loop {
                if descending {
                    match roots(shell) {
                        Some((inward, _)) if inward >= 0.0 => {
                            shell -= 1;
                            return Some((inward, -1));
                        }
                        // Turned around: climb back out from the last boundary the descent left
                        // behind, which is the one above the shell it failed to reach.
                        _ => {
                            shell += 1;
                            descending = false;
                        }
                    }
                } else {
                    let (_, outward) = roots(shell)?;
                    shell += 1;
                    return Some((outward, 1));
                }
            }
        })
    }
}
impl PointQuery for RadialMeshGeometry {
    fn cells_at(&self, local: Self::Position) -> impl Iterator<Item = CellOf<Self::Grid>> {
        // Central projection: only the direction from the centre matters, so a point anywhere off the
        // surface still names a face. The centre itself names none.
        let direction = (local - self.center).normalize_or_zero();
        let faces = if direction == Vec3::ZERO { &[][..] } else { &self.faces };
        faces
            .iter()
            .enumerate()
            .filter_map(move |(index, face)| self.face_contains(direction, face).then_some(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::Grid;

    // Shells sit at radius + k * spacing = 1, 2, 3, ... `layer_crossings` only reads the centre and
    // the radius, so the mesh itself can be empty.
    fn unit_sphere() -> RadialMeshGeometry {
        RadialMeshGeometry::new(Vec3::ZERO, 1.0, vec![], vec![])
    }

    // hexasphere shares the points along each edge, so its indices are welded and the icosphere comes
    // out a closed surface: every face finds a neighbour across every edge. If it duplicated vertices
    // instead, the faces would share no indices and every edge would read as a boundary.
    #[test]
    fn ico_sphere_builds_a_closed_surface() {
        // One subdivision: (1 + 1)^2 triangles on each of the 20 base faces.
        let (grid, _geometry) = RadialMeshGeometry::ico_sphere(Vec3::ZERO, 1.0, 1);
        for face in 0..80usize {
            assert_eq!(grid.slots(face).count(), grid.neighbours(face).count());
        }
    }

    // `dirs` are directions from the centre, not positions, so an off-origin sphere must carry its
    // centre into every corner it reports.
    #[test]
    fn cell_corners_sit_on_the_sphere_around_an_off_origin_centre() {
        let center = Vec3::new(10.0, -3.0, 2.0);
        let (_grid, geometry) = RadialMeshGeometry::ico_sphere(center, 4.0, 1);
        for (_, corner) in geometry.try_cell_corners(0usize).unwrap() {
            assert!(
                (corner.distance(center) - 4.0).abs() < 1e-4,
                "corner off the sphere: {corner}"
            );
        }
    }

    // A face's own centre must name that face, which fails outright if the winding sign in
    // `face_contains` is inverted: every face would reject every direction.
    #[test]
    fn cells_at_finds_the_face_beneath_its_own_centre() {
        let center = Vec3::new(10.0, -3.0, 2.0);
        let (_grid, geometry) = RadialMeshGeometry::ico_sphere(center, 4.0, 1);
        for face in 0..80usize {
            let cell_center = geometry.try_cell_center(face).unwrap();
            assert!(
                geometry.cells_at(cell_center).any(|found| found == face),
                "face {face} does not contain its own centre"
            );
        }
    }

    // A direction on a shared edge belongs to both faces, so the slack must let it in rather than
    // biasing inwards and dropping it from each.
    #[test]
    fn cells_at_on_a_shared_edge_reports_both_faces() {
        let (_grid, geometry) = RadialMeshGeometry::ico_sphere(Vec3::ZERO, 1.0, 1);
        let corners: Vec<Vec3> = geometry.try_cell_corners(0usize).unwrap().map(|(_, c)| c).collect();
        let edge_midpoint = (corners[0] + corners[1]) / 2.0;
        assert_eq!(geometry.cells_at(edge_midpoint).count(), 2);
    }

    // A ray that grazes the sphere, aimed so its projection sweeps a long arc across many faces
    // rather than clipping a corner. Origin sits off to one side, pointing across.
    fn grazing_march(grid: &GraphGrid, geometry: &RadialMeshGeometry) -> Vec<RayHitOf<GraphGrid>> {
        geometry
            .raycast(grid, Vec3::new(-3.0, 0.4, 0.2), Vec3::new(1.0, 0.0, 0.0))
            .take(12)
            .collect()
    }

    #[test]
    fn raycast_starts_in_the_cell_under_the_origin() {
        let (grid, geometry) = RadialMeshGeometry::ico_sphere(Vec3::ZERO, 1.0, 1);
        let origin = Vec3::new(-3.0, 0.4, 0.2);
        let first = geometry
            .raycast(&grid, origin, Vec3::X)
            .next()
            .expect("the march starts");

        assert_eq!(first.t, 0.0);
        assert_eq!(first.face, None, "nothing was crossed to reach the first cell");
        assert!(
            geometry.cells_at(origin).any(|cell| cell == first.cell),
            "the first cell must be one the origin projects into"
        );
    }

    #[test]
    fn raycast_visits_cells_in_nondecreasing_t() {
        let (grid, geometry) = RadialMeshGeometry::ico_sphere(Vec3::ZERO, 1.0, 1);
        let hits = grazing_march(&grid, &geometry);
        assert!(hits.len() > 1, "a grazing ray sweeps more than one face");
        for pair in hits.windows(2) {
            assert!(pair[1].t >= pair[0].t, "t must be nondecreasing: {pair:?}");
        }
    }

    // The march's own consistency check: stepping across the slot a hit reports must land back on the
    // cell it came from. If the entry slot is renumbered wrongly, a consumer walking the hits back
    // through the grid ends up somewhere else.
    #[test]
    fn raycast_each_hit_is_adjacent_across_the_slot_it_reports() {
        let (grid, geometry) = RadialMeshGeometry::ico_sphere(Vec3::ZERO, 1.0, 1);
        let hits = grazing_march(&grid, &geometry);
        for pair in hits.windows(2) {
            let (previous, entered) = (pair[0], pair[1]);
            let slot = entered
                .face
                .expect("every cell after the first was entered through a slot");
            assert_eq!(
                grid.try_neighbour(entered.cell, slot),
                Some(previous.cell),
                "the entry slot must lead back to the cell the ray came from"
            );
        }
    }

    // Ties the march back to the point query: the hit is where the ray crosses into the cell, so the
    // point just past it must project into the cell the march names.
    #[test]
    fn raycast_hit_points_project_into_the_cells_they_name() {
        let (grid, geometry) = RadialMeshGeometry::ico_sphere(Vec3::ZERO, 1.0, 1);
        let hits = grazing_march(&grid, &geometry);
        for pair in hits.windows(2) {
            // Midway between this crossing and the next, so the point is clear of both edges.
            let midpoint_t = (pair[0].t + pair[1].t) / 2.0;
            let point = Vec3::new(-3.0, 0.4, 0.2) + midpoint_t * Vec3::X;
            assert!(
                geometry.cells_at(point).any(|cell| cell == pair[0].cell),
                "cell {:?} does not contain the ray between its entry and its exit",
                pair[0].cell
            );
        }
    }

    #[test]
    fn layer_crossings_report_each_shell_a_receding_ray_leaves() {
        let crossings: Vec<_> = unit_sphere()
            .layer_crossings(Vec3::new(0.0, 0.0, 2.0), Vec3::new(0.0, 0.0, 1.0), 1.0)
            .take(3)
            .collect();
        assert_eq!(crossings, vec![(1.0, 1), (2.0, 1), (3.0, 1)]);
    }

    // `dir` arrives unnormalized so that raycast `t` reads as a world distance, which is why the
    // quadratic keeps its `a` term: doubling `dir` must halve every `t`.
    #[test]
    fn layer_crossings_scale_with_an_unnormalized_dir() {
        let crossings: Vec<_> = unit_sphere()
            .layer_crossings(Vec3::new(0.0, 0.0, 2.0), Vec3::new(0.0, 0.0, 2.0), 1.0)
            .take(3)
            .collect();
        assert_eq!(crossings, vec![(0.5, 1), (1.0, 1), (1.5, 1)]);
    }

    // Straight through the core: the ray falls through the shells at radius 3, 2 and 1, passes the
    // centre at t = 3.5 with no shell beneath it, then climbs back out through the same three.
    #[test]
    fn layer_crossings_through_the_core_descend_then_ascend() {
        let crossings: Vec<_> = unit_sphere()
            .layer_crossings(Vec3::new(0.0, 0.0, 3.5), Vec3::new(0.0, 0.0, -1.0), 1.0)
            .take(6)
            .collect();
        assert_eq!(
            crossings,
            vec![(0.5, -1), (1.5, -1), (2.5, -1), (4.5, 1), (5.5, 1), (6.5, 1)]
        );
    }

    // Closest approach is 3.5, so the shells at radius 3 and inward are never reached: the ray
    // descends exactly three of them, turns, and climbs back out without `t` ever going backwards.
    #[test]
    fn layer_crossings_grazing_ray_never_reaches_the_inner_shells() {
        let crossings: Vec<_> = unit_sphere()
            .layer_crossings(Vec3::new(0.0, 3.5, 5.0), Vec3::new(0.0, 0.0, -1.0), 1.0)
            .take(8)
            .collect();

        let descents = crossings.iter().filter(|&&(_, step)| step == -1).count();
        assert_eq!(descents, 3, "only the shells outside the closest approach are crossed");
        assert!(
            crossings.iter().take(descents).all(|&(_, step)| step == -1),
            "every descent precedes every climb"
        );
        for pair in crossings.windows(2) {
            assert!(pair[1].0 >= pair[0].0, "t must be nondecreasing: {pair:?}");
        }
    }
}
