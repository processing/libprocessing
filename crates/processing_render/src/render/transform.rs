use bevy::math::{Affine3A, Mat3, Quat, Vec3};

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

    pub fn translate(&mut self, x: f32, y: f32) {
        self.translate_3d(x, y, 0.0);
    }

    pub fn rotate(&mut self, angle: f32) {
        self.rotate_z(angle);
    }

    pub fn scale_uniform(&mut self, s: f32) {
        self.scale(s, s);
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.scale_3d(sx, sy, 1.0);
    }

    pub fn shear_x(&mut self, angle: f32) {
        let shear = Affine3A::from_mat3(Mat3::from_cols(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(angle.tan(), 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ));
        self.current = self.current * shear;
    }

    pub fn shear_y(&mut self, angle: f32) {
        let shear = Affine3A::from_mat3(Mat3::from_cols(
            Vec3::new(1.0, angle.tan(), 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ));
        self.current = self.current * shear;
    }

    pub fn translate_3d(&mut self, x: f32, y: f32, z: f32) {
        let t = Affine3A::from_translation(Vec3::new(x, y, z));
        self.current = self.current * t;
    }

    pub fn rotate_x(&mut self, angle: f32) {
        let r = Affine3A::from_quat(Quat::from_rotation_x(angle));
        self.current = self.current * r;
    }

    pub fn rotate_y(&mut self, angle: f32) {
        let r = Affine3A::from_quat(Quat::from_rotation_y(angle));
        self.current = self.current * r;
    }

    pub fn rotate_z(&mut self, angle: f32) {
        let r = Affine3A::from_quat(Quat::from_rotation_z(angle));
        self.current = self.current * r;
    }

    pub fn rotate_axis(&mut self, angle: f32, axis: Vec3) {
        let r = Affine3A::from_quat(Quat::from_axis_angle(axis.normalize(), angle));
        self.current = self.current * r;
    }

    pub fn scale_3d(&mut self, sx: f32, sy: f32, sz: f32) {
        let s = Affine3A::from_scale(Vec3::new(sx, sy, sz));
        self.current = self.current * s;
    }

    pub fn apply(&mut self, transform: Affine3A) {
        self.current = self.current * transform;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

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
        stack.translate(100.0, 50.0);
        let (x, y) = stack.transform_point_2d(10.0, 20.0);
        assert!(approx_eq(x, 110.0));
        assert!(approx_eq(y, 70.0));
    }

    #[test]
    fn test_scale() {
        let mut stack = TransformStack::new();
        stack.scale(2.0, 3.0);
        let (x, y) = stack.transform_point_2d(10.0, 10.0);
        assert!(approx_eq(x, 20.0));
        assert!(approx_eq(y, 30.0));
    }

    #[test]
    fn test_rotate_90() {
        let mut stack = TransformStack::new();
        stack.rotate(PI / 2.0);
        let (x, y) = stack.transform_point_2d(10.0, 0.0);
        assert!(approx_eq(x, 0.0));
        assert!(approx_eq(y, 10.0));
    }

    #[test]
    fn test_push_pop() {
        let mut stack = TransformStack::new();
        stack.translate(100.0, 100.0);
        stack.push();
        stack.translate(50.0, 50.0);

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
        stack.translate(50.0, 50.0);
        stack.pop();
        let (x, y) = stack.transform_point_2d(0.0, 0.0);
        assert!(approx_eq(x, 50.0));
        assert!(approx_eq(y, 50.0));
    }
}
