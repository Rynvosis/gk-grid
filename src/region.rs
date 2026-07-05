use crate::grid::GridCell;
use glam::{IVec2, UVec2};

/// A bounded set of cells.
pub trait Region {
    type Cell: GridCell;
    /// Iterates every cell in the region.
    fn iter(&self) -> impl Iterator<Item = Self::Cell>;
    fn contains(&self, cell: Self::Cell) -> bool;
    /// Index of a cell in iteration order, or None if outside the region.
    fn index_of(&self, cell: Self::Cell) -> Option<usize> {
        self.iter().position(|c| c == cell)
    }
    /// Number of cells in the region.
    fn len(&self) -> usize {
        self.iter().count()
    }
}

/// A half-open rectangular region, origin inclusive and far edge exclusive.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct RectRegion {
    pub origin: IVec2,
    pub size: UVec2,
}
impl RectRegion {
    /// Builds a region from its origin and size.
    pub fn new(origin: IVec2, size: UVec2) -> Self {
        RectRegion { origin, size }
    }

    /// Builds a region from two corners, both inclusive.
    pub fn from_corners_inclusive(corner1: IVec2, corner2: IVec2) -> Self {
        let origin = IVec2::min(corner1, corner2);
        let size = (IVec2::max(corner1, corner2) - origin + IVec2::ONE).as_uvec2();
        RectRegion { origin, size }
    }

    // Assumes cell is already in bounds.
    fn to_local(&self, cell: IVec2) -> IVec2 {
        cell - self.origin
    }

    // One past the last cell in the region.
    fn end(&self) -> IVec2 {
        self.origin + self.size.as_ivec2()
    }

    fn clamp(&self, cell: IVec2) -> IVec2 {
        cell.clamp(self.origin, self.end() - 1)
    }
}
impl Region for RectRegion {
    type Cell = IVec2;
    fn iter(&self) -> impl Iterator<Item = Self::Cell> {
        let origin = self.origin;
        let end = self.end();
        (origin.y..end.y).flat_map(move |y| (origin.x..end.x).map(move |x| IVec2::new(x, y)))
    }
    fn contains(&self, cell: Self::Cell) -> bool {
        let end = self.end();
        self.origin.x <= cell.x && cell.x < end.x && self.origin.y <= cell.y && cell.y < end.y
    }
    fn index_of(&self, cell: Self::Cell) -> Option<usize> {
        if !self.contains(cell) {
            return None;
        }
        let local = self.to_local(cell);
        Some((local.y * self.size.x as i32 + local.x) as usize)
    }
    fn len(&self) -> usize {
        self.size.x as usize * self.size.y as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_of_and_len_agree_with_iter_on_negative_origin() {
        let region = RectRegion::new(IVec2::new(-4, -4), UVec2::new(6, 6));
        for cell in region.iter() {
            assert_eq!(region.index_of(cell), region.iter().position(|c| c == cell));
        }
        assert_eq!(region.len(), region.iter().count());
        assert_eq!(region.index_of(IVec2::new(100, 100)), None);
    }

    #[test]
    fn index_of_matches_row_major_order_on_non_square_region() {
        // catches x/y transposition, which a square region can't.
        let region = RectRegion::new(IVec2::ZERO, UVec2::new(5, 3));
        assert_eq!(region.index_of(IVec2::new(4, 0)), Some(4));
        assert_eq!(region.index_of(IVec2::new(0, 1)), Some(5));
        assert_eq!(region.len(), 15);
    }

    #[test]
    fn from_corners_inclusive_matches_new_with_bumped_far_corner() {
        let inclusive = RectRegion::from_corners_inclusive(IVec2::ZERO, IVec2::new(3, 3));
        let direct = RectRegion::new(IVec2::ZERO, UVec2::new(4, 4));
        assert_eq!(inclusive, direct);
    }

    #[test]
    fn from_corners_inclusive_same_point_is_a_single_cell() {
        let p = IVec2::new(2, 2);
        let region = RectRegion::from_corners_inclusive(p, p);
        assert_eq!(region.len(), 1);
        assert!(region.contains(p));
    }
}
