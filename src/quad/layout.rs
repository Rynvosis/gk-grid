use crate::chunk::ChunkLayout;
use crate::prelude::{QuadChunkLayout, RectRegion};
use glam::IVec2;

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
