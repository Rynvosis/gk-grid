use glam::{Mat2, Vec2, Vec3, Vec3Swizzles};

use crate::{
    grid::{
        CellOf, CornerOf,
        geometry::{GridGeometry, PointQuery, RayCast, RayHit, TotalGridGeometry, TotalPointQuery},
        swizzle::GridSwizzle,
    },
    quad::{ALL_QUAD_CORNERS, QuadGrid},
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
    /// rectangle grid of cell size
    pub fn rect_grid(cell_size: Vec2) -> Self {
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
    fn raycast(&self, origin: Self::Position, dir: Self::Position) -> impl Iterator<Item = RayHit<CellOf<Self::Grid>>> {
        // Transform ray to grid space
        let inv = self.projection.inverse();
        let local_origin = self.swizzle.invert(origin).xy();
        let local_dir = self.swizzle.invert(dir).xy();

        let grid_origin = inv * local_origin;
        let grid_dir = inv * local_dir;

        // A ray with no direction has no cells to march.
        let degenerate = grid_dir.x.abs() < 1e-10 && grid_dir.y.abs() < 1e-10;

        // Starting cell
        let mut current = grid_origin.floor().as_ivec2();

        // Step directions
        let step_x = if grid_dir.x > 0.0 { 1 } else { -1 };
        let step_y = if grid_dir.y > 0.0 { 1 } else { -1 };

        // Calculate t_delta (how far along ray to cross one cell)
        let t_delta_x = if grid_dir.x.abs() < 1e-10 {
            f32::INFINITY
        } else {
            (1.0 / grid_dir.x).abs()
        };
        let t_delta_y = if grid_dir.y.abs() < 1e-10 {
            f32::INFINITY
        } else {
            (1.0 / grid_dir.y).abs()
        };

        // Calculate t_max (t value to next grid boundary)
        let mut t_max_x = if grid_dir.x.abs() < 1e-10 {
            f32::INFINITY
        } else {
            let next_boundary = if grid_dir.x > 0.0 {
                current.x as f32 + 1.0
            } else {
                current.x as f32
            };
            (next_boundary - grid_origin.x) / grid_dir.x
        };

        let mut t_max_y = if grid_dir.y.abs() < 1e-10 {
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

        std::iter::from_fn(move || {
            if degenerate {
                return None;
            }
            let hit = RayHit { cell: current, t };

            // Step to next cell
            if t_max_x < t_max_y {
                current.x += step_x;
                t = t_max_x;
                t_max_x += t_delta_x;
            } else {
                current.y += step_y;
                t = t_max_y;
                t_max_y += t_delta_y;
            }

            Some(hit)
        })
    }
}

#[cfg(test)]
mod tests {
    use glam::IVec2;

    use super::*;

    #[test]
    fn test_point_near_center_maps_to_cell() {
        let geom = QuadGridGeometry::rect_grid(Vec2::new(1.0, 1.0));
        let cell = IVec2::new(5, 3);
        assert_eq!(geom.cell_at(geom.cell_center(cell) + Vec3::new(0.3, -0.3, 0.0)), cell);
        assert_eq!(
            geom.cell_at(geom.cell_center(cell) + Vec3::new(0.6, 0.0, 0.0)),
            cell + IVec2::X
        );
    }

    #[test]
    fn cell_at_round_trips_with_non_unit_cell_size() {
        let geom = QuadGridGeometry::rect_grid(Vec2::new(32.0, 16.0));
        for cell in [
            IVec2::new(0, 0),
            IVec2::new(5, 3),
            IVec2::new(-2, 7),
            IVec2::new(-4, -6),
        ] {
            assert_eq!(geom.cell_at(geom.cell_center(cell)), cell);
        }
    }

    #[test]
    fn raycast_produces_edge_adjacent_cells_with_monotonic_t() {
        let geom = QuadGridGeometry::rect_grid(Vec2::new(32.0, 16.0));
        let dir = Vec3::new(-1.0, -1.0, 0.0);
        let origin = Vec3::new(48.0, 56.0, 0.0); // Center of cell (1, 3)

        let target_hits: [IVec2; 7] = [
            IVec2::new(1, 3),
            IVec2::new(1, 2),
            IVec2::new(0, 2),
            IVec2::new(0, 1),
            IVec2::new(0, 0),
            IVec2::new(-1, 0),
            IVec2::new(-1, -1),
        ];
        let hits: Vec<_> = geom.raycast(origin, dir).take(target_hits.len() + 1).collect();

        assert_eq!(
            hits.len(),
            target_hits.len() + 1,
            "raycast should produce expected number of hits"
        );

        let mut prev_t = -f32::INFINITY;
        for i in 0..target_hits.len() {
            let hit = hits[i];

            // Check cell matches expected
            assert_eq!(hit.cell, target_hits[i]);

            // Check t is non-decreasing
            assert!(
                hit.t >= prev_t,
                "t values must be non-decreasing: {} < {}",
                hit.t,
                prev_t
            );
            prev_t = hit.t;

            // Check consecutive cells are edge-adjacent (differ by 1 in exactly one axis)
            let next_hit = hits[i + 1];
            let delta = (next_hit.cell - hit.cell).abs();
            assert!(
                (delta.x == 1 && delta.y == 0) || (delta.x == 0 && delta.y == 1),
                "consecutive cells must be edge-adjacent: {:?} -> {:?}",
                hit.cell,
                next_hit.cell
            );

            // Check that sampling the midpoint between this t and next t lands in the correct cell
            let mid_t = (hit.t + next_hit.t) / 2.0;
            let sample_point = origin + mid_t * dir;
            let sampled_cell = geom.cell_at(sample_point);
            assert_eq!(
                sampled_cell, hit.cell,
                "ray sampled at midpoint t={} should land in cell {:?}",
                mid_t, hit.cell
            );
        }
    }
}
