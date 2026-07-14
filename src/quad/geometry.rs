use glam::{Mat2, Vec2, Vec3, Vec3Swizzles};

use crate::{
    grid::{
        CellOf, CornerOf,
        geometry::{
            GridGeometry, Layerable, PointQuery, RayCast, RayHit, RayHitOf, Surface, TotalGridGeometry, TotalPointQuery,
        },
        swizzle::GridSwizzle,
    },
    quad::{ALL_QUAD_CORNERS, QuadDir, QuadGrid},
};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct QuadGridGeometry {
    cell_size: Vec2,
    projection: Mat2,
    swizzle: GridSwizzle,
}
impl QuadGridGeometry {
    pub fn projection(&self) -> Mat2 {
        self.projection
    }
    pub fn cell_size(&self) -> Vec2 {
        self.cell_size
    }

    /// Reorders which local axes this grid's XY plane maps onto.
    /// Defaults to `GridSwizzle::Xyz` (z = 0).
    pub fn with_swizzle(mut self, swizzle: GridSwizzle) -> Self {
        self.swizzle = swizzle;
        self
    }

    fn from_projection(cell_size: Vec2, projection: Mat2) -> Self {
        Self {
            cell_size,
            projection,
            swizzle: GridSwizzle::default(),
        }
    }

    // constructors
    pub fn rect(cell_size: Vec2) -> Self {
        Self::from_projection(cell_size, Mat2::from_diagonal(cell_size))
    }

    /// dimetric grid from width and height.
    /// for the classic 2:1 "isometric" grid use with y = 2x
    pub fn dimetric(cell_size: Vec2) -> Self {
        Self::from_projection(
            cell_size,
            Mat2::from_cols(
                Vec2::new(cell_size.x / 2.0, -cell_size.y / 2.0),
                Vec2::new(-cell_size.x / 2.0, -cell_size.y / 2.0),
            ),
        )
    }

    /// true isometric: 120 degrees between axes, so width is sqrt(3) times height.
    pub fn isometric(size: f32) -> Self {
        let x = size * 3.0_f32.sqrt() / 2.0;
        let y = size / 2.0;
        Self::from_projection(Vec2::splat(size), Mat2::from_cols(Vec2::new(x, -y), Vec2::new(-x, -y)))
    }

    /// cavalier oblique: grid-x stays horizontal, grid-y shears up by `angle` at full depth.
    pub fn cavalier(cell_size: Vec2, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::from_projection(
            cell_size,
            Mat2::from_cols(
                Vec2::new(cell_size.x, 0.0),
                Vec2::new(cell_size.y * cos, cell_size.y * sin),
            ),
        )
    }
}
impl GridGeometry for QuadGridGeometry {
    type Grid = QuadGrid;
    type Position = Vec3;

    fn try_cell_center(&self, cell: impl Into<CellOf<Self::Grid>>) -> Option<Self::Position> {
        let cell = cell.into();
        // projection already carries cell_size, so no extra scale here.
        let local = self.projection * (cell.as_vec2() + 0.5);
        Some(self.swizzle.apply(local.extend(0.0)))
    }

    fn try_cell_corners(
        &self,
        cell: impl Into<CellOf<Self::Grid>>,
    ) -> Option<impl Iterator<Item = (CornerOf<Self::Grid>, Self::Position)>> {
        let cell = cell.into();
        let projection = self.projection;
        let swizzle = self.swizzle;
        Some(ALL_QUAD_CORNERS.into_iter().map(move |corner| {
            let local = projection * (cell.as_vec2() + corner.offset());
            (corner, swizzle.apply(local.extend(0.0)))
        }))
    }
}

impl TotalGridGeometry for QuadGridGeometry {}

impl PointQuery for QuadGridGeometry {
    fn cells_at(&self, local: Self::Position) -> impl Iterator<Item = CellOf<Self::Grid>> {
        let local = self.swizzle.invert(local).xy();
        let local = self.projection.inverse() * local;
        std::iter::once(local.floor().as_ivec2())
    }
}

impl TotalPointQuery for QuadGridGeometry {}

impl RayCast for QuadGridGeometry {
    /// The quad grid's neighbours are its cell coordinates plus an offset, so the march needs no
    /// topology and ignores the grid.
    fn raycast(
        &self,
        _grid: &Self::Grid,
        origin: Self::Position,
        dir: Self::Position,
    ) -> impl Iterator<Item = RayHitOf<Self::Grid>> {
        // Transform ray to grid space
        let inv = self.projection.inverse();
        let local_origin = self.swizzle.invert(origin).xy();
        let local_dir = self.swizzle.invert(dir).xy();

        let grid_origin = inv * local_origin;
        let grid_dir = inv * local_dir;

        // A ray with no direction has no cells to march.
        let degenerate = grid_dir.x.abs() < 1e-6 && grid_dir.y.abs() < 1e-6;

        let mut current = grid_origin.floor().as_ivec2();

        let step_x = if grid_dir.x > 0.0 { 1 } else { -1 };
        let step_y = if grid_dir.y > 0.0 { 1 } else { -1 };

        // t to cross one full cell along each axis.
        let t_delta_x = if grid_dir.x.abs() < 1e-6 {
            f32::INFINITY
        } else {
            (1.0 / grid_dir.x).abs()
        };
        let t_delta_y = if grid_dir.y.abs() < 1e-6 {
            f32::INFINITY
        } else {
            (1.0 / grid_dir.y).abs()
        };

        // t to the next grid boundary on each axis.
        let mut t_max_x = if grid_dir.x.abs() < 1e-6 {
            f32::INFINITY
        } else {
            let next_boundary = if grid_dir.x > 0.0 {
                current.x as f32 + 1.0
            } else {
                current.x as f32
            };
            (next_boundary - grid_origin.x) / grid_dir.x
        };

        let mut t_max_y = if grid_dir.y.abs() < 1e-6 {
            f32::INFINITY
        } else {
            let next_boundary = if grid_dir.y > 0.0 {
                current.y as f32 + 1.0
            } else {
                current.y as f32
            };
            (next_boundary - grid_origin.y) / grid_dir.y
        };
        let mut t = 0.0;
        let mut entry_face: Option<QuadDir> = None;

        std::iter::from_fn(move || {
            if degenerate {
                return None;
            }
            let hit = RayHit {
                cell: current,
                t,
                face: entry_face,
            };

            // Step to next cell
            if t_max_x < t_max_y {
                current.x += step_x;
                entry_face = Some(if step_x > 0 { QuadDir::W } else { QuadDir::E });
                t = t_max_x;
                t_max_x += t_delta_x;
            } else {
                current.y += step_y;
                entry_face = Some(if step_y > 0 { QuadDir::S } else { QuadDir::N });
                t = t_max_y;
                t_max_y += t_delta_y;
            }

            Some(hit)
        })
    }
}

impl Surface for QuadGridGeometry {
    fn pierce(&self, origin: Vec3, dir: Vec3) -> Option<(f32, Vec3)> {
        let h0 = self.height(origin);
        let rate = self.swizzle.invert(dir).z;

        // Parallel to (or lying in) the grid plane
        if rate.abs() < 1e-6 {
            return None;
        }

        let t = -h0 / rate;
        (t >= 0.0).then(|| (t, origin + t * dir))
    }
}

impl Layerable for QuadGridGeometry {
    fn lift(&self, point: Vec3, offset: f32) -> Vec3 {
        point + offset * self.swizzle.apply(Vec3::Z)
    }

    fn height(&self, point: Vec3) -> f32 {
        self.swizzle.invert(point).z
    }

    fn layer_crossings(&self, origin: Vec3, dir: Vec3, spacing: f32) -> impl Iterator<Item = (f32, i32)> {
        let h0 = self.height(origin);
        let rate = self.swizzle.invert(dir).z;

        // Bands are half-open [k, k+1), so a point on a boundary is inside the upper band: leaving
        // upward crosses nothing at t = 0 (first boundary strictly above, floor + 1), leaving
        // downward crosses immediately (boundary at-or-below, plain floor).
        let first_crossing = if rate > 1e-6 {
            let k = (h0 / spacing).floor() + 1.0;
            Some(((k * spacing - h0) / rate, 1))
        } else if rate < -1e-6 {
            let k = (h0 / spacing).floor();
            Some(((k * spacing - h0) / rate, -1))
        } else {
            None // In-plane ray never changes layer
        };

        first_crossing.into_iter().flat_map(move |(t0, step)| {
            let dt = spacing / rate.abs();
            (0..).map(move |i| (t0 + i as f32 * dt, step))
        })
    }
}

#[cfg(test)]
mod tests {
    use glam::IVec2;

    use super::*;
    use crate::grid::Grid;

    #[test]
    fn test_point_near_center_maps_to_cell() {
        let geom = QuadGridGeometry::rect(Vec2::new(1.0, 1.0));
        let cell = IVec2::new(5, 3);
        assert_eq!(geom.cell_at(geom.cell_center(cell) + Vec3::new(0.3, -0.3, 0.0)), cell);
        assert_eq!(
            geom.cell_at(geom.cell_center(cell) + Vec3::new(0.6, 0.0, 0.0)),
            cell + IVec2::X
        );
    }

    #[test]
    fn cell_at_round_trips_with_non_unit_cell_size() {
        let geom = QuadGridGeometry::rect(Vec2::new(32.0, 16.0));
        for cell in [
            IVec2::new(0, 0),
            IVec2::new(5, 3),
            IVec2::new(-2, 7),
            IVec2::new(-4, -6),
        ] {
            assert_eq!(geom.cell_at(geom.cell_center(cell)), cell);
        }
    }

    // A diagonal ray from the centre of cell (1,3) on a 32x16 grid, and the cells it
    // marches through (hand-verified). Shared by the geometric-property tests below.
    fn diagonal() -> (QuadGridGeometry, Vec3, Vec3, Vec<IVec2>) {
        let geom = QuadGridGeometry::rect(Vec2::new(32.0, 16.0));
        let origin = Vec3::new(48.0, 56.0, 0.0);
        let dir = Vec3::new(-1.0, -1.0, 0.0);
        let cells = vec![
            IVec2::new(1, 3),
            IVec2::new(1, 2),
            IVec2::new(0, 2),
            IVec2::new(0, 1),
            IVec2::new(0, 0),
            IVec2::new(-1, 0),
            IVec2::new(-1, -1),
        ];
        (geom, origin, dir, cells)
    }

    #[test]
    fn raycast_marches_cells_in_ray_order() {
        let (geom, origin, dir, cells) = diagonal();
        let hits: Vec<_> = geom.raycast(&QuadGrid {}, origin, dir).take(cells.len() + 1).collect();
        assert_eq!(hits.len(), cells.len() + 1, "unexpected hit count");
        for (hit, &cell) in hits.iter().zip(&cells) {
            assert_eq!(hit.cell, cell);
        }
    }

    #[test]
    fn raycast_t_is_nondecreasing() {
        let (geom, origin, dir, cells) = diagonal();
        let hits: Vec<_> = geom.raycast(&QuadGrid {}, origin, dir).take(cells.len() + 1).collect();
        for w in hits.windows(2) {
            assert!(w[1].t >= w[0].t, "t must be non-decreasing: {} < {}", w[1].t, w[0].t);
        }
    }

    #[test]
    fn raycast_consecutive_cells_are_edge_adjacent() {
        let (geom, origin, dir, cells) = diagonal();
        let hits: Vec<_> = geom.raycast(&QuadGrid {}, origin, dir).take(cells.len() + 1).collect();
        for w in hits.windows(2) {
            let delta = (w[1].cell - w[0].cell).abs();
            assert!(
                (delta.x == 1 && delta.y == 0) || (delta.x == 0 && delta.y == 1),
                "consecutive cells must be edge-adjacent: {:?} -> {:?}",
                w[0].cell,
                w[1].cell
            );
        }
    }

    #[test]
    fn raycast_midpoint_samples_land_in_reported_cell() {
        let (geom, origin, dir, cells) = diagonal();
        let hits: Vec<_> = geom.raycast(&QuadGrid {}, origin, dir).take(cells.len() + 1).collect();
        for w in hits.windows(2) {
            let mid_t = (w[0].t + w[1].t) / 2.0;
            let sample = origin + mid_t * dir;
            assert_eq!(
                geom.cell_at(sample),
                w[0].cell,
                "ray at midpoint t={} should land in cell {:?}",
                mid_t,
                w[0].cell
            );
        }
    }

    #[test]
    fn raycast_entry_face_walks_back_to_previous_cell() {
        // Opposite-sign direction so the x and y step directions disagree: the case a
        // same-sign ray can't distinguish, and the one that exercises the y-face branch.
        let geom = QuadGridGeometry::rect(Vec2::new(32.0, 16.0));
        let grid = QuadGrid {};
        let origin = Vec3::new(48.0, 56.0, 0.0);
        let dir = Vec3::new(1.0, -1.0, 0.0);
        let hits: Vec<_> = geom.raycast(&QuadGrid {}, origin, dir).take(8).collect();

        assert!(hits[0].face.is_none(), "origin cell has no entry face");
        for w in hits.windows(2) {
            let (prev, cur) = (w[0], w[1]);
            assert_eq!(
                grid.try_neighbour(cur.cell, cur.face.unwrap()),
                Some(prev.cell),
                "entry face of {:?} must walk back to {:?}",
                cur.cell,
                prev.cell
            );
        }
    }

    // Bands are half-open [k, k+1): a point on a boundary is inside the upper band, so descending
    // leaves through the boundary at t = 0 while ascending crosses nothing until the next one up.
    #[test]
    fn layer_crossings_descending_from_a_boundary_crosses_immediately() {
        let geom = QuadGridGeometry::rect(Vec2::ONE);
        let crossings: Vec<_> = geom
            .layer_crossings(Vec3::new(0.5, 0.5, 2.0), Vec3::new(0.0, 0.0, -1.0), 1.0)
            .take(3)
            .collect();
        assert_eq!(crossings, vec![(0.0, -1), (1.0, -1), (2.0, -1)]);
    }

    #[test]
    fn layer_crossings_ascending_from_a_boundary_skips_it() {
        let geom = QuadGridGeometry::rect(Vec2::ONE);
        let crossings: Vec<_> = geom
            .layer_crossings(Vec3::new(0.5, 0.5, 2.0), Vec3::new(0.0, 0.0, 1.0), 1.0)
            .take(2)
            .collect();
        assert_eq!(crossings, vec![(1.0, 1), (2.0, 1)]);
    }
}
