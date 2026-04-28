use std::hash::{Hash, Hasher};

use bevy::math::{EulerRot, Quat, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};
use pyo3::{
    exceptions::{PyAttributeError, PyTypeError},
    prelude::*,
    types::PyTuple,
};

pub fn hash_f32(val: f32, state: &mut impl Hasher) {
    if val == 0.0 {
        0.0f32.to_bits().hash(state);
    } else {
        val.to_bits().hash(state);
    }
}

#[derive(FromPyObject)]
pub(crate) enum Vec2Arg {
    Instance((PyVec2,)),
    Components(f32, f32),
}

impl Vec2Arg {
    pub fn into_vec2(self) -> Vec2 {
        match self {
            Vec2Arg::Instance((v,)) => v.0,
            Vec2Arg::Components(x, y) => Vec2::new(x, y),
        }
    }
}

#[derive(FromPyObject)]
pub(crate) enum Vec3Arg {
    Instance((PyVec3,)),
    Components(f32, f32, f32),
}

impl Vec3Arg {
    pub fn into_vec3(self) -> Vec3 {
        match self {
            Vec3Arg::Instance((v,)) => v.0,
            Vec3Arg::Components(x, y, z) => Vec3::new(x, y, z),
        }
    }
}

#[derive(FromPyObject)]
pub(crate) enum Vec4Arg {
    Instance((PyVec4,)),
    Components(f32, f32, f32, f32),
}

impl Vec4Arg {
    pub fn into_vec4(self) -> Vec4 {
        match self {
            Vec4Arg::Instance((v,)) => v.0,
            Vec4Arg::Components(x, y, z, w) => Vec4::new(x, y, z, w),
        }
    }
}

// Vec3Like for single-object extraction (e.g., Quat::from_axis_angle(axis, angle))
#[derive(FromPyObject)]
pub(crate) enum Vec3Like {
    Instance(PyVec3),
    Tuple((f32, f32, f32)),
    List([f32; 3]),
}

impl Vec3Like {
    pub fn into_vec3(self) -> Vec3 {
        match self {
            Vec3Like::Instance(v) => v.0,
            Vec3Like::Tuple((x, y, z)) => Vec3::new(x, y, z),
            Vec3Like::List(arr) => Vec3::from_array(arr),
        }
    }
}

pub(crate) fn extract_vec2(args: &Bound<'_, PyTuple>) -> PyResult<Vec2> {
    Ok(args.extract::<Vec2Arg>()?.into_vec2())
}

pub(crate) fn extract_vec3(args: &Bound<'_, PyTuple>) -> PyResult<Vec3> {
    Ok(args.extract::<Vec3Arg>()?.into_vec3())
}

pub(crate) fn extract_vec4(args: &Bound<'_, PyTuple>) -> PyResult<Vec4> {
    Ok(args.extract::<Vec4Arg>()?.into_vec4())
}

// Implements a PyVecN class with the given name, fields, and underlying glam type,
// including arithmetic operations, indexing, iteration, and common vector methods, etc.
// The `extra` block can be used to add additional methods specific to certain vector types
// (e.g., angle/rotate for Vec2, cross for Vec3).
macro_rules! impl_py_vec {
    (
        $name:ident, $py_name:literal, $n:literal,
        [$(($field:ident, $set_field:ident, $idx:literal)),+],
        $glam_ty:ty
        $(, extra { $($extra:tt)* })?
    ) => {
        #[pyclass(name = $py_name, from_py_object)]
        #[derive(Clone, Debug)]
        pub struct $name(pub(crate) $glam_ty);

        impl From<$glam_ty> for $name {
            fn from(v: $glam_ty) -> Self { Self(v) }
        }

        impl From<$name> for $glam_ty {
            fn from(v: $name) -> Self { v.0 }
        }

        #[pymethods]
        impl $name {
            #[new]
            #[pyo3(signature = (*args))]
            pub fn py_new(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
                match args.len() {
                    0 => Ok(Self(<$glam_ty>::ZERO)),
                    1 => {
                        let first = args.get_item(0)?;
                        if let Ok(s) = first.extract::<f32>() {
                            return Ok(Self(<$glam_ty>::splat(s)));
                        }
                        if let Ok(s) = first.extract::<i64>() {
                            return Ok(Self(<$glam_ty>::splat(s as f32)));
                        }
                        if let Ok(v) = first.extract::<PyRef<$name>>() {
                            return Ok(Self(v.0));
                        }
                        if let Ok(arr) = first.extract::<[f32; $n]>() {
                            return Ok(Self(<$glam_ty>::from_array(arr)));
                        }
                        Err(PyTypeError::new_err(concat!(
                            "expected scalar, ", $py_name, ", or sequence of ",
                            stringify!($n), " floats"
                        )))
                    }
                    $n => {
                        let mut arr = [0.0f32; $n];
                        $(arr[$idx] = args.get_item($idx)?.extract::<f32>()?;)+
                        Ok(Self(<$glam_ty>::from_array(arr)))
                    }
                    _ => Err(PyTypeError::new_err(concat!(
                        $py_name, " takes 0, 1, or ", stringify!($n), " arguments"
                    ))),
                }
            }

            $(
                #[getter]
                fn $field(&self) -> f32 { self.0[$idx] }

                #[setter]
                fn $set_field(&mut self, val: f32) { self.0[$idx] = val; }
            )+

            fn __add__(&self, other: &Self) -> Self { Self(self.0 + other.0) }
            fn __radd__(&self, other: &Self) -> Self { Self(other.0 + self.0) }
            fn __iadd__(&mut self, other: &Self) { self.0 += other.0; }

            fn __sub__(&self, other: &Self) -> Self { Self(self.0 - other.0) }
            fn __rsub__(&self, other: &Self) -> Self { Self(other.0 - self.0) }
            fn __isub__(&mut self, other: &Self) { self.0 -= other.0; }

            fn __neg__(&self) -> Self { Self(-self.0) }

            fn __mul__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<Self> {
                if let Ok(s) = rhs.extract::<f32>() {
                    return Ok(Self(self.0 * s));
                }
                if let Ok(s) = rhs.extract::<i64>() {
                    return Ok(Self(self.0 * s as f32));
                }
                if let Ok(other) = rhs.extract::<PyRef<$name>>() {
                    return Ok(Self(self.0 * other.0));
                }
                Err(PyTypeError::new_err(concat!(
                    "unsupported operand type(s) for *: '", $py_name, "'"
                )))
            }

            fn __rmul__(&self, lhs: &Bound<'_, PyAny>) -> PyResult<Self> {
                if let Ok(s) = lhs.extract::<f32>() {
                    return Ok(Self(self.0 * s));
                }
                if let Ok(s) = lhs.extract::<i64>() {
                    return Ok(Self(self.0 * s as f32));
                }
                Err(PyTypeError::new_err(concat!(
                    "unsupported operand type(s) for *: '", $py_name, "'"
                )))
            }

            fn __imul__(&mut self, rhs: &Bound<'_, PyAny>) -> PyResult<()> {
                if let Ok(s) = rhs.extract::<f32>() {
                    self.0 *= s; return Ok(());
                }
                if let Ok(s) = rhs.extract::<i64>() {
                    self.0 *= s as f32; return Ok(());
                }
                if let Ok(other) = rhs.extract::<PyRef<$name>>() {
                    self.0 *= other.0; return Ok(());
                }
                Err(PyTypeError::new_err(concat!(
                    "unsupported operand type(s) for *=: '", $py_name, "'"
                )))
            }

            fn __truediv__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<Self> {
                if let Ok(s) = rhs.extract::<f32>() {
                    return Ok(Self(self.0 / s));
                }
                if let Ok(s) = rhs.extract::<i64>() {
                    return Ok(Self(self.0 / s as f32));
                }
                if let Ok(other) = rhs.extract::<PyRef<$name>>() {
                    return Ok(Self(self.0 / other.0));
                }
                Err(PyTypeError::new_err(concat!(
                    "unsupported operand type(s) for /: '", $py_name, "'"
                )))
            }

            fn __rtruediv__(&self, lhs: &Bound<'_, PyAny>) -> PyResult<Self> {
                if let Ok(s) = lhs.extract::<f32>() {
                    return Ok(Self(<$glam_ty>::splat(s) / self.0));
                }
                if let Ok(s) = lhs.extract::<i64>() {
                    return Ok(Self(<$glam_ty>::splat(s as f32) / self.0));
                }
                Err(PyTypeError::new_err(concat!(
                    "unsupported operand type(s) for /: '", $py_name, "'"
                )))
            }

            fn __itruediv__(&mut self, rhs: &Bound<'_, PyAny>) -> PyResult<()> {
                if let Ok(s) = rhs.extract::<f32>() {
                    self.0 /= s; return Ok(());
                }
                if let Ok(s) = rhs.extract::<i64>() {
                    self.0 /= s as f32; return Ok(());
                }
                if let Ok(other) = rhs.extract::<PyRef<$name>>() {
                    self.0 /= other.0; return Ok(());
                }
                Err(PyTypeError::new_err(concat!(
                    "unsupported operand type(s) for /=: '", $py_name, "'"
                )))
            }

            fn __eq__(&self, other: &Self) -> bool { self.0 == other.0 }

            fn __hash__(&self) -> u64 {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                for &c in self.0.to_array().iter() {
                    hash_f32(c, &mut hasher);
                }
                std::hash::Hasher::finish(&hasher)
            }

            fn __repr__(&self) -> String {
                let parts: Vec<String> = self.0.to_array().iter().map(|c| format!("{c}")).collect();
                format!("{}({})", $py_name, parts.join(", "))
            }

            fn __str__(&self) -> String { self.__repr__() }

            fn __len__(&self) -> usize { $n }

            fn __getitem__(&self, idx: isize) -> PyResult<f32> {
                let idx = if idx < 0 { $n as isize + idx } else { idx };
                if !(0..$n as isize).contains(&idx) {
                    return Err(PyTypeError::new_err("index out of range"));
                }
                Ok(self.0[idx as usize])
            }

            fn __setitem__(&mut self, idx: isize, val: f32) -> PyResult<()> {
                let idx = if idx < 0 { $n as isize + idx } else { idx };
                if !(0..$n as isize).contains(&idx) {
                    return Err(PyTypeError::new_err("index out of range"));
                }
                self.0[idx as usize] = val;
                Ok(())
            }

            fn __iter__(slf: PyRef<'_, Self>) -> PyVecIter {
                PyVecIter {
                    values: slf.0.to_array().to_vec(),
                    index: 0,
                }
            }

            fn __getattr__<'py>(
                slf: PyRef<'py, Self>,
                name: &str,
            ) -> PyResult<Bound<'py, PyAny>> {
                let py = slf.py();
                let chars: Vec<char> = name.chars().collect();
                let len = chars.len();
                let attr_err = || {
                    PyAttributeError::new_err(format!(
                        "'{}' object has no attribute '{}'",
                        $py_name, name
                    ))
                };
                if !(2..=4).contains(&len) {
                    return Err(attr_err());
                }
                let mut vals = [0.0f32; 4];
                for (i, c) in chars.iter().enumerate() {
                    let idx: usize = match c {
                        'x' => 0,
                        'y' => 1,
                        'z' => 2,
                        'w' => 3,
                        _ => return Err(attr_err()),
                    };
                    if idx >= $n {
                        return Err(attr_err());
                    }
                    vals[i] = slf.0[idx];
                }
                Ok(match len {
                    2 => PyVec2(Vec2::new(vals[0], vals[1]))
                        .into_pyobject(py)?
                        .into_any(),
                    3 => PyVec3(Vec3::new(vals[0], vals[1], vals[2]))
                        .into_pyobject(py)?
                        .into_any(),
                    4 => PyVec4(Vec4::new(vals[0], vals[1], vals[2], vals[3]))
                        .into_pyobject(py)?
                        .into_any(),
                    _ => unreachable!(),
                })
            }

            fn length(&self) -> f32 { self.0.length() }
            fn length_squared(&self) -> f32 { self.0.length_squared() }
            fn normalize(&self) -> Self { Self(self.0.normalize()) }
            fn dot(&self, other: &Self) -> f32 { self.0.dot(other.0) }
            fn distance(&self, other: &Self) -> f32 { self.0.distance(other.0) }
            fn lerp(&self, other: &Self, t: f32) -> Self { Self(self.0.lerp(other.0, t)) }

            #[pyo3(name = "min")]
            fn py_min(&self, other: &Self) -> Self { Self(self.0.min(other.0)) }
            #[pyo3(name = "max")]
            fn py_max(&self, other: &Self) -> Self { Self(self.0.max(other.0)) }
            fn clamp(&self, min: &Self, max: &Self) -> Self { Self(self.0.clamp(min.0, max.0)) }
            #[pyo3(name = "abs")]
            fn py_abs(&self) -> Self { Self(self.0.abs()) }

            fn to_list(&self) -> Vec<f32> { self.0.to_array().to_vec() }

            fn to_tuple<'py>(&self, py: Python<'py>) -> Bound<'py, PyTuple> {
                PyTuple::new(py, self.0.to_array()).unwrap()
            }

            fn mag(&self) -> f32 { self.0.length() }
            fn mag_sq(&self) -> f32 { self.0.length_squared() }
            fn dist(&self, other: &Self) -> f32 { self.0.distance(other.0) }

            fn copy(&self) -> Self { Self(self.0) }

            fn equals(&self, other: &Self) -> bool { self.0 == other.0 }

            fn hash_code(&self) -> u64 {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                for &c in self.0.to_array().iter() {
                    hash_f32(c, &mut hasher);
                }
                std::hash::Hasher::finish(&hasher)
            }

            #[pyo3(signature = (*args))]
            fn set(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
                match args.len() {
                    1 => {
                        let first = args.get_item(0)?;
                        if let Ok(s) = first.extract::<f32>() {
                            self.0 = <$glam_ty>::splat(s);
                            return Ok(());
                        }
                        if let Ok(s) = first.extract::<i64>() {
                            self.0 = <$glam_ty>::splat(s as f32);
                            return Ok(());
                        }
                        if let Ok(v) = first.extract::<PyRef<$name>>() {
                            self.0 = v.0;
                            return Ok(());
                        }
                        if let Ok(arr) = first.extract::<[f32; $n]>() {
                            self.0 = <$glam_ty>::from_array(arr);
                            return Ok(());
                        }
                        Err(PyTypeError::new_err(concat!(
                            "expected scalar, ", $py_name, ", or sequence of ",
                            stringify!($n), " floats"
                        )))
                    }
                    $n => {
                        let mut arr = [0.0f32; $n];
                        $(arr[$idx] = args.get_item($idx)?.extract::<f32>()?;)+
                        self.0 = <$glam_ty>::from_array(arr);
                        Ok(())
                    }
                    _ => Err(PyTypeError::new_err(concat!(
                        $py_name, ".set takes 1 or ", stringify!($n), " arguments"
                    ))),
                }
            }

            #[pyo3(signature = (*args))]
            fn add(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
                match args.len() {
                    1 => {
                        let first = args.get_item(0)?;
                        if let Ok(v) = first.extract::<PyRef<$name>>() {
                            self.0 += v.0;
                            return Ok(());
                        }
                        if let Ok(arr) = first.extract::<[f32; $n]>() {
                            self.0 += <$glam_ty>::from_array(arr);
                            return Ok(());
                        }
                        Err(PyTypeError::new_err(concat!(
                            "expected ", $py_name, " or sequence of ",
                            stringify!($n), " floats"
                        )))
                    }
                    $n => {
                        let mut arr = [0.0f32; $n];
                        $(arr[$idx] = args.get_item($idx)?.extract::<f32>()?;)+
                        self.0 += <$glam_ty>::from_array(arr);
                        Ok(())
                    }
                    _ => Err(PyTypeError::new_err(concat!(
                        $py_name, ".add takes 1 or ", stringify!($n), " arguments"
                    ))),
                }
            }

            #[pyo3(signature = (*args))]
            fn sub(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
                match args.len() {
                    1 => {
                        let first = args.get_item(0)?;
                        if let Ok(v) = first.extract::<PyRef<$name>>() {
                            self.0 -= v.0;
                            return Ok(());
                        }
                        if let Ok(arr) = first.extract::<[f32; $n]>() {
                            self.0 -= <$glam_ty>::from_array(arr);
                            return Ok(());
                        }
                        Err(PyTypeError::new_err(concat!(
                            "expected ", $py_name, " or sequence of ",
                            stringify!($n), " floats"
                        )))
                    }
                    $n => {
                        let mut arr = [0.0f32; $n];
                        $(arr[$idx] = args.get_item($idx)?.extract::<f32>()?;)+
                        self.0 -= <$glam_ty>::from_array(arr);
                        Ok(())
                    }
                    _ => Err(PyTypeError::new_err(concat!(
                        $py_name, ".sub takes 1 or ", stringify!($n), " arguments"
                    ))),
                }
            }

            fn mult(&mut self, n: f32) { self.0 *= n; }
            fn div(&mut self, n: f32) { self.0 /= n; }

            fn set_mag(&mut self, len: f32) {
                self.0 = self.0.normalize_or_zero() * len;
            }

            fn limit(&mut self, max: f32) {
                if self.0.length_squared() > max * max {
                    self.0 = self.0.normalize_or_zero() * max;
                }
            }

            #[staticmethod]
            fn angle_between(v1: &Self, v2: &Self) -> f32 {
                if v1.0.length_squared() == 0.0 || v2.0.length_squared() == 0.0 {
                    return 0.0;
                }
                let cos_angle = (v1.0.dot(v2.0)
                    / (v1.0.length() * v2.0.length()))
                .clamp(-1.0, 1.0);
                cos_angle.acos()
            }

            $($($extra)*)?
        }
    };
}

impl_py_vec!(PyVec2, "Vec2", 2, [(x, set_x, 0), (y, set_y, 1)], Vec2, extra {
    fn angle(&self) -> f32 {
        self.0.y.atan2(self.0.x)
    }

    fn heading(&self) -> f32 {
        self.0.y.atan2(self.0.x)
    }

    fn rotate(&self, angle: f32) -> Self {
        Self(Vec2::from_angle(angle).rotate(self.0))
    }

    fn perpendicular(&self) -> Self {
        Self(self.0.perp())
    }

    fn set_heading(&mut self, angle: f32) {
        let mag = self.0.length();
        self.0 = Vec2::new(angle.cos() * mag, angle.sin() * mag);
    }

    #[staticmethod]
    fn from_angle(angle: f32) -> Self {
        Self(Vec2::from_angle(angle))
    }

    #[staticmethod]
    fn random() -> Self {
        use rand_distr::{Distribution, UnitCircle};
        let [x, y]: [f32; 2] = UnitCircle.sample(&mut rand::rng());
        Self(Vec2::new(x, y))
    }

    fn extend(&self, z: f32) -> PyVec3 {
        PyVec3(self.0.extend(z))
    }
});

impl_py_vec!(PyVec3, "Vec3", 3, [(x, set_x, 0), (y, set_y, 1), (z, set_z, 2)], Vec3, extra {
    fn cross(&self, other: &Self) -> Self {
        Self(self.0.cross(other.0))
    }

    #[staticmethod]
    fn random() -> Self {
        use rand_distr::{Distribution, UnitSphere};
        let [x, y, z]: [f32; 3] = UnitSphere.sample(&mut rand::rng());
        Self(Vec3::new(x, y, z))
    }

    fn extend(&self, w: f32) -> PyVec4 {
        PyVec4(self.0.extend(w))
    }

    fn truncate(&self) -> PyVec2 {
        PyVec2(self.0.truncate())
    }
});

impl_py_vec!(
    PyVec4,
    "Vec4",
    4,
    [(x, set_x, 0), (y, set_y, 1), (z, set_z, 2), (w, set_w, 3)],
    Vec4,
    extra {
        fn truncate(&self) -> PyVec3 {
            PyVec3(self.0.truncate())
        }
    }
);

#[pyclass(name = "Quat", from_py_object)]
#[derive(Clone, Debug)]
pub struct PyQuat(pub(crate) Quat);

impl From<Quat> for PyQuat {
    fn from(q: Quat) -> Self {
        Self(q)
    }
}

impl From<PyQuat> for Quat {
    fn from(q: PyQuat) -> Self {
        q.0
    }
}

#[pymethods]
impl PyQuat {
    #[new]
    #[pyo3(signature = (*args))]
    pub fn py_new(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        match args.len() {
            0 => Ok(Self(Quat::IDENTITY)),
            4 => Ok(Self(Quat::from_xyzw(
                args.get_item(0)?.extract()?,
                args.get_item(1)?.extract()?,
                args.get_item(2)?.extract()?,
                args.get_item(3)?.extract()?,
            ))),
            _ => Err(PyTypeError::new_err("Quat takes 0 or 4 arguments")),
        }
    }

    #[staticmethod]
    fn identity() -> Self {
        Self(Quat::IDENTITY)
    }

    #[staticmethod]
    fn from_rotation_x(angle: f32) -> Self {
        Self(Quat::from_rotation_x(angle))
    }

    #[staticmethod]
    fn from_rotation_y(angle: f32) -> Self {
        Self(Quat::from_rotation_y(angle))
    }

    #[staticmethod]
    fn from_rotation_z(angle: f32) -> Self {
        Self(Quat::from_rotation_z(angle))
    }

    #[staticmethod]
    fn from_axis_angle(axis: Vec3Like, angle: f32) -> Self {
        Self(Quat::from_axis_angle(axis.into_vec3().normalize(), angle))
    }

    #[staticmethod]
    fn from_euler(x: f32, y: f32, z: f32) -> Self {
        Self(Quat::from_euler(EulerRot::XYZ, x, y, z))
    }

    // --- Properties (using to_array/from_array for SIMD compat) ---
    #[getter]
    fn x(&self) -> f32 {
        self.0.x
    }
    #[setter]
    fn set_x(&mut self, val: f32) {
        let [_, y, z, w] = self.0.to_array();
        self.0 = Quat::from_xyzw(val, y, z, w);
    }

    #[getter]
    fn y(&self) -> f32 {
        self.0.y
    }
    #[setter]
    fn set_y(&mut self, val: f32) {
        let [x, _, z, w] = self.0.to_array();
        self.0 = Quat::from_xyzw(x, val, z, w);
    }

    #[getter]
    fn z(&self) -> f32 {
        self.0.z
    }
    #[setter]
    fn set_z(&mut self, val: f32) {
        let [x, y, _, w] = self.0.to_array();
        self.0 = Quat::from_xyzw(x, y, val, w);
    }

    #[getter]
    fn w(&self) -> f32 {
        self.0.w
    }
    #[setter]
    fn set_w(&mut self, val: f32) {
        let [x, y, z, _] = self.0.to_array();
        self.0 = Quat::from_xyzw(x, y, z, val);
    }

    fn normalize(&self) -> Self {
        Self(self.0.normalize())
    }
    fn inverse(&self) -> Self {
        Self(self.0.inverse())
    }
    fn slerp(&self, other: &Self, t: f32) -> Self {
        Self(self.0.slerp(other.0, t))
    }
    fn length(&self) -> f32 {
        self.0.length()
    }
    fn dot(&self, other: &Self) -> f32 {
        self.0.dot(other.0)
    }

    fn mul_vec3(&self, v: Vec3Like) -> PyVec3 {
        PyVec3(self.0.mul_vec3(v.into_vec3()))
    }

    fn to_euler(&self) -> PyVec3 {
        let (x, y, z) = self.0.to_euler(EulerRot::XYZ);
        PyVec3(Vec3::new(x, y, z))
    }

    fn __mul__(&self, rhs: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let py = rhs.py();
        if let Ok(other) = rhs.extract::<PyRef<PyQuat>>() {
            return Ok(PyQuat(self.0 * other.0)
                .into_pyobject(py)?
                .into_any()
                .unbind());
        }
        if let Ok(v) = rhs.extract::<PyRef<PyVec3>>() {
            return Ok(PyVec3(self.0.mul_vec3(v.0))
                .into_pyobject(py)?
                .into_any()
                .unbind());
        }
        Err(PyTypeError::new_err(
            "unsupported operand type(s) for *: 'Quat'",
        ))
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for &c in self.0.to_array().iter() {
            hash_f32(c, &mut hasher);
        }
        std::hash::Hasher::finish(&hasher)
    }

    fn __repr__(&self) -> String {
        format!(
            "Quat({}, {}, {}, {})",
            self.0.x, self.0.y, self.0.z, self.0.w
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

#[pyclass]
pub struct PyVecIter {
    pub(crate) values: Vec<f32>,
    pub(crate) index: usize,
}

#[pymethods]
impl PyVecIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<f32> {
        if self.index < self.values.len() {
            let val = self.values[self.index];
            self.index += 1;
            Some(val)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn test_vec3_basics() {
        let v = PyVec3(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(v.0.x, 1.0);
        assert_eq!(v.0.y, 2.0);
        assert_eq!(v.0.z, 3.0);
    }

    #[test]
    fn test_vec3_arithmetic() {
        let a = PyVec3(Vec3::new(1.0, 2.0, 3.0));
        let b = PyVec3(Vec3::new(4.0, 5.0, 6.0));
        assert_eq!((a.0 + b.0), Vec3::new(5.0, 7.0, 9.0));
        assert_eq!((a.0 - b.0), Vec3::new(-3.0, -3.0, -3.0));
        assert_eq!((-a.0), Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!((a.0 * 2.0), Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_vec3_cross() {
        let a = PyVec3(Vec3::X);
        let b = PyVec3(Vec3::Y);
        assert_eq!(a.cross(&b).0, Vec3::Z);
    }

    #[test]
    fn test_vec3_normalize() {
        let v = PyVec3(Vec3::new(3.0, 0.0, 0.0));
        let n = v.normalize();
        assert!((n.0.length() - 1.0).abs() < 1e-6);
        assert_eq!(n.0, Vec3::X);
    }

    #[test]
    fn test_vec3_dot() {
        let a = PyVec3(Vec3::new(1.0, 2.0, 3.0));
        let b = PyVec3(Vec3::new(4.0, 5.0, 6.0));
        assert_eq!(a.dot(&b), 32.0);
    }

    #[test]
    fn test_vec2_angle() {
        let v = PyVec2(Vec2::X);
        assert!(v.angle().abs() < 1e-6);
        let v = PyVec2(Vec2::Y);
        assert!((v.angle() - FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn test_vec2_perpendicular() {
        let v = PyVec2(Vec2::X);
        let p = v.perpendicular();
        assert!((p.0.x).abs() < 1e-6);
        assert!((p.0.y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_quat_rotation() {
        let q = PyQuat(Quat::from_rotation_z(FRAC_PI_2));
        let rotated = q.0.mul_vec3(Vec3::X);
        assert!(rotated.x.abs() < 1e-5);
        assert!((rotated.y - 1.0).abs() < 1e-5);
        assert!(rotated.z.abs() < 1e-5);
    }

    #[test]
    fn test_quat_composition() {
        let q1 = Quat::from_rotation_z(FRAC_PI_2);
        let q2 = Quat::from_rotation_z(FRAC_PI_2);
        let composed = q1 * q2;
        let v = composed.mul_vec3(Vec3::X);
        assert!((v.x - (-1.0)).abs() < 1e-5);
        assert!(v.y.abs() < 1e-5);
    }

    #[test]
    fn test_vec_conversions() {
        let v = PyVec3(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(v.to_list(), vec![1.0, 2.0, 3.0]);
        let glam_v: Vec3 = v.into();
        assert_eq!(glam_v, Vec3::new(1.0, 2.0, 3.0));
        let back: PyVec3 = glam_v.into();
        assert_eq!(back.0, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_vec3_lerp() {
        let a = PyVec3(Vec3::ZERO);
        let b = PyVec3(Vec3::new(10.0, 10.0, 10.0));
        let mid = a.lerp(&b, 0.5);
        assert_eq!(mid.0, Vec3::new(5.0, 5.0, 5.0));
    }

    #[test]
    fn test_vec3_distance() {
        let a = PyVec3(Vec3::ZERO);
        let b = PyVec3(Vec3::new(3.0, 4.0, 0.0));
        assert!((a.distance(&b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec3_pvector_aliases() {
        let a = PyVec3(Vec3::new(3.0, 4.0, 0.0));
        let b = PyVec3(Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(a.mag(), 5.0);
        assert_eq!(a.mag_sq(), 25.0);
        assert_eq!(a.dist(&b), 5.0);
    }

    #[test]
    fn test_vec3_copy_and_equals() {
        let a = PyVec3(Vec3::new(1.0, 2.0, 3.0));
        let b = a.copy();
        assert!(a.equals(&b));
        assert_eq!(a.hash_code(), b.hash_code());
    }

    #[test]
    fn test_vec3_mut_add_sub() {
        let mut v = PyVec3(Vec3::new(1.0, 2.0, 3.0));
        v.0 += Vec3::new(1.0, 1.0, 1.0);
        assert_eq!(v.0, Vec3::new(2.0, 3.0, 4.0));
        v.0 -= Vec3::new(2.0, 0.0, 4.0);
        assert_eq!(v.0, Vec3::new(0.0, 3.0, 0.0));
    }

    #[test]
    fn test_vec3_mult_div() {
        let mut v = PyVec3(Vec3::new(1.0, 2.0, 3.0));
        v.mult(2.0);
        assert_eq!(v.0, Vec3::new(2.0, 4.0, 6.0));
        v.div(2.0);
        assert_eq!(v.0, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_vec3_set_mag() {
        let mut v = PyVec3(Vec3::new(3.0, 4.0, 0.0));
        v.set_mag(10.0);
        assert!((v.0.length() - 10.0).abs() < 1e-5);
        assert!((v.0.x - 6.0).abs() < 1e-5);
        assert!((v.0.y - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_vec3_set_mag_zero_vec_stays_zero() {
        let mut v = PyVec3(Vec3::ZERO);
        v.set_mag(5.0);
        assert_eq!(v.0, Vec3::ZERO);
    }

    #[test]
    fn test_vec3_limit() {
        let mut v = PyVec3(Vec3::new(3.0, 4.0, 0.0));
        v.limit(10.0);
        assert_eq!(v.0, Vec3::new(3.0, 4.0, 0.0));
        v.limit(2.5);
        assert!((v.0.length() - 2.5).abs() < 1e-5);
    }

    #[test]
    fn test_angle_between() {
        let a = PyVec3(Vec3::X);
        let b = PyVec3(Vec3::Y);
        assert!((PyVec3::angle_between(&a, &b) - FRAC_PI_2).abs() < 1e-5);

        let z = PyVec3(Vec3::ZERO);
        assert_eq!(PyVec3::angle_between(&a, &z), 0.0);
    }

    #[test]
    fn test_vec2_heading_alias() {
        let v = PyVec2(Vec2::Y);
        assert!((v.heading() - FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn test_vec2_set_heading_preserves_mag() {
        let mut v = PyVec2(Vec2::new(3.0, 4.0));
        let mag_before = v.0.length();
        v.set_heading(0.0);
        assert!((v.0.length() - mag_before).abs() < 1e-5);
        assert!((v.0.x - mag_before).abs() < 1e-5);
        assert!(v.0.y.abs() < 1e-5);
    }

    #[test]
    fn test_vec2_from_angle() {
        let v = PyVec2::from_angle(0.0);
        assert!((v.0.x - 1.0).abs() < 1e-6);
        assert!(v.0.y.abs() < 1e-6);

        let v = PyVec2::from_angle(FRAC_PI_2);
        assert!(v.0.x.abs() < 1e-6);
        assert!((v.0.y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec2_random_is_unit() {
        for _ in 0..32 {
            let v = PyVec2::random();
            assert!((v.0.length() - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_vec3_random_is_unit() {
        for _ in 0..32 {
            let v = PyVec3::random();
            assert!((v.0.length() - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_vec2_extend() {
        let v = PyVec2(Vec2::new(1.0, 2.0));
        let v3 = v.extend(3.0);
        assert_eq!(v3.0, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_vec3_extend_truncate() {
        let v = PyVec3(Vec3::new(1.0, 2.0, 3.0));
        let v4 = v.extend(4.0);
        assert_eq!(v4.0, Vec4::new(1.0, 2.0, 3.0, 4.0));
        let v2 = v.truncate();
        assert_eq!(v2.0, Vec2::new(1.0, 2.0));
    }

    #[test]
    fn test_vec4_truncate() {
        let v = PyVec4(Vec4::new(1.0, 2.0, 3.0, 4.0));
        let v3 = v.truncate();
        assert_eq!(v3.0, Vec3::new(1.0, 2.0, 3.0));
    }
}
