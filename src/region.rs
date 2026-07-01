use crate::grid::GridCell;
use glam::IVec2;

pub trait Region {
    type Cell: GridCell;
    fn iter(&self) -> impl Iterator<Item = Self::Cell>;
    fn contains(&self, cell: Self::Cell) -> bool;
}

/// half-open rectangle region
pub struct RectRegion {
    pub min: IVec2,
    pub max: IVec2,
}
impl Region for RectRegion {
    type Cell = IVec2;
    fn iter(&self) -> impl Iterator<Item = Self::Cell> {
        (self.min.y..self.max.y)
            .flat_map(move |y| (self.min.x..self.max.x).map(move |x| IVec2::new(x, y)))
    }
    fn contains(&self, cell: Self::Cell) -> bool {
        self.min.x <= cell.x && cell.x < self.max.x && self.min.y <= cell.y && cell.y < self.max.y
    }
}
