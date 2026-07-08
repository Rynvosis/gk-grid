use crate::grid::geometry::{GridGeometry, PointQuery, TotalGridGeometry};
use crate::grid::{CellOf, CornerOf};
use crate::quad::{QuadGrid, ALL_QUAD_CORNERS};
use glam::{Mat2, Vec2};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct QuadGridGeometry {
    cell_size: Vec2,
    projection: Mat2,
}
impl QuadGridGeometry {
    pub fn projection(&self) -> Mat2 {
        self.projection
    }
    pub fn cell_size(&self) -> Vec2 {
        self.cell_size
    }

    // constructors
    /// rectangle grid of cell size
    pub fn rect_grid(cell_size: Vec2) -> Self {
        Self {
            cell_size,
            projection: Mat2::from_diagonal(cell_size)
        }
    }

    /// dimetric grid from width and height.
    /// for the classic 2:1 "isometric" grid use with y = 2x
    pub fn dimetric(cell_size: Vec2) -> Self {
        Self {
            cell_size,
            projection: Mat2::from_cols(
                Vec2::new(cell_size.x/2.0, -cell_size.y/2.0),
                Vec2::new(-cell_size.x/2.0, -cell_size.y/2.0)
            )
        }
    }

    /// true isometric: 120 degrees between axes, so width is sqrt(3) times height.
    pub fn isometric(size: f32) -> Self {
        let x = size * 3.0_f32.sqrt() / 2.0;
        let y = size / 2.0;
        Self {
            cell_size: Vec2::splat(size),
            projection: Mat2::from_cols(
                Vec2::new(x, -y),
                Vec2::new(-x, -y)
            )
        }
    }



    /// cavalier oblique: grid-x stays horizontal, grid-y shears up by `angle` at full depth.
    pub fn cavalier(cell_size: Vec2, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self {
            cell_size,
            projection: Mat2::from_cols(
                Vec2::new(cell_size.x, 0.0),
                Vec2::new(cell_size.y * cos, cell_size.y * sin)
            )
        }
    }
}
impl GridGeometry for QuadGridGeometry {
    type Grid = QuadGrid;
    type Position = Vec2;

    fn try_cell_center(&self, cell: impl Into<CellOf<Self::Grid>>) -> Option<Self::Position> {
        let cell = cell.into();
        // projection already carries cell_size, so no extra scale here.
        Some(self.projection * (cell.as_vec2() + 0.5))
    }

    fn try_cell_corners(
        &self,
        cell: impl Into<CellOf<Self::Grid>>,
    ) -> Option<impl Iterator<Item = (CornerOf<Self::Grid>, Self::Position)>> {
        let cell = cell.into();
        let projection = self.projection;
        Some(
            ALL_QUAD_CORNERS
                .into_iter()
                .map(move |corner| (corner, projection * (cell.as_vec2() + corner.offset()))),
        )
    }
}

impl TotalGridGeometry for QuadGridGeometry {}

impl PointQuery for QuadGridGeometry {
    fn world_to_cell(&self, world: Self::Position) -> Option<CellOf<Self::Grid>> {
        let local = self.projection.inverse() * world;
        Some(local.floor().as_ivec2())
    }
}