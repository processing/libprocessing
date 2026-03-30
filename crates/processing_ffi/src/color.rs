use bevy::color::LinearRgba;
use processing::prelude::color::{ColorMode, ColorSpace};

/// A color with 4 float components and its color space.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub c1: f32,
    pub c2: f32,
    pub c3: f32,
    pub a: f32,
    pub space: u8,
}

impl Color {
    pub fn resolve(self, mode: &ColorMode) -> bevy::color::Color {
        let c1 = mode.scale(self.c1, 0);
        let c2 = mode.scale(self.c2, 1);
        let c3 = mode.scale(self.c3, 2);
        let ca = mode.scale(self.a, 3);
        mode.space.color(c1, c2, c3, ca)
    }

    pub fn from_linear(lin: LinearRgba) -> Self {
        Color {
            c1: lin.red,
            c2: lin.green,
            c3: lin.blue,
            a: lin.alpha,
            space: ColorSpace::Linear as u8,
        }
    }
}
