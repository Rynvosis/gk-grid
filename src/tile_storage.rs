use crate::region::Region;

/// Dense per-cell data over a region, one slot per cell in iteration order.
#[derive(Clone, Debug)]
pub struct TileStorage<T> {
    tiles: Vec<T>,
}

impl<T> TileStorage<T> {
    /// Builds full storage over a region, one value per cell from `fill`.
    pub fn from_region<R: Region>(region: &R, fill: impl FnMut(R::Cell) -> T) -> Self {
        todo!()
    }

    /// Value at a cell, or None if outside the region.
    pub fn get<R: Region>(&self, region: &R, cell: R::Cell) -> Option<&T> {
        todo!()
    }

    /// Mutable value at a cell, or None if outside the region.
    pub fn get_mut<R: Region>(&mut self, region: &R, cell: R::Cell) -> Option<&mut T> {
        todo!()
    }
}
