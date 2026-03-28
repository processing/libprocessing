use bevy::color::{Color, Hsla, Hsva, Hwba, Laba, Lcha, LinearRgba, Oklaba, Oklcha, Srgba, Xyza};
use bevy::prelude::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorSpace {
    Srgb = 0,
    Linear = 1,
    Hsl = 2,
    Hsv = 3,
    Hwb = 4,
    Oklab = 5,
    Oklch = 6,
    Lab = 7,
    Lch = 8,
    Xyz = 9,
}

impl ColorSpace {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Srgb),
            1 => Some(Self::Linear),
            2 => Some(Self::Hsl),
            3 => Some(Self::Hsv),
            4 => Some(Self::Hwb),
            5 => Some(Self::Oklab),
            6 => Some(Self::Oklch),
            7 => Some(Self::Lab),
            8 => Some(Self::Lch),
            9 => Some(Self::Xyz),
            _ => None,
        }
    }

    pub fn default_maxes(&self) -> [f32; 4] {
        match self {
            Self::Srgb | Self::Linear | Self::Oklab | Self::Xyz => [1.0, 1.0, 1.0, 1.0],
            Self::Hsl | Self::Hsv | Self::Hwb => [360.0, 1.0, 1.0, 1.0],
            Self::Oklch => [1.0, 1.0, 360.0, 1.0],
            Self::Lab => [100.0, 1.0, 1.0, 1.0],
            Self::Lch => [100.0, 1.0, 360.0, 1.0],
        }
    }

    pub fn color(self, c1: f32, c2: f32, c3: f32, alpha: f32) -> Color {
        match self {
            Self::Srgb => Color::Srgba(Srgba::new(c1, c2, c3, alpha)),
            Self::Linear => Color::LinearRgba(LinearRgba::new(c1, c2, c3, alpha)),
            Self::Hsl => Color::Hsla(Hsla::new(c1, c2, c3, alpha)),
            Self::Hsv => Color::Hsva(Hsva::new(c1, c2, c3, alpha)),
            Self::Hwb => Color::Hwba(Hwba::new(c1, c2, c3, alpha)),
            Self::Oklab => Color::Oklaba(Oklaba::new(c1, c2, c3, alpha)),
            Self::Oklch => Color::Oklcha(Oklcha::new(c1, c2, c3, alpha)),
            Self::Lab => Color::Laba(Laba::new(c1, c2, c3, alpha)),
            Self::Lch => Color::Lcha(Lcha::new(c1, c2, c3, alpha)),
            Self::Xyz => Color::Xyza(Xyza::new(c1, c2, c3, alpha)),
        }
    }

    pub fn gray(self, v: f32, alpha: f32) -> Color {
        match self {
            Self::Srgb | Self::Linear | Self::Xyz => self.color(v, v, v, alpha),
            Self::Hsl | Self::Hsv | Self::Hwb => self.color(0.0, 0.0, v, alpha),
            Self::Oklab | Self::Lab => self.color(v, 0.0, 0.0, alpha),
            Self::Oklch | Self::Lch => self.color(v, 0.0, 0.0, alpha),
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct ColorMode {
    pub space: ColorSpace,
    pub max: [f32; 4],
}

impl Default for ColorMode {
    fn default() -> Self {
        Self::with_defaults(ColorSpace::Srgb)
    }
}

impl ColorMode {
    pub fn new(space: ColorSpace, max1: f32, max2: f32, max3: f32, max_alpha: f32) -> Self {
        Self {
            space,
            max: [max1, max2, max3, max_alpha],
        }
    }

    pub fn with_defaults(space: ColorSpace) -> Self {
        Self {
            space,
            max: space.default_maxes(),
        }
    }

    pub fn with_uniform_max(space: ColorSpace, max: f32) -> Self {
        Self {
            space,
            max: [max, max, max, max],
        }
    }

    /// Scale a raw float value for a given channel to the 0-1 normalized range.
    pub fn scale(&self, value: f32, ch: usize) -> f32 {
        let native = self.space.default_maxes();
        value / self.max[ch] * native[ch]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::color::ColorToComponents;

    #[test]
    fn test_srgb_color() {
        let c = ColorSpace::Srgb.color(1.0, 0.0, 0.0, 1.0);
        let s: Srgba = c.into();
        assert!((s.red - 1.0).abs() < 1e-6);
        assert!(s.green.abs() < 1e-6);
    }

    #[test]
    fn test_hsl_color() {
        let c = ColorSpace::Hsl.color(180.0, 0.5, 0.5, 1.0);
        let h: Hsla = c.into();
        assert!((h.hue - 180.0).abs() < 0.5);
        assert!((h.saturation - 0.5).abs() < 0.01);
        assert!((h.lightness - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_hsv_red() {
        let c = ColorSpace::Hsv.color(0.0, 1.0, 1.0, 1.0);
        let s: Srgba = c.into();
        assert!((s.red - 1.0).abs() < 0.01);
        assert!(s.green < 0.01);
    }

    #[test]
    fn test_gray_srgb() {
        let c = ColorSpace::Srgb.gray(0.5, 1.0);
        let s: Srgba = c.into();
        assert!((s.red - 0.5).abs() < 0.01);
        assert!((s.green - 0.5).abs() < 0.01);
        assert!((s.blue - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_gray_hsl() {
        let c = ColorSpace::Hsl.gray(0.5, 1.0);
        let s: Srgba = c.into();
        assert!((s.red - 0.5).abs() < 0.05);
        assert!((s.green - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_scale_identity() {
        let mode = ColorMode::default();
        assert!((mode.scale(0.5, 0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_scale_255() {
        let mode = ColorMode::with_uniform_max(ColorSpace::Srgb, 255.0);
        assert!((mode.scale(255.0, 0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_scale_hsl_percent() {
        let mode = ColorMode::new(ColorSpace::Hsl, 360.0, 100.0, 100.0, 1.0);
        assert!((mode.scale(180.0, 0) - 180.0).abs() < 0.01);
        assert!((mode.scale(50.0, 1) - 0.5).abs() < 1e-4);
        assert!((mode.scale(50.0, 2) - 0.5).abs() < 1e-4);
    }
}
