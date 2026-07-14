pub(crate) mod geometry;
mod layout;

use glam::{IVec2, Vec2};
pub use layout::QuadChunkLayout;

use crate::{
    grid::{Connection, ConnectionOf, TotalGrid},
    prelude::*,
};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct QuadGrid {}
impl Grid for QuadGrid {
    type Cell = IVec2;
    type Corner = QuadCorner;
    type Slot = QuadDir;

    fn slots(&self, _cell: impl Into<Self::Cell>) -> impl Iterator<Item = Self::Slot> {
        ALL_QUAD_DIRS.into_iter()
    }

    fn try_connection(
        &self,
        cell: impl Into<Self::Cell>,
        direction: impl Into<Self::Slot>,
    ) -> Option<ConnectionOf<Self>> {
        let direction = direction.into();
        Some(Connection::new(cell.into() + direction.delta(), direction.opposite()))
    }
}

impl TotalGrid for QuadGrid {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum QuadDir {
    E,
    N,
    W,
    S,
}
impl QuadDir {
    fn delta(self) -> IVec2 {
        match self {
            QuadDir::E => IVec2::new(1, 0),
            QuadDir::N => IVec2::new(0, 1),
            QuadDir::W => IVec2::new(-1, 0),
            QuadDir::S => IVec2::new(0, -1),
        }
    }

    /// The direction leading back, which is the slot the neighbour shares this edge by.
    pub fn opposite(self) -> Self {
        match self {
            QuadDir::E => QuadDir::W,
            QuadDir::N => QuadDir::S,
            QuadDir::W => QuadDir::E,
            QuadDir::S => QuadDir::N,
        }
    }
}
pub(crate) const ALL_QUAD_DIRS: [QuadDir; 4] = [QuadDir::E, QuadDir::N, QuadDir::W, QuadDir::S];

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum QuadCorner {
    NE,
    NW,
    SW,
    SE,
}
impl QuadCorner {
    fn offset(self) -> Vec2 {
        match self {
            QuadCorner::NE => Vec2::new(1.0, 1.0),
            QuadCorner::NW => Vec2::new(0.0, 1.0),
            QuadCorner::SW => Vec2::new(0.0, 0.0),
            QuadCorner::SE => Vec2::new(1.0, 0.0),
        }
    }
}
pub(crate) const ALL_QUAD_CORNERS: [QuadCorner; 4] = [QuadCorner::NE, QuadCorner::NW, QuadCorner::SW, QuadCorner::SE];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbours_yield_the_four_orthogonal_dirs_in_ccw_winding() {
        let grid = QuadGrid {};
        let cell = IVec2::new(2, 3);
        assert_eq!(
            grid.neighbours(cell).collect::<Vec<_>>(),
            vec![
                (QuadDir::E, IVec2::new(3, 3)),
                (QuadDir::N, IVec2::new(2, 4)),
                (QuadDir::W, IVec2::new(1, 3)),
                (QuadDir::S, IVec2::new(2, 2)),
            ]
        );
        assert_eq!(
            grid.slots(cell).collect::<Vec<_>>(),
            vec![QuadDir::E, QuadDir::N, QuadDir::W, QuadDir::S]
        );
        assert_eq!(grid.try_neighbour(cell, QuadDir::S), Some(IVec2::new(2, 2)));
    }
}
