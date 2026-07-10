use glam::{Vec3, Vec3Swizzles};

/// Reorders a planar grid's local XYZ axes onto world XYZ, matching Unity's
/// Grid component "Cell Swizzle" (XYZ, XZY, YXZ, YZX, ZXY, ZYX).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridSwizzle {
    #[default]
    Xyz,
    Xzy,
    Yxz,
    Yzx,
    Zxy,
    Zyx,
}
impl GridSwizzle {
    /// grid-local (x, y, z) reordered per the variant.
    pub fn apply(self, grid: Vec3) -> Vec3 {
        match self {
            Self::Xyz => grid,
            Self::Xzy => grid.xzy(),
            Self::Yxz => grid.yxz(),
            Self::Yzx => grid.yzx(),
            Self::Zxy => grid.zxy(),
            Self::Zyx => grid.zyx(),
        }
    }

    /// Inverse of `apply`.
    pub fn invert(self, swizzled: Vec3) -> Vec3 {
        match self {
            Self::Xyz => swizzled,
            Self::Xzy => swizzled.xzy(),
            Self::Yxz => swizzled.yxz(),
            Self::Yzx => swizzled.zxy(),
            Self::Zxy => swizzled.yzx(),
            Self::Zyx => swizzled.zyx(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invert_undoes_apply_for_every_variant() {
        let v = Vec3::new(1.0, 2.0, 3.0);

        assert_eq!(GridSwizzle::Xyz.invert(GridSwizzle::Xyz.apply(v)), v);
        assert_eq!(GridSwizzle::Xzy.invert(GridSwizzle::Xzy.apply(v)), v);
        assert_eq!(GridSwizzle::Yxz.invert(GridSwizzle::Yxz.apply(v)), v);
        assert_eq!(GridSwizzle::Yzx.invert(GridSwizzle::Yzx.apply(v)), v);
        assert_eq!(GridSwizzle::Zxy.invert(GridSwizzle::Zxy.apply(v)), v);
        assert_eq!(GridSwizzle::Zyx.invert(GridSwizzle::Zyx.apply(v)), v);
    }
}
