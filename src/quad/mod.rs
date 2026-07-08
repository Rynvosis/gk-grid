pub(crate) mod geometry;
mod layout;

use crate::prelude::*;
use glam::{IVec2, UVec2, Vec2};
use crate::grid::TotalGrid;

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

    fn neighbours(&self, cell: impl Into<Self::Cell>) -> impl Iterator<Item=(Self::Slot, Self::Cell)> {
        let cell = cell.into();
        ALL_QUAD_DIRS
            .into_iter()
            .map(move |dir| (dir, cell + dir.delta()))
    }
}

impl TotalGrid for QuadGrid {}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum QuadDirs {
    E, N, W, S
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
    NE, NW, SW, SE
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
pub(crate) const ALL_QUAD_CORNERS: [QuadCorners; 4] = [QuadCorners::NE, QuadCorners::NW, QuadCorners::SW, QuadCorners::SE];

/// Uniform quad chunking: chunks of `size` cells, phase-shifted by `align`.
#[derive(Clone, Copy, Debug)]
pub struct QuadChunkLayout {
    pub size: UVec2,
    /// Cell where chunk (0,0)'s corner sits. ZERO is corner-aligned.
    pub align: IVec2,
}

impl QuadChunkLayout {
    /// Corner-aligned chunks: chunk (0,0) starts at cell (0,0).
    pub fn new(size: UVec2) -> Self {
        Self {
            size,
            align: IVec2::ZERO,
        }
    }

    /// Chunks phase-shifted so chunk (0,0)'s corner sits at `align`.
    pub fn with_align(size: UVec2, align: IVec2) -> Self {
        Self { size, align }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_near_center_maps_to_cell() {
        let geom = QuadGridGeometry::rect_grid(Vec2::new(1.0, 1.0));
        let cell = IVec2::new(5, 3);
        assert_eq!(
            geom.world_to_cell(geom.cell_center(cell) + Vec2::new(0.3, -0.3)),
            Some(cell)
        );
        assert_eq!(
            geom.world_to_cell(geom.cell_center(cell) + Vec2::new(0.6, 0.0)),
            Some(cell + IVec2::X)
        );
    }

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

    #[test]
    fn world_to_cell_round_trips_with_non_unit_cell_size() {
        let geom = QuadGridGeometry::rect_grid(Vec2::new(32.0, 16.0));
        for cell in [
            IVec2::new(0, 0),
            IVec2::new(5, 3),
            IVec2::new(-2, 7),
            IVec2::new(-4, -6),
        ] {
            assert_eq!(geom.world_to_cell(geom.cell_center(cell)), Some(cell));
        }
    }

    #[test]
    fn chunk_round_trip_corner_aligned() {
        let layout = QuadChunkLayout::new(UVec2::splat(16));
        for cell in [
            IVec2::new(0, 0),
            IVec2::new(15, 15),
            IVec2::new(16, 0),
            IVec2::new(-1, -1),
            IVec2::new(-16, 5),
            IVec2::new(100, -50),
        ] {
            let chunk = layout.chunk_of(cell);
            let local = layout.local_of(cell);
            assert!(local.cmpge(IVec2::ZERO).all() && local.cmplt(IVec2::splat(16)).all());
            assert_eq!(layout.cell_at(chunk, local), cell);
        }
    }

    #[test]
    fn negative_cells_land_in_negative_chunks() {
        let layout = QuadChunkLayout::new(UVec2::splat(16));
        assert_eq!(layout.chunk_of(IVec2::new(-1, -1)), IVec2::new(-1, -1));
        assert_eq!(layout.local_of(IVec2::new(-1, -1)), IVec2::new(15, 15));
    }

    #[test]
    fn align_shifts_chunk_boundaries() {
        // align (8,8): chunk (0,0) now covers cells [8, 24)
        let layout = QuadChunkLayout::with_align(UVec2::splat(16), IVec2::splat(8));
        assert_eq!(layout.chunk_of(IVec2::splat(8)), IVec2::ZERO);
        assert_eq!(layout.local_of(IVec2::splat(8)), IVec2::ZERO);
        assert_eq!(layout.chunk_of(IVec2::splat(7)), IVec2::splat(-1));
        let cell = IVec2::new(3, 20);
        assert_eq!(
            layout.cell_at(layout.chunk_of(cell), layout.local_of(cell)),
            cell
        );
    }

    #[test]
    fn chunk_region_covers_only_its_chunk() {
        let layout = QuadChunkLayout::new(UVec2::splat(4));
        let region = layout.chunk_region(IVec2::new(1, 0));
        assert_eq!(region.len(), 16);
        assert!(region.contains(IVec2::new(4, 0)));
        assert!(region.contains(IVec2::new(7, 3)));
        assert!(!region.contains(IVec2::new(3, 0)));
        for cell in region.iter() {
            assert_eq!(layout.chunk_of(cell), IVec2::new(1, 0));
        }
    }
}
