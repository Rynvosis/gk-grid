use crate::grid::GridCell;
use glam::IVec2;

pub trait Region {
    type Cell: GridCell;
    fn iter(&self) -> impl Iterator<Item = Self::Cell>;
    fn contains(&self, cell: Self::Cell) -> bool;
    fn index_of(&self, cell: Self::Cell) -> Option<usize> {
        self.iter().position(|c| c == cell)
    }
    fn len(&self) -> usize {
        self.iter().count()
    }
}

/// half-open rectangle region
#[derive(Clone, Debug)]
pub struct RectRegion {
    pub min: IVec2,
    pub max: IVec2,
}
impl RectRegion {
    pub fn new(corner1: IVec2, corner2: IVec2) -> Self {
        let min = IVec2::min(corner1, corner2);
        let max = IVec2::max(corner1, corner2);
        RectRegion { min, max }
    }
    fn to_local(&self, cell: IVec2) -> IVec2 {
        IVec2::new(cell.x - self.min.x, cell.y - self.min.y)
    }
    fn size(&self) -> IVec2 {
        (self.max - self.min).abs()
    }
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
    fn index_of(&self, cell: Self::Cell) -> Option<usize> {
        if !self.contains(cell) {
            return None;
        }
        let local = self.to_local(cell);
        let size = self.size();
        Some((local.y * size.x + local.x) as usize)
    }
    fn len(&self) -> usize {
        ((self.max.x - self.min.x) * (self.max.y - self.max.y)) as usize
    }
}
