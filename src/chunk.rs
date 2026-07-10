//! A chunk is a finite region of a grid.

use crate::{grid::GridCellIndex, region::Region};

/// Partitions a grid's cells into a tiling of chunks.
pub trait ChunkLayout {
    type Cell: GridCellIndex;
    type ChunkCoord: GridCellIndex;
    type ChunkRegion: Region<Cell = Self::Cell>;

    /// Which chunk a cell belongs to.
    fn chunk_of(&self, cell: Self::Cell) -> Self::ChunkCoord;

    /// A cell's position within its chunk.
    fn local_of(&self, cell: Self::Cell) -> Self::Cell;

    /// Recombines a chunk coord and local cell into a global cell.
    fn cell_at(&self, chunk: Self::ChunkCoord, local: Self::Cell) -> Self::Cell;

    /// A chunk's region, in global cell coords.
    fn chunk_region(&self, chunk: Self::ChunkCoord) -> Self::ChunkRegion;
}
