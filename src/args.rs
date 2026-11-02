//! Argument traits that stand in for Processing's numeric and arity overloads.

use bevy::math::{Vec2, Vec3};

mod sealed {
    pub trait Sealed {}
}

/// Any primitive number, converted to `f32`.
pub trait Scalar: sealed::Sealed + Copy {
    fn to_f32(self) -> f32;
}

macro_rules! scalar {
    ($($t:ty),*) => {$(
        impl sealed::Sealed for $t {}
        impl Scalar for $t {
            #[inline]
            fn to_f32(self) -> f32 {
                self as f32
            }
        }
    )*};
}
scalar!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

/// A 2D or 3D point: `(x, y)`, `(x, y, z)`, `Vec2` or `Vec3`. 2D points get `z = 0`.
pub trait IntoPoint {
    fn into_point(self) -> Vec3;
}

impl<A: Scalar, B: Scalar> IntoPoint for (A, B) {
    fn into_point(self) -> Vec3 {
        Vec3::new(self.0.to_f32(), self.1.to_f32(), 0.0)
    }
}

impl<A: Scalar, B: Scalar, C: Scalar> IntoPoint for (A, B, C) {
    fn into_point(self) -> Vec3 {
        Vec3::new(self.0.to_f32(), self.1.to_f32(), self.2.to_f32())
    }
}

impl IntoPoint for Vec2 {
    fn into_point(self) -> Vec3 {
        self.extend(0.0)
    }
}

impl IntoPoint for Vec3 {
    fn into_point(self) -> Vec3 {
        self
    }
}

/// A per-axis size: one number for all axes, or `(x, y)` / `(x, y, z)`. Missing axes are `1`.
pub trait IntoExtent {
    fn into_extent(self) -> Vec3;
}

impl<T: Scalar> IntoExtent for T {
    fn into_extent(self) -> Vec3 {
        Vec3::splat(self.to_f32())
    }
}

impl<A: Scalar, B: Scalar> IntoExtent for (A, B) {
    fn into_extent(self) -> Vec3 {
        Vec3::new(self.0.to_f32(), self.1.to_f32(), 1.0)
    }
}

impl<A: Scalar, B: Scalar, C: Scalar> IntoExtent for (A, B, C) {
    fn into_extent(self) -> Vec3 {
        Vec3::new(self.0.to_f32(), self.1.to_f32(), self.2.to_f32())
    }
}

impl IntoExtent for Vec2 {
    fn into_extent(self) -> Vec3 {
        self.extend(1.0)
    }
}

impl IntoExtent for Vec3 {
    fn into_extent(self) -> Vec3 {
        self
    }
}
