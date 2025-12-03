use bevy::color::{LinearRgba, Srgba};

/// A sRGB (?) color
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Color> for bevy::color::Color {
    fn from(color: Color) -> Self {
        bevy::color::Color::srgba(color.r, color.g, color.b, color.a)
    }
}

impl From<LinearRgba> for Color {
    fn from(lin: LinearRgba) -> Self {
        let srgb: Srgba = lin.into();
        srgb.into()
    }
}

impl From<Srgba> for Color {
    fn from(srgb: Srgba) -> Self {
        Color {
            r: srgb.red,
            g: srgb.green,
            b: srgb.blue,
            a: srgb.alpha,
        }
    }
}
