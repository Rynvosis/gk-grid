use crate::prelude::*;
use glam::{Affine2, IVec2, UVec2, Vec2};

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct SquareGrid {
    pub cell_size: Vec2,
    pub projection: Affine2, //IDENTIFY for a plain square grid
}

impl SquareGrid {
    pub fn new(cell_size: Vec2) -> Self {
        Self::with_projection(cell_size, Affine2::IDENTITY)
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct SquareTilemap {
    pub region: RectRegion,
}
impl Tilemap for SquareTilemap {
    type TilemapRegion = RectRegion;

    fn region(&self) -> &Self::TilemapRegion {
        &self.region
    }
}

/// Uniform square chunking: chunks of `size` cells, phase-shifted by `align`.
#[derive(Clone, Copy, Debug)]
pub struct SquareChunkLayout {
    pub size: UVec2,
    /// Cell where chunk (0,0)'s corner sits. ZERO is corner-aligned.
    pub align: IVec2,
}

impl SquareChunkLayout {
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

impl ChunkLayout for SquareChunkLayout {
    type Cell = IVec2;
    type ChunkCoord = IVec2;
    type ChunkRegion = RectRegion;

    fn chunk_of(&self, cell: Self::Cell) -> Self::ChunkCoord {
        (cell - self.align).div_euclid(self.size.as_ivec2())
    }

    fn local_of(&self, cell: Self::Cell) -> Self::Cell {
        (cell - self.align).rem_euclid(self.size.as_ivec2())
    }

    fn cell_at(&self, chunk: Self::ChunkCoord, local: Self::Cell) -> Self::Cell {
        chunk * self.size.as_ivec2() + local + self.align
    }

    fn chunk_region(&self, chunk: Self::ChunkCoord) -> Self::ChunkRegion {
        RectRegion::new(chunk * self.size.as_ivec2() + self.align, self.size)
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

    #[test]
    fn chunk_round_trip_corner_aligned() {
        let layout = SquareChunkLayout::new(UVec2::splat(16));
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
        let layout = SquareChunkLayout::new(UVec2::splat(16));
        assert_eq!(layout.chunk_of(IVec2::new(-1, -1)), IVec2::new(-1, -1));
        assert_eq!(layout.local_of(IVec2::new(-1, -1)), IVec2::new(15, 15));
    }

    #[test]
    fn align_shifts_chunk_boundaries() {
        // align (8,8): chunk (0,0) now covers cells [8, 24)
        let layout = SquareChunkLayout::with_align(UVec2::splat(16), IVec2::splat(8));
        assert_eq!(layout.chunk_of(IVec2::splat(8)), IVec2::ZERO);
        assert_eq!(layout.local_of(IVec2::splat(8)), IVec2::ZERO);
        assert_eq!(layout.chunk_of(IVec2::splat(7)), IVec2::splat(-1));
        let cell = IVec2::new(3, 20);
        assert_eq!(layout.cell_at(layout.chunk_of(cell), layout.local_of(cell)), cell);
    }

    #[test]
    fn chunk_region_covers_only_its_chunk() {
        let layout = SquareChunkLayout::new(UVec2::splat(4));
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
