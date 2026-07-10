use glam::{Mat2, Vec2, Vec3, Vec3Swizzles};

use crate::{
    grid::{
        CellOf, CornerOf,
        geometry::{GridGeometry, PointQuery, TotalGridGeometry},
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
    fn local_to_cell(&self, local: Self::Position) -> Option<CellOf<Self::Grid>> {
        let local = self.swizzle.invert(local).xy();
        let local = self.projection.inverse() * local;
        Some(local.floor().as_ivec2())
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
        assert_eq!(
            geom.local_to_cell(geom.cell_center(cell) + Vec3::new(0.3, -0.3, 0.0)),
            Some(cell)
        );
        assert_eq!(
            geom.local_to_cell(geom.cell_center(cell) + Vec3::new(0.6, 0.0, 0.0)),
            Some(cell + IVec2::X)
        );
    }

    #[test]
    fn local_to_cell_round_trips_with_non_unit_cell_size() {
        let geom = QuadGridGeometry::rect_grid(Vec2::new(32.0, 16.0));
        for cell in [
            IVec2::new(0, 0),
            IVec2::new(5, 3),
            IVec2::new(-2, 7),
            IVec2::new(-4, -6),
        ] {
            assert_eq!(geom.local_to_cell(geom.cell_center(cell)), Some(cell));
        }
    }
}
