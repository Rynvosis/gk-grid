use crate::chunk::ChunkLayout;
use crate::grid::GridCell;
use crate::region::Region;
use std::collections::HashMap;

/// Per-cell data addressed by cell coordinate, over any backing.
pub trait TileStore {
    /// Type used for Grid Coordinates
    type Cell: GridCell;
    /// Type of Value in the store.
    type Item;
    /// Value at a cell, or None if the store holds nothing there.
    fn get(&self, cell: Self::Cell) -> Option<&Self::Item>;
    /// Mutable value at a cell, or None if the store holds nothing there.
    fn get_mut(&mut self, cell: Self::Cell) -> Option<&mut Self::Item>;
    /// Every cell the store holds data for.
    fn cells(&self) -> impl Iterator<Item = Self::Cell>;
}

/// Dense storage: one slot per cell, addressed by its region.
/// Null tiles? Make T an Option.
#[derive(Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct Dense<R, T> {
    region: R,
    tiles: Vec<T>,
}

impl<R: Region, T> Dense<R, T> {
    /// Builds full storage over a region, one value per cell from `fill`.
    pub fn from_region(region: R, fill: impl FnMut(R::Cell) -> T) -> Self {
        let tiles = region.iter().map(fill).collect();
        Self { region, tiles }
    }
}

impl<R: Region, T> TileStore for Dense<R, T> {
    type Cell = R::Cell;
    type Item = T;

    fn get(&self, cell: R::Cell) -> Option<&T> {
        self.region.index_of(cell).map(|i| &self.tiles[i])
    }

    fn get_mut(&mut self, cell: R::Cell) -> Option<&mut T> {
        self.region.index_of(cell).map(|i| &mut self.tiles[i])
    }

    fn cells(&self) -> impl Iterator<Item = R::Cell> {
        self.region.iter()
    }
}

/// Sparse storage: only populated cells, addressed by a hash map.
#[derive(Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct Sparse<C, T> {
    map: HashMap<C, T>,
}

impl<C: GridCell, T> Default for Sparse<C, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: GridCell, T> Sparse<C, T> {
    /// An empty sparse store.
    pub fn new() -> Self {
        Sparse {
            map: HashMap::new(),
        }
    }

    /// Sets the value at a cell, returning the previous one.
    pub fn insert(&mut self, cell: C, value: T) -> Option<T> {
        self.map.insert(cell, value)
    }
}

impl<C: GridCell, T> TileStore for Sparse<C, T> {
    type Cell = C;
    type Item = T;

    fn get(&self, cell: C) -> Option<&T> {
        self.map.get(&cell)
    }

    fn get_mut(&mut self, cell: C) -> Option<&mut T> {
        self.map.get_mut(&cell)
    }

    fn cells(&self) -> impl Iterator<Item = C> {
        self.map.keys().copied()
    }
}

/// Chunked storage: a sparse map of chunks, each an inner store.
#[derive(Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct Chunked<K: ChunkLayout, S> {
    layout: K,
    chunks: HashMap<K::ChunkCoord, S>,
}

impl<K: ChunkLayout, S> Chunked<K, S> {
    /// An empty chunked store over a chunk layout.
    pub fn new(layout: K) -> Self {
        Chunked {
            layout,
            chunks: HashMap::new(),
        }
    }
}

impl<K: ChunkLayout, S: TileStore<Cell = K::Cell>> TileStore for Chunked<K, S> {
    type Cell = K::Cell;
    type Item = S::Item;

    fn get(&self, cell: K::Cell) -> Option<&S::Item> {
        self.chunks.get(&self.layout.chunk_of(cell))?.get(cell)
    }

    fn get_mut(&mut self, cell: K::Cell) -> Option<&mut S::Item> {
        self.chunks
            .get_mut(&self.layout.chunk_of(cell))?
            .get_mut(cell)
    }

    fn cells(&self) -> impl Iterator<Item = K::Cell> {
        // already global, just flatten.
        let mut out = Vec::new();
        for store in self.chunks.values() {
            out.extend(store.cells());
        }
        out.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::region::RectRegion;
    use crate::square::SquareChunkLayout;
    use glam::{IVec2, UVec2};
    use std::collections::HashSet;

    // fill runs per cell, every cell reads back (storing coords catches x/y swaps),
    // oob is None, then get_mut writes one and leaves the rest.
    #[test]
    fn dense_round_trips() {
        let region = RectRegion::new(IVec2::new(-3, 2), UVec2::new(5, 4));
        let cells: Vec<_> = region.iter().collect();
        let mut calls = 0usize;
        let mut map = Dense::from_region(region, |c| {
            calls += 1;
            c
        });
        assert_eq!(calls, cells.len());
        for &c in &cells {
            assert_eq!(map.get(c), Some(&c));
        }
        assert_eq!(map.get(IVec2::new(100, 100)), None);
        *map.get_mut(cells[1]).unwrap() = IVec2::new(9, 9);
        assert_eq!(map.get(cells[1]), Some(&IVec2::new(9, 9)));
        assert_eq!(map.get(cells[0]), Some(&cells[0]));
    }

    // Sparse holds only inserted cells.
    #[test]
    fn sparse_stores_only_inserted_cells() {
        let mut map = Sparse::new();
        map.insert(IVec2::new(2, 5), 7);
        assert_eq!(map.get(IVec2::new(2, 5)), Some(&7));
        assert_eq!(map.get(IVec2::ZERO), None);
    }

    // get + cells() agree in global coords; a local/global mix-up breaks one of them.
    #[test]
    fn chunked_round_trips_global_cells() {
        let layout = SquareChunkLayout::new(UVec2::splat(4));
        let seeds = [IVec2::new(1, 2), IVec2::new(5, 6), IVec2::new(-2, 3)];
        let mut chunks = HashMap::new();
        for &cell in &seeds {
            let coord = layout.chunk_of(cell);
            chunks
                .entry(coord)
                .or_insert_with(|| Dense::from_region(layout.chunk_region(coord), |c| c));
        }
        let expected: HashSet<IVec2> = chunks
            .keys()
            .flat_map(|&coord| layout.chunk_region(coord).iter().collect::<Vec<_>>())
            .collect();
        let store = Chunked { layout, chunks };
        for &cell in &seeds {
            assert_eq!(store.get(cell), Some(&cell));
        }
        assert_eq!(store.cells().collect::<HashSet<_>>(), expected);
    }
}
