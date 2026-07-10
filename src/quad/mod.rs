pub(crate) mod geometry;
mod layout;

use glam::{IVec2, Vec2};
pub use layout::QuadChunkLayout;

use crate::{grid::TotalGrid, prelude::*};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct QuadGrid {}
impl Grid for QuadGrid {
    type Cell = IVec2;
    type Corner = QuadCorners;
    type Slot = QuadDirs;

    fn slots(&self, _cell: impl Into<Self::Cell>) -> impl Iterator<Item = Self::Slot> {
        ALL_QUAD_DIRS.into_iter()
    }

    fn try_neighbour(&self, cell: impl Into<Self::Cell>, direction: impl Into<Self::Slot>) -> Option<Self::Cell> {
        Some(cell.into() + direction.into().delta())
    }

    fn neighbours(&self, cell: impl Into<Self::Cell>) -> impl Iterator<Item = (Self::Slot, Self::Cell)> {
        let cell = cell.into();
        ALL_QUAD_DIRS.into_iter().map(move |dir| (dir, cell + dir.delta()))
    }
}

impl TotalGrid for QuadGrid {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum QuadDirs {
    E,
    N,
    W,
    S,
}
impl QuadDirs {
    fn delta(self) -> IVec2 {
        match self {
            QuadDirs::E => IVec2::new(1, 0),
            QuadDirs::N => IVec2::new(0, 1),
            QuadDirs::W => IVec2::new(-1, 0),
            QuadDirs::S => IVec2::new(0, -1),
        }
    }
}
pub(crate) const ALL_QUAD_DIRS: [QuadDirs; 4] = [QuadDirs::E, QuadDirs::N, QuadDirs::W, QuadDirs::S];

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum QuadCorners {
    NE,
    NW,
    SW,
    SE,
}
impl QuadCorners {
    fn offset(self) -> Vec2 {
        match self {
            QuadCorners::NE => Vec2::new(1.0, 1.0),
            QuadCorners::NW => Vec2::new(0.0, 1.0),
            QuadCorners::SW => Vec2::new(0.0, 0.0),
            QuadCorners::SE => Vec2::new(1.0, 0.0),
        }
    }
}
pub(crate) const ALL_QUAD_CORNERS: [QuadCorners; 4] =
    [QuadCorners::NE, QuadCorners::NW, QuadCorners::SW, QuadCorners::SE];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbours_yield_the_four_orthogonals_in_ccw_winding() {
        let grid = QuadGrid {};
        let cell = IVec2::new(2, 3);
        assert_eq!(
            grid.neighbours(cell).collect::<Vec<_>>(),
            vec![
                (QuadDirs::E, IVec2::new(3, 3)),
                (QuadDirs::N, IVec2::new(2, 4)),
                (QuadDirs::W, IVec2::new(1, 3)),
                (QuadDirs::S, IVec2::new(2, 2)),
            ]
        );
        assert_eq!(
            grid.slots(cell).collect::<Vec<_>>(),
            vec![QuadDirs::E, QuadDirs::N, QuadDirs::W, QuadDirs::S]
        );
        assert_eq!(grid.try_neighbour(cell, QuadDirs::S), Some(IVec2::new(2, 2)));
    }
}
