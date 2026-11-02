//! Colors and the `fill(..)`-style color arguments.

use bevy::color::{Alpha, Color as BevyColor, Hsla, Hsva, LinearRgba, Mix, Oklcha, Srgba};
use processing_render::color::ColorMode;

pub use processing_render::color::ColorSpace;

/// A resolved color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub(crate) BevyColor);

impl Color {
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

    /// sRGB, channels 0-1.
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    /// sRGB with alpha, channels 0-1.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(BevyColor::Srgba(Srgba::new(r, g, b, a)))
    }

    /// sRGB, channels 0-255.
    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba8(r, g, b, 255)
    }

    /// sRGB with alpha, channels 0-255.
    pub const fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::rgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// `0xRRGGBB`.
    pub const fn hex(rgb: u32) -> Self {
        Self::rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
    }

    /// `0xRRGGBBAA`.
    pub const fn hexa(rgba: u32) -> Self {
        Self::rgba8(
            (rgba >> 24) as u8,
            (rgba >> 16) as u8,
            (rgba >> 8) as u8,
            rgba as u8,
        )
    }

    /// Hue in degrees, saturation and lightness 0-1.
    pub const fn hsl(h: f32, s: f32, l: f32) -> Self {
        Self(BevyColor::Hsla(Hsla::new(h, s, l, 1.0)))
    }

    /// Hue in degrees, saturation and value 0-1.
    pub const fn hsv(h: f32, s: f32, v: f32) -> Self {
        Self(BevyColor::Hsva(Hsva::new(h, s, v, 1.0)))
    }

    /// Lightness 0-1, chroma 0-~0.4, hue in degrees.
    pub const fn oklch(l: f32, c: f32, h: f32) -> Self {
        Self(BevyColor::Oklcha(Oklcha::new(l, c, h, 1.0)))
    }

    pub fn with_alpha(self, alpha: f32) -> Self {
        let mut c = self.0;
        c.set_alpha(alpha);
        Self(c)
    }

    /// Interpolates in Oklab, like Processing's `lerpColor`.
    pub fn lerp(self, other: Color, t: f32) -> Self {
        let a: bevy::color::Oklaba = self.0.into();
        let b: bevy::color::Oklaba = other.0.into();
        Self(a.mix(&b, t).into())
    }

    pub fn red(self) -> f32 {
        self.0.to_srgba().red
    }

    pub fn green(self) -> f32 {
        self.0.to_srgba().green
    }

    pub fn blue(self) -> f32 {
        self.0.to_srgba().blue
    }

    pub fn alpha(self) -> f32 {
        self.0.alpha()
    }

    pub(crate) fn from_linear(c: LinearRgba) -> Self {
        Self(BevyColor::LinearRgba(c))
    }

    pub(crate) fn to_linear(self) -> LinearRgba {
        self.0.to_linear()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Channel {
    Int(i64),
    Float(f32),
}

/// A single color channel: integers are 0-255, floats are 0-1, unless `color_mode().max(..)` is set.
pub trait IntoChannel: Copy {
    fn into_channel(self) -> Channel;
}

macro_rules! int_channel {
    ($($t:ty),*) => {$(
        impl IntoChannel for $t {
            fn into_channel(self) -> Channel {
                Channel::Int(self as i64)
            }
        }
    )*};
}
int_channel!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

impl IntoChannel for f32 {
    fn into_channel(self) -> Channel {
        Channel::Float(self)
    }
}

impl IntoChannel for f64 {
    fn into_channel(self) -> Channel {
        Channel::Float(self as f32)
    }
}

/// An unresolved color, interpreted against the current color mode when used.
#[derive(Debug, Clone, Copy)]
pub enum ColorArg {
    Color(Color),
    Gray(Channel, Option<Channel>),
    Channels([Channel; 3], Option<Channel>),
}

impl From<Color> for ColorArg {
    fn from(c: Color) -> Self {
        Self::Color(c)
    }
}

impl<A: IntoChannel> From<A> for ColorArg {
    fn from(v: A) -> Self {
        Self::Gray(v.into_channel(), None)
    }
}

impl<A: IntoChannel, B: IntoChannel> From<(A, B)> for ColorArg {
    fn from((v, a): (A, B)) -> Self {
        Self::Gray(v.into_channel(), Some(a.into_channel()))
    }
}

impl<A: IntoChannel, B: IntoChannel, C: IntoChannel> From<(A, B, C)> for ColorArg {
    fn from((c1, c2, c3): (A, B, C)) -> Self {
        Self::Channels(
            [c1.into_channel(), c2.into_channel(), c3.into_channel()],
            None,
        )
    }
}

impl<A: IntoChannel, B: IntoChannel, C: IntoChannel, D: IntoChannel> From<(A, B, C, D)>
    for ColorArg
{
    fn from((c1, c2, c3, a): (A, B, C, D)) -> Self {
        Self::Channels(
            [c1.into_channel(), c2.into_channel(), c3.into_channel()],
            Some(a.into_channel()),
        )
    }
}

/// Ranges for color channels: one number for all four, or `(c1, c2, c3)` / `(c1, c2, c3, alpha)`.
pub trait IntoColorMax {
    fn into_color_max(self, space: ColorSpace) -> [f32; 4];
}

impl<T: crate::Scalar> IntoColorMax for T {
    fn into_color_max(self, _space: ColorSpace) -> [f32; 4] {
        [self.to_f32(); 4]
    }
}

impl<A: crate::Scalar, B: crate::Scalar, C: crate::Scalar> IntoColorMax for (A, B, C) {
    fn into_color_max(self, space: ColorSpace) -> [f32; 4] {
        [
            self.0.to_f32(),
            self.1.to_f32(),
            self.2.to_f32(),
            space.default_maxes()[3],
        ]
    }
}

impl<A: crate::Scalar, B: crate::Scalar, C: crate::Scalar, D: crate::Scalar> IntoColorMax
    for (A, B, C, D)
{
    fn into_color_max(self, _space: ColorSpace) -> [f32; 4] {
        [
            self.0.to_f32(),
            self.1.to_f32(),
            self.2.to_f32(),
            self.3.to_f32(),
        ]
    }
}

/// The color mode as seen by a `Graphics`; `explicit_max` means `.max(..)` was set.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ColorState {
    pub mode: ColorMode,
    pub explicit_max: bool,
}

fn int_maxes(space: ColorSpace) -> [f32; 4] {
    match space {
        ColorSpace::Srgb | ColorSpace::Linear => [255.0, 255.0, 255.0, 255.0],
        ColorSpace::Hsl | ColorSpace::Hsv | ColorSpace::Hwb => [360.0, 100.0, 100.0, 255.0],
        ColorSpace::Oklch | ColorSpace::Lch => [100.0, 100.0, 360.0, 255.0],
        ColorSpace::Oklab | ColorSpace::Lab | ColorSpace::Xyz => [100.0, 100.0, 100.0, 255.0],
    }
}

impl ColorState {
    fn channel(&self, c: Channel, i: usize) -> f32 {
        let space = self.mode.space;
        let native = space.default_maxes()[i];
        match c {
            Channel::Int(v) if self.explicit_max => v as f32 / self.mode.max[i] * native,
            Channel::Float(v) if self.explicit_max => v / self.mode.max[i] * native,
            Channel::Int(v) => v as f32 / int_maxes(space)[i] * native,
            Channel::Float(v) => v,
        }
    }

    pub fn resolve(&self, arg: ColorArg) -> Color {
        let space = self.mode.space;
        let alpha = |a: Option<Channel>| match a {
            Some(a) => self.channel(a, 3),
            None => space.default_maxes()[3],
        };
        match arg {
            ColorArg::Color(c) => c,
            ColorArg::Gray(v, a) => Color(space.gray(self.channel(v, 0), alpha(a))),
            ColorArg::Channels([c1, c2, c3], a) => Color(space.color(
                self.channel(c1, 0),
                self.channel(c2, 1),
                self.channel(c3, 2),
                alpha(a),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn srgba(state: &ColorState, arg: impl Into<ColorArg>) -> [f32; 4] {
        let s = state.resolve(arg.into()).0.to_srgba();
        [s.red, s.green, s.blue, s.alpha]
    }

    #[test]
    fn ints_are_255_floats_are_unit() {
        let st = ColorState::default();
        assert_eq!(srgba(&st, 255), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(srgba(&st, 0.5), [0.5, 0.5, 0.5, 1.0]);
        assert_eq!(srgba(&st, (255, 0, 0, 0.5)), [1.0, 0.0, 0.0, 0.5]);
    }

    #[test]
    fn explicit_max_governs_ints_and_floats() {
        let st = ColorState {
            mode: ColorMode::with_uniform_max(ColorSpace::Srgb, 100.0),
            explicit_max: true,
        };
        assert_eq!(srgba(&st, (50, 100.0, 0)), [0.5, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn hex_round_trip() {
        let c = Color::hex(0xFF8000).0.to_srgba();
        assert_eq!([c.red, c.green, c.blue], [1.0, 128.0 / 255.0, 0.0]);
    }
}
