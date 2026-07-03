use crate::region::Region;

pub trait Tilemap {
    type TilemapRegion : Region;
    fn region(&self) -> &Self::TilemapRegion;
}