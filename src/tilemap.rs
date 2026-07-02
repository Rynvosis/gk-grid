use crate::region::RectRegion;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct Tilemap {
    pub region: RectRegion,
}
