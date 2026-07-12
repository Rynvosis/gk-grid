//! Geometry for a layered grid: two swappable ways to place stacked surface cells in space.
//! `PlanarLayeredGeometry` stacks flat layers (and its raycast reports side walls); `RadialLayeredGeometry`
//! stacks concentric shells. Cells are surfaces, never volumes.

use glam::Vec3;

use crate::{
    grid::{
        CellOf, CornerOf,
        geometry::{GridGeometry, PointQuery, RayCast, RayHit, RayHitOf, TotalPointQuery},
    },
    layered::{LayeredCell, LayeredGrid, LayeredSlot},
};

/// Stacks flat layers along `normal`, evenly spaced by `spacing`. The raycast reports the side
/// walls the ray crosses within each layer's span (`LayeredSlot::Base`) as well as the caps.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct PlanarLayeredGeometry<Geo> {
    base: Geo,
    normal: Vec3,
    spacing: f32,
}

impl<Geo> PlanarLayeredGeometry<Geo> {
    /// Stacks `base` into flat layers `spacing` apart along `normal`.
    pub fn new(base: Geo, normal: Vec3, spacing: f32) -> Self {
        Self { base, normal, spacing }
    }

    fn lift(&self, point: Vec3, layer: i32) -> Vec3 {
        point + self.normal.normalize() * (self.spacing * layer as f32)
    }
}

impl<Geo> GridGeometry for PlanarLayeredGeometry<Geo>
where
    Geo: GridGeometry<Position = Vec3>,
{
    type Grid = LayeredGrid<Geo::Grid>;
    type Position = Vec3;

    fn try_cell_center(&self, cell: impl Into<CellOf<Self::Grid>>) -> Option<Self::Position> {
        let cell = cell.into();
        Some(self.lift(self.base.try_cell_center(cell.cell)?, cell.layer))
    }

    fn try_cell_corners(
        &self,
        cell: impl Into<CellOf<Self::Grid>>,
    ) -> Option<impl Iterator<Item = (CornerOf<Self::Grid>, Self::Position)>> {
        let cell = cell.into();
        let base_corners = self.base.try_cell_corners(cell.cell)?;
        Some(base_corners.map(move |(corner, base_corner)| (corner, self.lift(base_corner, cell.layer))))
    }
}

impl<Geo> RayCast for PlanarLayeredGeometry<Geo>
where
    Geo: RayCast + PointQuery + GridGeometry<Position = Vec3>,
{
    fn raycast(&self, origin: Self::Position, dir: Self::Position) -> impl Iterator<Item = RayHitOf<Self::Grid>> {
        let normal = self.normal.normalize();
        let spacing = self.spacing;
        let rate = dir.dot(normal);
        let height0 = origin.dot(normal);
        let mut layer = (height0 / spacing).floor() as i32;

        // Layer surfaces sit at height = k * spacing; a non-grazing ray crosses them evenly in t.
        let (layer_step, mut layer_t, layer_t_delta) = if rate.abs() < 1e-10 {
            (0, f32::INFINITY, f32::INFINITY)
        } else if rate > 0.0 {
            (1, ((layer + 1) as f32 * spacing - height0) / rate, spacing / rate.abs())
        } else {
            (-1, (layer as f32 * spacing - height0) / rate, spacing / rate.abs())
        };

        // The base grid marches the lateral cells and hands us each wall face; its first item is the
        // origin cell, so skip it and seed the starting column from a point query instead.
        let mut walls = self.base.raycast(origin, dir).skip(1).peekable();
        let mut base_cell = self.base.cells_at(origin).next();
        let mut started = false;

        std::iter::from_fn(move || {
            let cell = base_cell?;
            if !started {
                started = true;
                return Some(RayHit {
                    cell: LayeredCell::new(cell, layer),
                    t: 0.0,
                    face: None,
                });
            }

            let wall_t = walls.peek().map_or(f32::INFINITY, |hit| hit.t);
            if wall_t.is_finite() && wall_t <= layer_t {
                let hit = walls.next().unwrap();
                base_cell = Some(hit.cell);
                return Some(RayHit {
                    cell: LayeredCell::new(hit.cell, layer),
                    t: hit.t,
                    face: hit.face.map(LayeredSlot::Base),
                });
            }

            if layer_t.is_finite() {
                layer += layer_step;
                let t = layer_t;
                layer_t += layer_t_delta;
                // Moving up enters the upper cell through its Down cap, and vice versa.
                let cap = if layer_step > 0 {
                    LayeredSlot::Down
                } else {
                    LayeredSlot::Up
                };
                return Some(RayHit {
                    cell: LayeredCell::new(cell, layer),
                    t,
                    face: Some(cap),
                });
            }

            None
        })
    }
}

impl<Geo> PointQuery for PlanarLayeredGeometry<Geo>
where
    Geo: PointQuery + GridGeometry<Position = Vec3>,
{
    fn cells_at(&self, local: Self::Position) -> impl Iterator<Item = CellOf<Self::Grid>> {
        let normal = self.normal.normalize();
        let layer = (local.dot(normal) / self.spacing).floor() as i32;
        let base_point = local - normal * local.dot(normal);
        self.base
            .cells_at(base_point)
            .map(move |cell| LayeredCell::new(cell, layer))
    }
}

impl<Geo> TotalPointQuery for PlanarLayeredGeometry<Geo> where Geo: TotalPointQuery + GridGeometry<Position = Vec3> {}

/// Stacks concentric shells around `center`, spaced by `thickness`. Geometry only for now;
/// the radial raycast (ray-vs-shells) is a future addition.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct RadialLayeredGeometry<Geo> {
    base: Geo,
    center: Vec3,
    thickness: f32,
}

impl<Geo> RadialLayeredGeometry<Geo> {
    /// Stacks `base` into shells `thickness` apart, pushing out from `center`.
    pub fn new(base: Geo, center: Vec3, thickness: f32) -> Self {
        Self {
            base,
            center,
            thickness,
        }
    }

    fn lift(&self, point: Vec3, layer: i32) -> Vec3 {
        let radial = (point - self.center).normalize();
        point + radial * (self.thickness * layer as f32)
    }
}

impl<Geo> GridGeometry for RadialLayeredGeometry<Geo>
where
    Geo: GridGeometry<Position = Vec3>,
{
    type Grid = LayeredGrid<Geo::Grid>;
    type Position = Vec3;

    fn try_cell_center(&self, cell: impl Into<CellOf<Self::Grid>>) -> Option<Self::Position> {
        let cell = cell.into();
        Some(self.lift(self.base.try_cell_center(cell.cell)?, cell.layer))
    }

    fn try_cell_corners(
        &self,
        cell: impl Into<CellOf<Self::Grid>>,
    ) -> Option<impl Iterator<Item = (CornerOf<Self::Grid>, Self::Position)>> {
        let cell = cell.into();
        let base_corners = self.base.try_cell_corners(cell.cell)?;
        Some(base_corners.map(move |(corner, base_corner)| (corner, self.lift(base_corner, cell.layer))))
    }
}

#[cfg(test)]
mod tests {
    use glam::{IVec2, Vec2};

    use super::*;
    use crate::{
        grid::Grid,
        quad::{QuadGrid, geometry::QuadGridGeometry},
    };

    fn planar() -> PlanarLayeredGeometry<QuadGridGeometry> {
        PlanarLayeredGeometry::new(QuadGridGeometry::rect(Vec2::ONE), Vec3::Z, 1.0)
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

    #[test]
    fn radial_center_pushes_out_by_thickness_per_layer() {
        let geom = RadialLayeredGeometry::new(QuadGridGeometry::rect(Vec2::ONE), Vec3::ZERO, 1.0);
        let base = geom.base.try_cell_center(IVec2::new(0, 0)).unwrap();
        let c0 = geom.try_cell_center(LayeredCell::new(IVec2::new(0, 0), 0)).unwrap();
        let c1 = geom.try_cell_center(LayeredCell::new(IVec2::new(0, 0), 1)).unwrap();

        assert_eq!(c0, base);
        assert!((c1.distance(Vec3::ZERO) - c0.distance(Vec3::ZERO) - 1.0).abs() < 1e-5);
    }
}
