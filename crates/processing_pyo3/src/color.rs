use bevy::color::{
    Alpha, Color, Gray, Hsla, Hsva, Hue, Hwba, Laba, Lcha, LinearRgba, Luminance, Mix, Oklaba,
    Oklcha, Saturation, Srgba, Xyza, color_difference::EuclideanDistance,
};
use pyo3::{exceptions::PyTypeError, prelude::*, types::PyTuple};

use crate::math::{PyVec4, PyVecIter, hash_f32, PyVec3};

pub use processing::prelude::color::{ColorMode, ColorSpace};

fn int_maxes(space: &ColorSpace) -> [f32; 4] {
    match space {
        ColorSpace::Srgb | ColorSpace::Linear => [255.0, 255.0, 255.0, 255.0],
        ColorSpace::Hsl | ColorSpace::Hsv | ColorSpace::Hwb => [360.0, 100.0, 100.0, 255.0],
        ColorSpace::Oklch => [100.0, 100.0, 360.0, 255.0],
        ColorSpace::Oklab => [100.0, 100.0, 100.0, 255.0],
        ColorSpace::Lab => [100.0, 100.0, 100.0, 255.0],
        ColorSpace::Lch => [100.0, 100.0, 360.0, 255.0],
        ColorSpace::Xyz => [100.0, 100.0, 100.0, 255.0],
    }
}

/// Parse a Python int or float into an f32 for a given channel.
pub(crate) fn parse_numeric(space: &ColorSpace, obj: &Bound<'_, PyAny>, ch: usize) -> PyResult<f32> {
    if let Ok(v) = obj.extract::<i64>() {
        let native = space.default_maxes();
        let imax = int_maxes(space);
        return Ok(v as f32 / imax[ch] * native[ch]);
    }
    if let Ok(v) = obj.extract::<f64>() {
        return Ok(v as f32);
    }
    Err(PyTypeError::new_err("expected int or float"))
}

fn convert_channel(mode: &ColorMode, obj: &Bound<'_, PyAny>, ch: usize) -> PyResult<f32> {
    let v = parse_numeric(&mode.space, obj, ch)?;
    Ok(mode.scale(v, ch))
}

// Accepts a varags of color-like arguments and extracts a Color, applying the provided ColorMode.
pub(crate) fn extract_color_with_mode(
    args: &Bound<'_, PyTuple>,
    mode: &ColorMode,
) -> PyResult<Color> {
    let space = mode.space;
    let native = space.default_maxes();
    match args.len() {
        0 => Err(PyTypeError::new_err("expected at least 1 argument")),
        1 => {
            let first = args.get_item(0)?;
            if let Ok(c) = first.extract::<PyRef<PyColor>>() {
                return Ok(c.0);
            }
            if let Ok(s) = first.extract::<String>() {
                return parse_hex(&s);
            }
            if let Ok(v) = first.extract::<PyRef<PyVec4>>() {
                return Ok(space.color(v.0.x, v.0.y, v.0.z, v.0.w));
            }
            if let Ok(v) = first.extract::<PyRef<PyVec3>>() {
                return Ok(space.color(v.0.x, v.0.y, v.0.z, native[3]));
            }
            let v = convert_channel(mode,&first, 0)?;
            Ok(space.gray(v, native[3]))
        }
        2 => {
            let v = convert_channel(mode,&args.get_item(0)?, 0)?;
            let a = convert_channel(mode,&args.get_item(1)?, 3)?;
            Ok(space.gray(v, a))
        }
        3 => {
            let c1 = convert_channel(mode,&args.get_item(0)?, 0)?;
            let c2 = convert_channel(mode,&args.get_item(1)?, 1)?;
            let c3 = convert_channel(mode,&args.get_item(2)?, 2)?;
            Ok(space.color(c1, c2, c3, native[3]))
        }
        4 => {
            let c1 = convert_channel(mode,&args.get_item(0)?, 0)?;
            let c2 = convert_channel(mode,&args.get_item(1)?, 1)?;
            let c3 = convert_channel(mode,&args.get_item(2)?, 2)?;
            let ca = convert_channel(mode,&args.get_item(3)?, 3)?;
            Ok(space.color(c1, c2, c3, ca))
        }
        _ => Err(PyTypeError::new_err("expected 1-4 arguments")),
    }
}

#[pyclass(name = "Color", from_py_object)]
#[derive(Clone, Debug)]
pub struct PyColor(pub(crate) Color);

impl From<Color> for PyColor {
    fn from(c: Color) -> Self {
        Self(c)
    }
}

impl From<PyColor> for Color {
    fn from(c: PyColor) -> Self {
        c.0
    }
}

impl From<Srgba> for PyColor {
    fn from(c: Srgba) -> Self {
        Self(Color::Srgba(c))
    }
}

fn components(color: &Color) -> [f32; 4] {
    use bevy::color::ColorToComponents;
    match *color {
        Color::Srgba(c) => c.to_f32_array(),
        Color::LinearRgba(c) => c.to_f32_array(),
        Color::Hsla(c) => c.to_f32_array(),
        Color::Hsva(c) => c.to_f32_array(),
        Color::Hwba(c) => c.to_f32_array(),
        Color::Laba(c) => c.to_f32_array(),
        Color::Lcha(c) => c.to_f32_array(),
        Color::Oklaba(c) => c.to_f32_array(),
        Color::Oklcha(c) => c.to_f32_array(),
        Color::Xyza(c) => c.to_f32_array(),
    }
}

fn components_no_alpha(color: &Color) -> [f32; 3] {
    use bevy::color::ColorToComponents;
    match *color {
        Color::Srgba(c) => c.to_f32_array_no_alpha(),
        Color::LinearRgba(c) => c.to_f32_array_no_alpha(),
        Color::Hsla(c) => c.to_f32_array_no_alpha(),
        Color::Hsva(c) => c.to_f32_array_no_alpha(),
        Color::Hwba(c) => c.to_f32_array_no_alpha(),
        Color::Laba(c) => c.to_f32_array_no_alpha(),
        Color::Lcha(c) => c.to_f32_array_no_alpha(),
        Color::Oklaba(c) => c.to_f32_array_no_alpha(),
        Color::Oklcha(c) => c.to_f32_array_no_alpha(),
        Color::Xyza(c) => c.to_f32_array_no_alpha(),
    }
}

#[pymethods]
impl PyColor {
    // Varargs ctor for positional calls like color(255, 0, 0). ColorLike handles single-value extraction.
    #[new]
    #[pyo3(signature = (*args))]
    pub fn py_new(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        extract_color_with_mode(args, &ColorMode::default()).map(Self)
    }

    #[staticmethod]
    #[pyo3(signature = (r, g, b, a=1.0))]
    pub fn srgb(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(Color::Srgba(Srgba::new(r, g, b, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (r, g, b, a=1.0))]
    pub fn linear(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(Color::LinearRgba(LinearRgba::new(r, g, b, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (h, s, l, a=1.0))]
    pub fn hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
        Self(Color::Hsla(Hsla::new(h, s, l, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (h, s, v, a=1.0))]
    pub fn hsva(h: f32, s: f32, v: f32, a: f32) -> Self {
        Self(Color::Hsva(Hsva::new(h, s, v, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (h, w, b, a=1.0))]
    pub fn hwba(h: f32, w: f32, b: f32, a: f32) -> Self {
        Self(Color::Hwba(Hwba::new(h, w, b, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (l, a_axis, b_axis, alpha=1.0))]
    pub fn oklab(l: f32, a_axis: f32, b_axis: f32, alpha: f32) -> Self {
        Self(Color::Oklaba(Oklaba::new(l, a_axis, b_axis, alpha)))
    }

    #[staticmethod]
    #[pyo3(signature = (l, c, h, a=1.0))]
    pub fn oklch(l: f32, c: f32, h: f32, a: f32) -> Self {
        Self(Color::Oklcha(Oklcha::new(l, c, h, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (l, a_axis, b_axis, alpha=1.0))]
    pub fn lab(l: f32, a_axis: f32, b_axis: f32, alpha: f32) -> Self {
        Self(Color::Laba(Laba::new(l, a_axis, b_axis, alpha)))
    }

    #[staticmethod]
    #[pyo3(signature = (l, c, h, a=1.0))]
    pub fn lch(l: f32, c: f32, h: f32, a: f32) -> Self {
        Self(Color::Lcha(Lcha::new(l, c, h, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (x, y, z, a=1.0))]
    pub fn xyz(x: f32, y: f32, z: f32, a: f32) -> Self {
        Self(Color::Xyza(Xyza::new(x, y, z, a)))
    }

    #[staticmethod]
    pub fn hex(s: &str) -> PyResult<Self> {
        parse_hex(s).map(Self)
    }

    fn to_srgba(&self) -> Self {
        Self(Color::Srgba(self.0.into()))
    }

    fn to_linear(&self) -> Self {
        Self(Color::LinearRgba(self.0.into()))
    }

    fn to_hsla(&self) -> Self {
        Self(Color::Hsla(self.0.into()))
    }

    fn to_hsva(&self) -> Self {
        Self(Color::Hsva(self.0.into()))
    }

    fn to_hwba(&self) -> Self {
        Self(Color::Hwba(self.0.into()))
    }

    fn to_oklab(&self) -> Self {
        Self(Color::Oklaba(self.0.into()))
    }

    fn to_oklch(&self) -> Self {
        Self(Color::Oklcha(self.0.into()))
    }

    fn to_lab(&self) -> Self {
        Self(Color::Laba(self.0.into()))
    }

    fn to_lch(&self) -> Self {
        Self(Color::Lcha(self.0.into()))
    }

    fn to_xyz(&self) -> Self {
        Self(Color::Xyza(self.0.into()))
    }

    #[getter]
    fn r(&self) -> f32 {
        self.0.to_srgba().red
    }
    #[setter]
    fn set_r(&mut self, val: f32) {
        let mut s = self.0.to_srgba();
        s.red = val;
        self.0 = Color::Srgba(s);
    }

    #[getter]
    fn g(&self) -> f32 {
        self.0.to_srgba().green
    }
    #[setter]
    fn set_g(&mut self, val: f32) {
        let mut s = self.0.to_srgba();
        s.green = val;
        self.0 = Color::Srgba(s);
    }

    #[getter]
    fn b(&self) -> f32 {
        self.0.to_srgba().blue
    }
    #[setter]
    fn set_b(&mut self, val: f32) {
        let mut s = self.0.to_srgba();
        s.blue = val;
        self.0 = Color::Srgba(s);
    }

    #[getter]
    fn a(&self) -> f32 {
        self.0.alpha()
    }
    #[setter]
    fn set_a(&mut self, val: f32) {
        self.0.set_alpha(val);
    }

    fn to_hex(&self) -> String {
        self.0.to_srgba().to_hex()
    }

    fn with_alpha(&self, a: f32) -> Self {
        Self(self.0.with_alpha(a))
    }

    fn mix(&self, other: &Self, t: f32) -> Self {
        Self(self.0.mix(&other.0, t))
    }

    fn lerp(&self, other: &Self, t: f32) -> Self {
        self.mix(other, t)
    }

    fn lighter(&self, amount: f32) -> Self {
        Self(self.0.lighter(amount))
    }

    fn darker(&self, amount: f32) -> Self {
        Self(self.0.darker(amount))
    }

    fn luminance(&self) -> f32 {
        self.0.luminance()
    }

    fn with_luminance(&self, value: f32) -> Self {
        Self(self.0.with_luminance(value))
    }

    fn hue(&self) -> f32 {
        self.0.hue()
    }

    fn with_hue(&self, hue: f32) -> Self {
        Self(self.0.with_hue(hue))
    }

    fn rotate_hue(&self, degrees: f32) -> Self {
        Self(self.0.rotate_hue(degrees))
    }

    fn saturation(&self) -> f32 {
        self.0.saturation()
    }

    fn with_saturation(&self, saturation: f32) -> Self {
        Self(self.0.with_saturation(saturation))
    }

    fn is_fully_transparent(&self) -> bool {
        self.0.is_fully_transparent()
    }

    fn is_fully_opaque(&self) -> bool {
        self.0.is_fully_opaque()
    }

    fn distance(&self, other: &Self) -> f32 {
        self.0.distance(&other.0)
    }

    fn distance_squared(&self, other: &Self) -> f32 {
        self.0.distance_squared(&other.0)
    }

    #[staticmethod]
    fn gray(lightness: f32) -> Self {
        Self(Color::Srgba(Srgba::gray(lightness)))
    }

    fn to_vec3(&self) -> crate::math::PyVec3 {
        let c = components_no_alpha(&self.0);
        crate::math::PyVec3(bevy::math::Vec3::from_array(c))
    }

    fn to_vec4(&self) -> PyVec4 {
        let c = components(&self.0);
        PyVec4(bevy::math::Vec4::from_array(c))
    }

    fn to_list(&self) -> Vec<f32> {
        components(&self.0).to_vec()
    }

    fn to_tuple<'py>(&self, py: Python<'py>) -> Bound<'py, PyTuple> {
        PyTuple::new(py, components(&self.0)).unwrap()
    }

    fn __repr__(&self) -> String {
        let c = components(&self.0);
        format!("Color({}, {}, {}, {})", c[0], c[1], c[2], c[3])
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn __eq__(&self, other: &Self) -> bool {
        // Compare in sRGBA so colors in different spaces can be equal
        self.0.to_srgba() == other.0.to_srgba()
    }

    fn __hash__(&self) -> u64 {
        // Hash in sRGBA so equal colors hash the same regardless of space
        let s = self.0.to_srgba();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hash_f32(s.red, &mut hasher);
        hash_f32(s.green, &mut hasher);
        hash_f32(s.blue, &mut hasher);
        hash_f32(s.alpha, &mut hasher);
        std::hash::Hasher::finish(&hasher)
    }

    fn __len__(&self) -> usize {
        4
    }

    fn __getitem__(&self, idx: isize) -> PyResult<f32> {
        let c = components(&self.0);
        let idx = if idx < 0 { 4 + idx } else { idx };

        if !(0..4).contains(&idx) {
            return Err(PyTypeError::new_err("index out of range"));
        }
        Ok(c[idx as usize])
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyVecIter {
        PyVecIter {
            values: components(&slf.0).to_vec(),
            index: 0,
        }
    }
}

#[derive(FromPyObject)]
pub enum ColorLike {
    Instance(PyColor),
    HexString(String),
    Vec4(PyVec4),
    Tuple4((f32, f32, f32, f32)),
    Tuple3((f32, f32, f32)),
}

impl ColorLike {
    pub fn into_color(self) -> PyResult<Color> {
        match self {
            ColorLike::Instance(c) => Ok(c.0),
            ColorLike::HexString(s) => parse_hex(&s),
            ColorLike::Vec4(v) => Ok(Color::srgba(v.0.x, v.0.y, v.0.z, v.0.w)),
            ColorLike::Tuple4((r, g, b, a)) => Ok(Color::srgba(r, g, b, a)),
            ColorLike::Tuple3((r, g, b)) => Ok(Color::srgba(r, g, b, 1.0)),
        }
    }
}


fn parse_hex(s: &str) -> PyResult<Color> {
    Srgba::hex(s)
        .map(Color::Srgba)
        .map_err(|e| PyTypeError::new_err(format!("invalid hex color: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_srgba() {
        let c = PyColor(Color::srgba(1.0, 0.0, 0.5, 1.0));
        let s = c.0.to_srgba();
        assert!((s.red - 1.0).abs() < 1e-6);
        assert!((s.green - 0.0).abs() < 1e-6);
        assert!((s.blue - 0.5).abs() < 1e-6);
        assert!((s.alpha - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_hex_roundtrip() {
        let c = parse_hex("#FF00FF").unwrap();
        let s = c.to_srgba();
        assert!((s.red - 1.0).abs() < 0.01);
        assert!((s.green - 0.0).abs() < 0.01);
        assert!((s.blue - 1.0).abs() < 0.01);
        assert!((s.alpha - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hex_with_alpha() {
        let c = parse_hex("#FF000080").unwrap();
        let s = c.to_srgba();
        assert!((s.red - 1.0).abs() < 0.01);
        assert!((s.alpha - 128.0 / 255.0).abs() < 0.01);
    }

    #[test]
    fn test_color_mix() {
        let a = PyColor(Color::srgba(0.0, 0.0, 0.0, 1.0));
        let b = PyColor(Color::srgba(1.0, 1.0, 1.0, 1.0));
        let mid = a.mix(&b, 0.5);
        let s = mid.0.to_srgba();
        assert!((s.red - 0.5).abs() < 0.05);
        assert!((s.green - 0.5).abs() < 0.05);
        assert!((s.blue - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_color_lighter_darker() {
        let c = PyColor(Color::srgba(0.5, 0.5, 0.5, 1.0));
        let lighter = c.lighter(0.1);
        let darker = c.darker(0.1);
        let sl = lighter.0.to_srgba();
        let sd = darker.0.to_srgba();
        assert!(sl.red > sd.red || sl.green > sd.green || sl.blue > sd.blue);
    }

    #[test]
    fn test_color_with_alpha() {
        let c = PyColor(Color::srgba(1.0, 0.0, 0.0, 1.0));
        let transparent = c.with_alpha(0.5);
        assert!((transparent.0.alpha() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_color_to_vec4() {
        let c = PyColor(Color::srgba(0.25, 0.5, 0.75, 1.0));
        let v = c.to_vec4();
        assert!((v.0.x - 0.25).abs() < 1e-6);
        assert!((v.0.y - 0.5).abs() < 1e-6);
        assert!((v.0.z - 0.75).abs() < 1e-6);
        assert!((v.0.w - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_color_eq() {
        let a = PyColor(Color::srgba(1.0, 0.0, 0.0, 1.0));
        let b = PyColor(Color::srgba(1.0, 0.0, 0.0, 1.0));
        assert!(a.__eq__(&b));
    }

    #[test]
    fn test_hsla_roundtrip() {
        let c = PyColor::hsla(0.0, 1.0, 0.5, 1.0);
        let s = c.0.to_srgba();
        assert!((s.red - 1.0).abs() < 0.01);
        assert!(s.green < 0.01);
        assert!(s.blue < 0.01);

        let [h, sat, l, a] = components(&c.to_hsla().0);
        assert!((h - 0.0).abs() < 0.5);
        assert!((sat - 1.0).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
        assert!((a - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_oklch_roundtrip() {
        let c = PyColor::oklch(0.7, 0.15, 30.0, 1.0);
        let [l, ch, h, a] = components(&c.to_oklch().0);
        assert!((l - 0.7).abs() < 0.01);
        assert!((ch - 0.15).abs() < 0.01);
        assert!((h - 30.0).abs() < 0.5);
        assert!((a - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_linear_roundtrip() {
        let c = PyColor::linear(0.5, 0.25, 0.1, 0.8);
        let [r, g, b, a] = components(&c.to_linear().0);
        assert!((r - 0.5).abs() < 0.01);
        assert!((g - 0.25).abs() < 0.01);
        assert!((b - 0.1).abs() < 0.01);
        assert!((a - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_to_list() {
        let c = PyColor(Color::srgba(0.1, 0.2, 0.3, 0.4));
        let list = c.to_list();
        assert_eq!(list.len(), 4);
        assert!((list[0] - 0.1).abs() < 1e-6);
        assert!((list[3] - 0.4).abs() < 1e-6);
    }

}
