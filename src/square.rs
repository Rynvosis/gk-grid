use crate::grid::{Grid, GridGeometry, GridTopology, PointQuery};
use glam::{Affine2, IVec2, Vec2};

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct SquareGrid {
    pub cell_size: Vec2,
    pub projection: Affine2, //IDENTIFY for a plain square grid
}

impl SquareGrid {
    pub fn new(cell_size: Vec2) -> Self {
        Self {
            cell_size,
            projection: Affine2::IDENTITY,
        }
    }

    pub fn with_projection(cell_size: Vec2, projection: Affine2) -> Self {
        Self {
            cell_size,
            projection,
        }
    }

    //todo: isometric()/cavalier(), helper functions
}

impl Grid for SquareGrid {
    type Cell = IVec2;
}

impl GridTopology for SquareGrid {
    fn neighbours(&self, cell: Self::Cell) -> impl Iterator<Item = Self::Cell> {
        [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .map(|(x, y)| IVec2::new(cell.x + x, cell.y + y))
            .into_iter()
    }
}

impl GridGeometry for SquareGrid {
    type Position = Vec2;

    fn cell_to_world(&self, cell: Self::Cell) -> Self::Position {
        self.projection
            .transform_point2((cell.as_vec2() + 0.5) * self.cell_size) //square center
    }

    fn cell_corners(&self, cell: Self::Cell) -> impl Iterator<Item = Self::Position> {
        // NE, SE, SW, NW - clockwise
        [(1, 1), (1, 0), (0, 0), (0, 1)]
            .into_iter()
            .map(move |(dx, dy)| {
                self.projection
                    .transform_point2((cell + IVec2::new(dx, dy)).as_vec2() * self.cell_size)
            })
    }
}

impl PointQuery for SquareGrid {
    fn world_to_cell(&self, world: Self::Position) -> Option<Self::Cell> {
        let local = self.projection.inverse().transform_point2(world);
        Some((local / self.cell_size).floor().as_ivec2())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_near_center_maps_to_cell() {
        let grid = SquareGrid::new(Vec2::new(1.0, 1.0));
        let cell = IVec2::new(5, 3);
        assert_eq!(
            grid.world_to_cell(grid.cell_to_world(cell) + Vec2::new(0.3, -0.3)),
            Some(cell)
        );
        assert_eq!(
            grid.world_to_cell(grid.cell_to_world(cell) + Vec2::new(0.6, 0.0)),
            Some(cell + IVec2::X)
        );
    }
}
