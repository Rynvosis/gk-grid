use glam::{IVec2, UVec2};

use crate::{chunk::ChunkLayout, prelude::RectRegion};

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

impl ChunkLayout for QuadChunkLayout {
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
    use crate::prelude::Region;

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
        assert_eq!(layout.cell_at(layout.chunk_of(cell), layout.local_of(cell)), cell);
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
