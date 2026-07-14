//! Geometry for a layered grid: one wrapper stacking any [`Layerable`] base into evenly spaced
//! layers along its normal field. Cells are surfaces, never volumes; a cell sits on its layer
//! surface and owns the band above it.

use glam::Vec3;

use crate::{
    grid::{
        CellOf, CornerOf,
        geometry::{GridGeometry, Layerable, PointQuery, RayCast, RayHit, RayHitOf, TotalPointQuery},
    },
    layered::{LayeredCell, LayeredGrid, LayeredSlot},
};

/// Stacks a [`Layerable`] base into layers `spacing` apart along its normal field.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct LayeredGeometry<Geo> {
    base: Geo,
    spacing: f32,
}

impl<Geo> LayeredGeometry<Geo> {
    /// Stacks `base` into layers `spacing` apart.
    pub fn new(base: Geo, spacing: f32) -> Self {
        Self { base, spacing }
    }
}

impl<Geo: Layerable> LayeredGeometry<Geo> {
    fn layer_of(&self, point: Vec3) -> i32 {
        // floor, not round: a cell owns the half-open band above its surface.
        (self.base.height(point) / self.spacing).floor() as i32
    }
}

impl<Geo: Layerable> GridGeometry for LayeredGeometry<Geo> {
    type Grid = LayeredGrid<Geo::Grid>;
    type Position = Vec3;

    fn try_cell_center(&self, cell: impl Into<CellOf<Self::Grid>>) -> Option<Self::Position> {
        let cell = cell.into();
        let center = self.base.try_cell_center(cell.cell)?;
        Some(self.base.lift(center, self.spacing * cell.layer as f32))
    }

    fn try_cell_corners(
        &self,
        cell: impl Into<CellOf<Self::Grid>>,
    ) -> Option<impl Iterator<Item = (CornerOf<Self::Grid>, Self::Position)>> {
        let cell = cell.into();
        let offset = self.spacing * cell.layer as f32;
        let base_corners = self.base.try_cell_corners(cell.cell)?;
        Some(base_corners.map(move |(corner, base_corner)| (corner, self.base.lift(base_corner, offset))))
    }
}

impl<Geo> RayCast for LayeredGeometry<Geo>
where
    Geo: Layerable + PointQuery,
{
    fn raycast(&self, origin: Self::Position, dir: Self::Position) -> impl Iterator<Item = RayHitOf<Self::Grid>> {
        let spacing = self.spacing;
        let mut layer = self.layer_of(origin);

        // The base march's first item is the origin cell, so skip it and seed from a point query instead.
        let mut walls = self.base.raycast(origin, dir).skip(1).peekable();
        let mut caps = self.base.layer_crossings(origin, dir, spacing).peekable();

        // Deliberately every cell containing the origin: folded bases seed multivalued. The lateral
        // march then resumes from one of them (arbitrary for a folded base); same deferred region fix
        // as the hole below. Exact for a single-cell base since the first wall overwrites it at once.
        let seeds: Vec<_> = self.base.cells_at(origin).collect();
        let mut base_cell = seeds.last().copied();
        let seed_hits = seeds.into_iter().map(move |cell| RayHit {
            cell: LayeredCell::new(cell, layer),
            t: 0.0,
            face: None,
        });

        let merged = std::iter::from_fn(move || {
            loop {
                let wall_t = walls.peek().map(|h| h.t);
                let cap_t = caps.peek().map(|&(t, _)| t);

                match (wall_t, cap_t) {
                    (None, None) => return None,
                    // Wall next; wins ties so the corner case enters the neighbouring column
                    // before stepping layers within it.
                    (Some(wt), ct) if ct.is_none_or(|c| wt <= c) => {
                        let hit = walls.next().unwrap();
                        base_cell = Some(hit.cell);
                        return Some(RayHit {
                            cell: LayeredCell::new(hit.cell, layer),
                            t: hit.t,
                            face: hit.face.map(LayeredSlot::Base),
                        });
                    }
                    _ => {
                        let (t, step) = caps.next().unwrap();
                        layer += step;
                        // Face of the entered cell that was crossed: going up enters through its underside.
                        let face = if step > 0 { LayeredSlot::Down } else { LayeredSlot::Up };
                        // Ray hasn't entered the base grid yet: advance the layer, emit nothing. Known
                        // hole: infinite caps with no base cell would spin here; closed once `raycast`
                        // takes a region.
                        let Some(cell) = base_cell else { continue };
                        return Some(RayHit {
                            cell: LayeredCell::new(cell, layer),
                            t,
                            face: Some(face),
                        });
                    }
                }
            }
        });

        seed_hits.chain(merged)
    }
}

impl<Geo> PointQuery for LayeredGeometry<Geo>
where
    Geo: Layerable + PointQuery,
{
    fn cells_at(&self, local: Self::Position) -> impl Iterator<Item = CellOf<Self::Grid>> {
        let layer = self.layer_of(local);
        self.base.cells_at(local).map(move |cell| LayeredCell::new(cell, layer))
    }
}

impl<Geo> TotalPointQuery for LayeredGeometry<Geo> where Geo: Layerable + TotalPointQuery {}

#[cfg(test)]
mod tests {
    use glam::{IVec2, Vec2};

    use super::*;
    use crate::{
        grid::Grid,
        quad::{QuadGrid, geometry::QuadGridGeometry},
    };

    fn planar() -> LayeredGeometry<QuadGridGeometry> {
        LayeredGeometry::new(QuadGridGeometry::rect(Vec2::ONE), 1.0)
    }

    #[test]
    fn planar_center_sits_on_the_layer_surface() {
        let geom = planar();
        assert_eq!(
            geom.try_cell_center(LayeredCell::new(IVec2::new(0, 0), 0)),
            Some(Vec3::new(0.5, 0.5, 0.0))
        );
        assert_eq!(
            geom.try_cell_center(LayeredCell::new(IVec2::new(0, 0), 2)),
            Some(Vec3::new(0.5, 0.5, 2.0))
        );
    }

    #[test]
    fn planar_corners_are_the_base_corners_on_one_surface() {
        let geom = planar();
        let corners: Vec<_> = geom
            .try_cell_corners(LayeredCell::new(IVec2::new(0, 0), 3))
            .unwrap()
            .collect();
        assert_eq!(corners.len(), 4);
        assert!(corners.iter().all(|(_, p)| p.z == 3.0));
    }

    // The shared-merge regression: walls from the base march interleaved with caps by t.
    #[test]
    fn planar_raycast_reports_walls_and_caps() {
        let geom = planar();
        let grid = LayeredGrid::new(QuadGrid {});
        // Chosen so wall crossings (x=1 at t=0.7, ...) and cap crossings (z=1 at t=0.45, ...) never coincide.
        let hits: Vec<_> = geom
            .raycast(Vec3::new(0.3, 0.5, 0.1), Vec3::new(1.0, 0.0, 2.0))
            .take(6)
            .collect();

        let cells: Vec<_> = hits.iter().map(|h| h.cell).collect();
        assert_eq!(
            cells,
            vec![
                LayeredCell::new(IVec2::new(0, 0), 0),
                LayeredCell::new(IVec2::new(0, 0), 1),
                LayeredCell::new(IVec2::new(1, 0), 1),
                LayeredCell::new(IVec2::new(1, 0), 2),
                LayeredCell::new(IVec2::new(1, 0), 3),
                LayeredCell::new(IVec2::new(2, 0), 3),
            ]
        );

        assert!(hits[0].face.is_none(), "origin cell has no entry face");
        for w in hits.windows(2) {
            assert!(w[1].t >= w[0].t, "t must be nondecreasing: {} < {}", w[1].t, w[0].t);
            assert_eq!(
                grid.try_neighbour(w[1].cell, w[1].face.unwrap()),
                Some(w[0].cell),
                "entry face of {:?} must walk back to {:?}",
                w[1].cell,
                w[0].cell
            );
        }
    }

    #[test]
    fn planar_cells_at_recovers_the_cell() {
        let geom = planar();
        assert_eq!(
            geom.cells_at(Vec3::new(1.5, 0.5, 2.5)).collect::<Vec<_>>(),
            vec![LayeredCell::new(IVec2::new(1, 0), 2)]
        );
    }
}
