use bevy::math::{Affine3A, Mat3, Vec3};

#[derive(Debug, Clone, Default)]
pub struct TransformStack {
    current: Affine3A,
    stack: Vec<Affine3A>,
}

impl TransformStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current(&self) -> Affine3A {
        self.current
    }

    pub fn push(&mut self) {
        self.stack.push(self.current);
    }

    pub fn pop(&mut self) {
        if let Some(t) = self.stack.pop() {
            self.current = t;
        }
    }

    pub fn reset(&mut self) {
        self.current = Affine3A::IDENTITY;
    }

    pub fn clear(&mut self) {
        self.current = Affine3A::IDENTITY;
        self.stack.clear();
    }

    pub fn apply(&mut self, transform: Affine3A) {
        self.current *= transform;
    }

    pub fn to_bevy_transform(&self) -> bevy::prelude::Transform {
        let (scale, rotation, translation) = self.current.to_scale_rotation_translation();
        bevy::prelude::Transform {
            translation,
            rotation,
            scale,
        }
    }

    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.current.transform_point3(point)
    }

    pub fn transform_point_2d(&self, x: f32, y: f32) -> (f32, f32) {
        let p = self.current.transform_point3(Vec3::new(x, y, 0.0));
        (p.x, p.y)
    }
}

pub fn shear_x(angle: f32) -> Affine3A {
    Affine3A::from_mat3(Mat3::from_cols(
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(angle.tan(), 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ))
}

pub fn shear_y(angle: f32) -> Affine3A {
    Affine3A::from_mat3(Mat3::from_cols(
        Vec3::new(1.0, angle.tan(), 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ))
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;

    static EPSILON: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_identity() {
        let stack = TransformStack::new();
        let (x, y) = stack.transform_point_2d(10.0, 20.0);
        assert!(approx_eq(x, 10.0));
        assert!(approx_eq(y, 20.0));
    }

    #[test]
    fn test_translate() {
        let mut stack = TransformStack::new();
        stack.apply(Affine3A::from_translation(Vec3::new(100.0, 50.0, 0.0)));
        let (x, y) = stack.transform_point_2d(10.0, 20.0);
        assert!(approx_eq(x, 110.0));
        assert!(approx_eq(y, 70.0));
    }

    #[test]
    fn test_scale() {
        let mut stack = TransformStack::new();
        stack.apply(Affine3A::from_scale(Vec3::new(2.0, 3.0, 1.0)));
        let (x, y) = stack.transform_point_2d(10.0, 10.0);
        assert!(approx_eq(x, 20.0));
        assert!(approx_eq(y, 30.0));
    }

    #[test]
    fn test_rotate_90() {
        let mut stack = TransformStack::new();
        stack.apply(Affine3A::from_rotation_z(PI / 2.0));
        let (x, y) = stack.transform_point_2d(10.0, 0.0);
        assert!(approx_eq(x, 0.0));
        assert!(approx_eq(y, 10.0));
    }

    #[test]
    fn test_push_pop() {
        let mut stack = TransformStack::new();
        stack.apply(Affine3A::from_translation(Vec3::new(100.0, 100.0, 0.0)));
        stack.push();
        stack.apply(Affine3A::from_translation(Vec3::new(50.0, 50.0, 0.0)));

        let (x, y) = stack.transform_point_2d(0.0, 0.0);
        assert!(approx_eq(x, 150.0));
        assert!(approx_eq(y, 150.0));

        stack.pop();

        let (x, y) = stack.transform_point_2d(0.0, 0.0);
        assert!(approx_eq(x, 100.0));
        assert!(approx_eq(y, 100.0));
    }

    #[test]
    fn test_pop_empty_is_noop() {
        let mut stack = TransformStack::new();
        stack.apply(Affine3A::from_translation(Vec3::new(50.0, 50.0, 0.0)));
        stack.pop();
        let (x, y) = stack.transform_point_2d(0.0, 0.0);
        assert!(approx_eq(x, 50.0));
        assert!(approx_eq(y, 50.0));
    }
}
