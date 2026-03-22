use bevy::color::{
    Alpha, Color, Gray, Hsla, Hsva, Hue, Hwba, Laba, Lcha, LinearRgba, Luminance, Mix, Oklaba,
    Oklcha, Saturation, Srgba, Xyza, color_difference::EuclideanDistance,
};
use pyo3::{exceptions::PyTypeError, prelude::*, types::PyTuple};

use crate::math::{PyVec4, PyVecIter, hash_f32};

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

fn extract_component(obj: &Bound<'_, PyAny>) -> PyResult<f32> {
    if let Ok(v) = obj.extract::<i64>() {
        return Ok(v as f32 / 255.0);
    }
    if let Ok(v) = obj.extract::<f64>() {
        return Ok(v as f32);
    }
    Err(PyTypeError::new_err("expected int or float"))
}

fn to_srgba(color: &Color) -> Srgba {
    color.to_srgba()
}

#[pymethods]
impl PyColor {
    #[new]
    #[pyo3(signature = (*args))]
    pub fn py_new(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        match args.len() {
            0 => Err(PyTypeError::new_err("Color requires at least 1 argument")),
            1 => {
                let first = args.get_item(0)?;
                if let Ok(c) = first.extract::<PyRef<PyColor>>() {
                    return Ok(Self(c.0));
                }
                if let Ok(s) = first.extract::<String>() {
                    return Ok(Self(parse_hex(&s)?));
                }
                if let Ok(v) = first.extract::<PyRef<PyVec4>>() {
                    return Ok(Self(Color::srgba(v.0.x, v.0.y, v.0.z, v.0.w)));
                }
                let v = extract_component(&first)?;
                Ok(Self(Color::srgba(v, v, v, 1.0)))
            }
            2 => {
                let v = extract_component(&args.get_item(0)?)?;
                let a = extract_component(&args.get_item(1)?)?;
                Ok(Self(Color::srgba(v, v, v, a)))
            }
            3 => {
                let r = extract_component(&args.get_item(0)?)?;
                let g = extract_component(&args.get_item(1)?)?;
                let b = extract_component(&args.get_item(2)?)?;
                Ok(Self(Color::srgba(r, g, b, 1.0)))
            }
            4 => {
                let r = extract_component(&args.get_item(0)?)?;
                let g = extract_component(&args.get_item(1)?)?;
                let b = extract_component(&args.get_item(2)?)?;
                let a = extract_component(&args.get_item(3)?)?;
                Ok(Self(Color::srgba(r, g, b, a)))
            }
            _ => Err(PyTypeError::new_err("Color takes 1-4 arguments")),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (r, g, b, a=1.0))]
    pub fn from_srgb(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(Color::Srgba(Srgba::new(r, g, b, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (r, g, b, a=1.0))]
    pub fn from_linear(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(Color::LinearRgba(LinearRgba::new(r, g, b, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (h, s, l, a=1.0))]
    pub fn from_hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
        Self(Color::Hsla(Hsla::new(h, s, l, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (h, s, v, a=1.0))]
    pub fn from_hsva(h: f32, s: f32, v: f32, a: f32) -> Self {
        Self(Color::Hsva(Hsva::new(h, s, v, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (h, w, b, a=1.0))]
    pub fn from_hwba(h: f32, w: f32, b: f32, a: f32) -> Self {
        Self(Color::Hwba(Hwba::new(h, w, b, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (l, a_axis, b_axis, alpha=1.0))]
    pub fn from_oklab(l: f32, a_axis: f32, b_axis: f32, alpha: f32) -> Self {
        Self(Color::Oklaba(Oklaba::new(l, a_axis, b_axis, alpha)))
    }

    #[staticmethod]
    #[pyo3(signature = (l, c, h, a=1.0))]
    pub fn from_oklch(l: f32, c: f32, h: f32, a: f32) -> Self {
        Self(Color::Oklcha(Oklcha::new(l, c, h, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (l, a_axis, b_axis, alpha=1.0))]
    pub fn from_lab(l: f32, a_axis: f32, b_axis: f32, alpha: f32) -> Self {
        Self(Color::Laba(Laba::new(l, a_axis, b_axis, alpha)))
    }

    #[staticmethod]
    #[pyo3(signature = (l, c, h, a=1.0))]
    pub fn from_lch(l: f32, c: f32, h: f32, a: f32) -> Self {
        Self(Color::Lcha(Lcha::new(l, c, h, a)))
    }

    #[staticmethod]
    #[pyo3(signature = (x, y, z, a=1.0))]
    pub fn from_xyz(x: f32, y: f32, z: f32, a: f32) -> Self {
        Self(Color::Xyza(Xyza::new(x, y, z, a)))
    }

    #[staticmethod]
    pub fn from_hex(s: &str) -> PyResult<Self> {
        parse_hex(s).map(Self)
    }

    fn to_srgba(&self) -> (f32, f32, f32, f32) {
        let s: Srgba = self.0.into();
        (s.red, s.green, s.blue, s.alpha)
    }

    fn to_linear(&self) -> (f32, f32, f32, f32) {
        let l: LinearRgba = self.0.into();
        (l.red, l.green, l.blue, l.alpha)
    }

    fn to_hsla(&self) -> (f32, f32, f32, f32) {
        let h: Hsla = self.0.into();
        (h.hue, h.saturation, h.lightness, h.alpha)
    }

    fn to_hsva(&self) -> (f32, f32, f32, f32) {
        let h: Hsva = self.0.into();
        (h.hue, h.saturation, h.value, h.alpha)
    }

    fn to_hwba(&self) -> (f32, f32, f32, f32) {
        let h: Hwba = self.0.into();
        (h.hue, h.whiteness, h.blackness, h.alpha)
    }

    fn to_oklab(&self) -> (f32, f32, f32, f32) {
        let o: Oklaba = self.0.into();
        (o.lightness, o.a, o.b, o.alpha)
    }

    fn to_oklch(&self) -> (f32, f32, f32, f32) {
        let o: Oklcha = self.0.into();
        (o.lightness, o.chroma, o.hue, o.alpha)
    }

    fn to_lab(&self) -> (f32, f32, f32, f32) {
        let l: Laba = self.0.into();
        (l.lightness, l.a, l.b, l.alpha)
    }

    fn to_lch(&self) -> (f32, f32, f32, f32) {
        let l: Lcha = self.0.into();
        (l.lightness, l.chroma, l.hue, l.alpha)
    }

    fn to_xyz(&self) -> (f32, f32, f32, f32) {
        let x: Xyza = self.0.into();
        (x.x, x.y, x.z, x.alpha)
    }

    #[getter]
    fn r(&self) -> f32 {
        to_srgba(&self.0).red
    }
    #[setter]
    fn set_r(&mut self, val: f32) {
        let mut s = to_srgba(&self.0);
        s.red = val;
        self.0 = Color::Srgba(s);
    }

    #[getter]
    fn g(&self) -> f32 {
        to_srgba(&self.0).green
    }
    #[setter]
    fn set_g(&mut self, val: f32) {
        let mut s = to_srgba(&self.0);
        s.green = val;
        self.0 = Color::Srgba(s);
    }

    #[getter]
    fn b(&self) -> f32 {
        to_srgba(&self.0).blue
    }
    #[setter]
    fn set_b(&mut self, val: f32) {
        let mut s = to_srgba(&self.0);
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

    fn hex(&self) -> String {
        to_srgba(&self.0).to_hex()
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
        let s = to_srgba(&self.0);
        crate::math::PyVec3(bevy::math::Vec3::new(s.red, s.green, s.blue))
    }

    fn to_vec4(&self) -> PyVec4 {
        let s = to_srgba(&self.0);
        PyVec4(bevy::math::Vec4::new(s.red, s.green, s.blue, s.alpha))
    }

    fn to_list(&self) -> Vec<f32> {
        let s = to_srgba(&self.0);
        vec![s.red, s.green, s.blue, s.alpha]
    }

    fn to_tuple<'py>(&self, py: Python<'py>) -> Bound<'py, PyTuple> {
        let s = to_srgba(&self.0);
        PyTuple::new(py, [s.red, s.green, s.blue, s.alpha]).unwrap()
    }

    fn __repr__(&self) -> String {
        let s = to_srgba(&self.0);
        format!("Color({}, {}, {}, {})", s.red, s.green, s.blue, s.alpha)
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn __eq__(&self, other: &Self) -> bool {
        to_srgba(&self.0) == to_srgba(&other.0)
    }

    fn __hash__(&self) -> u64 {
        let s = to_srgba(&self.0);
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
        let s = to_srgba(&self.0);
        let idx = if idx < 0 { 4 + idx } else { idx };
        match idx {
            0 => Ok(s.red),
            1 => Ok(s.green),
            2 => Ok(s.blue),
            3 => Ok(s.alpha),
            _ => Err(PyTypeError::new_err("index out of range")),
        }
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyVecIter {
        let s = to_srgba(&slf.0);
        PyVecIter {
            values: vec![s.red, s.green, s.blue, s.alpha],
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

pub(crate) fn extract_color(args: &Bound<'_, PyTuple>) -> PyResult<Color> {
    PyColor::py_new(args).map(|c| c.0)
}

fn parse_hex(s: &str) -> PyResult<Color> {
    Srgba::hex(s)
        .map(|srgba| Color::Srgba(srgba))
        .map_err(|e| PyTypeError::new_err(format!("invalid hex color: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_srgba() {
        let c = PyColor(Color::srgba(1.0, 0.0, 0.5, 1.0));
        let s = to_srgba(&c.0);
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
        let s = to_srgba(&mid.0);
        assert!((s.red - 0.5).abs() < 0.05);
        assert!((s.green - 0.5).abs() < 0.05);
        assert!((s.blue - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_color_lighter_darker() {
        let c = PyColor(Color::srgba(0.5, 0.5, 0.5, 1.0));
        let lighter = c.lighter(0.1);
        let darker = c.darker(0.1);
        let sl = to_srgba(&lighter.0);
        let sd = to_srgba(&darker.0);
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
        let c = PyColor::from_hsla(0.0, 1.0, 0.5, 1.0);
        let s = to_srgba(&c.0);
        assert!((s.red - 1.0).abs() < 0.01);
        assert!(s.green < 0.01);
        assert!(s.blue < 0.01);

        let (h, sat, l, a) = c.to_hsla();
        assert!((h - 0.0).abs() < 0.5);
        assert!((sat - 1.0).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
        assert!((a - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_oklch_roundtrip() {
        let c = PyColor::from_oklch(0.7, 0.15, 30.0, 1.0);
        let (l, ch, h, a) = c.to_oklch();
        assert!((l - 0.7).abs() < 0.01);
        assert!((ch - 0.15).abs() < 0.01);
        assert!((h - 30.0).abs() < 0.5);
        assert!((a - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_linear_roundtrip() {
        let c = PyColor::from_linear(0.5, 0.25, 0.1, 0.8);
        let (r, g, b, a) = c.to_linear();
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
