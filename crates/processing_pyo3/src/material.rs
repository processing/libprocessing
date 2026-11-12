use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::types::{PyDict, PyInt};
use pyo3::{exceptions::PyRuntimeError, prelude::*};

use crate::color::PyColor;
use crate::compute::Buffer;
use crate::graphics::ImageRef;
use crate::math::{PyVec2, PyVec3, PyVec4};
use crate::shader::Shader;

#[pyclass(unsendable)]
pub struct Material {
    pub(crate) entity: Entity,
}

pub(crate) fn py_to_shader_value(value: &Bound<'_, PyAny>) -> PyResult<shader_value::ShaderValue> {
    if let Ok(img_ref) = value.extract::<ImageRef>() {
        return Ok(shader_value::ShaderValue::Texture(img_ref.texture()?));
    }
    if let Ok(int_val) = value.cast::<PyInt>() {
        if let Ok(v) = int_val.extract::<i32>() {
            return Ok(shader_value::ShaderValue::Int(v));
        }
        if let Ok(v) = int_val.extract::<u32>() {
            return Ok(shader_value::ShaderValue::UInt(v));
        }
    }
    if let Ok(v) = value.extract::<f32>() {
        return Ok(shader_value::ShaderValue::Float(v));
    }

    if let Ok(v) = value.extract::<PyRef<PyVec4>>() {
        return Ok(shader_value::ShaderValue::Float4(v.0.to_array()));
    }
    if let Ok(v) = value.extract::<PyRef<PyVec3>>() {
        return Ok(shader_value::ShaderValue::Float3(v.0.to_array()));
    }
    if let Ok(v) = value.extract::<PyRef<PyVec2>>() {
        return Ok(shader_value::ShaderValue::Float2(v.0.to_array()));
    }
    // before the sequence fallbacks: a Color iterates in its own space (h, s, v, a for hsva)
    if let Ok(c) = value.extract::<PyRef<PyColor>>() {
        return Ok(shader_value::ShaderValue::Color(c.0));
    }

    if let Ok(buf) = value.extract::<PyRef<Buffer>>() {
        return Ok(shader_value::ShaderValue::Buffer(buf.entity));
    }
    if let Ok(grid) = value.extract::<PyRef<crate::particles::Grid>>() {
        return Ok(shader_value::ShaderValue::Grid(grid.entity));
    }

    // row-major like `get_matrix()`, WGSL wants columns
    if let Ok(rows) = value.extract::<[f32; 16]>() {
        let m = bevy::math::Mat4::from_cols_array(&rows).transpose();
        return Ok(shader_value::ShaderValue::Mat4(m.to_cols_array()));
    }
    if let Ok(rows) = value.extract::<[[f32; 4]; 4]>() {
        let m = bevy::math::Mat4::from_cols_array_2d(&rows).transpose();
        return Ok(shader_value::ShaderValue::Mat4(m.to_cols_array()));
    }

    if let Ok(v) = value.extract::<[f32; 4]>() {
        return Ok(shader_value::ShaderValue::Float4(v));
    }
    if let Ok(v) = value.extract::<[f32; 3]>() {
        return Ok(shader_value::ShaderValue::Float3(v));
    }
    if let Ok(v) = value.extract::<[f32; 2]>() {
        return Ok(shader_value::ShaderValue::Float2(v));
    }

    Err(PyRuntimeError::new_err(format!(
        "unsupported material value type: {}",
        value.get_type().name()?
    )))
}

fn apply_albedo(entity: Entity, value: &Bound<'_, PyAny>) -> PyResult<()> {
    if let Ok(buf) = value.extract::<PyRef<Buffer>>() {
        return material_set_albedo_buffer(entity, buf.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")));
    }
    let rgba = if let Ok(c) = value.extract::<PyRef<PyColor>>() {
        let srgba: bevy::color::Srgba = c.0.into();
        Some([srgba.red, srgba.green, srgba.blue, srgba.alpha])
    } else if let Ok(rgba) = value.extract::<[f32; 4]>() {
        Some(rgba)
    } else if let Ok(rgb) = value.extract::<[f32; 3]>() {
        Some([rgb[0], rgb[1], rgb[2], 1.0])
    } else {
        None
    };
    if let Some(rgba) = rgba {
        return material_set(entity, "color", shader_value::ShaderValue::Float4(rgba))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")));
    }
    Err(PyRuntimeError::new_err(format!(
        "unsupported albedo type: {} (expected Color, Buffer, or [r,g,b,(a)])",
        value.get_type().name()?
    )))
}

fn apply_emissive(entity: Entity, value: &Bound<'_, PyAny>) -> PyResult<()> {
    let rt = |e| PyRuntimeError::new_err(format!("{e}"));
    if let Ok(buf) = value.extract::<PyRef<Buffer>>() {
        return material_set_emissive_buffer(entity, buf.entity).map_err(rt);
    }
    // emissive is a linear radiance, not an sRGB color
    let linear: bevy::color::LinearRgba = if let Ok(c) = value.extract::<PyRef<PyColor>>() {
        c.0.into()
    } else if let Ok([r, g, b, a]) = value.extract::<[f32; 4]>() {
        bevy::color::LinearRgba::new(r, g, b, a)
    } else if let Ok([r, g, b]) = value.extract::<[f32; 3]>() {
        bevy::color::LinearRgba::new(r, g, b, 1.0)
    } else {
        return Err(PyRuntimeError::new_err(format!(
            "unsupported emissive type: {} (expected Color, Buffer, or [r,g,b,(a)])",
            value.get_type().name()?
        )));
    };
    material_set(
        entity,
        "emissive",
        shader_value::ShaderValue::Float4([linear.red, linear.green, linear.blue, linear.alpha]),
    )
    .map_err(rt)
}

fn py_truthy(value: &Bound<'_, PyAny>) -> PyResult<bool> {
    value
        .extract::<bool>()
        .or_else(|_| value.extract::<f64>().map(|f| f > 0.5))
}

fn apply_kwargs(entity: Entity, kwargs: &Bound<'_, PyDict>) -> PyResult<()> {
    for (key, value) in kwargs.iter() {
        let name: String = key.extract()?;
        let rt = |e| PyRuntimeError::new_err(format!("{e}"));
        match name.as_str() {
            "albedo" => apply_albedo(entity, &value)?,
            "emissive" => apply_emissive(entity, &value)?,
            "unlit" => material_set_unlit(entity, py_truthy(&value)?).map_err(rt)?,
            "double_sided" => material_set_double_sided(entity, py_truthy(&value)?).map_err(rt)?,
            "depth_write" => material_set_depth_write(entity, py_truthy(&value)?).map_err(rt)?,
            "alpha_mode" => {
                material_set_alpha_mode(entity, value.extract::<u8>()?, 0.5).map_err(rt)?
            }
            _ => {
                let v = py_to_shader_value(&value)?;
                material_set(entity, &name, v).map_err(rt)?;
            }
        }
    }
    Ok(())
}

#[pymethods]
impl Material {
    /// Opaque id for this object.
    pub fn id(&self) -> u64 {
        self.entity.to_bits()
    }

    #[new]
    #[pyo3(signature = (shader=None, **kwargs))]
    pub fn new(shader: Option<&Shader>, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let entity = if let Some(shader) = shader {
            material_create_custom(shader.entity)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?
        } else {
            material_create_pbr().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?
        };

        if let Some(kwargs) = kwargs {
            apply_kwargs(entity, kwargs)?;
        }
        Ok(Self { entity })
    }

    #[staticmethod]
    #[pyo3(signature = (**kwargs))]
    pub fn pbr(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let entity = material_create_pbr().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        if let Some(kwargs) = kwargs {
            apply_kwargs(entity, kwargs)?;
        }
        Ok(Self { entity })
    }

    #[staticmethod]
    #[pyo3(signature = (**kwargs))]
    pub fn unlit(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let entity = material_create_pbr().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        material_set_unlit(entity, true).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        if let Some(kwargs) = kwargs {
            apply_kwargs(entity, kwargs)?;
        }
        Ok(Self { entity })
    }

    #[pyo3(signature = (**kwargs))]
    pub fn set(&self, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let Some(kwargs) = kwargs else {
            return Ok(());
        };
        apply_kwargs(self.entity, kwargs)
    }

    /// Base color: a `Color`, `[r, g, b, (a)]`, or a per-instance `Buffer`.
    pub fn albedo(&self, value: &Bound<'_, PyAny>) -> PyResult<()> {
        apply_albedo(self.entity, value)
    }

    pub fn metalness(&self, value: f32) -> PyResult<()> {
        self.set_pbr("metallic", value)
    }

    pub fn roughness(&self, value: f32) -> PyResult<()> {
        self.set_pbr("roughness", value)
    }

    pub fn reflectance(&self, value: f32) -> PyResult<()> {
        self.set_pbr("reflectance", value)
    }

    /// Emitted light: a `Color`, linear `[r, g, b, (a)]`, or a per-instance `Buffer`.
    pub fn emissive(&self, value: &Bound<'_, PyAny>) -> PyResult<()> {
        apply_emissive(self.entity, value)
    }

    pub fn opaque(&self) -> PyResult<()> {
        material_set_alpha_mode(self.entity, 0, 0.0).map_err(rt)
    }

    /// Discards fragments with alpha below `cutoff`.
    #[pyo3(signature = (cutoff=0.5))]
    pub fn mask(&self, cutoff: f32) -> PyResult<()> {
        material_set_alpha_mode(self.entity, 1, cutoff).map_err(rt)
    }

    pub fn transparent(&self) -> PyResult<()> {
        material_set_alpha_mode(self.entity, 2, 0.0).map_err(rt)
    }

    #[pyo3(signature = (on=true))]
    pub fn double_sided(&self, on: bool) -> PyResult<()> {
        material_set_double_sided(self.entity, on).map_err(rt)
    }

    #[pyo3(signature = (on=true))]
    pub fn depth_write(&self, on: bool) -> PyResult<()> {
        material_set_depth_write(self.entity, on).map_err(rt)
    }

    /// Blends with what's behind it using a `BlendMode` (`ADD`, `MULTIPLY`, a custom one, ...).
    pub fn blend_mode(&self, mode: &crate::graphics::PyBlendMode) -> PyResult<()> {
        let state = mode
            .blend_state
            .unwrap_or(bevy::render::render_resource::BlendState::ALPHA_BLENDING);
        material_set_custom_blend(self.entity, state).map_err(rt)
    }
}

fn rt(e: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(format!("{e}"))
}

impl Material {
    fn set_pbr(&self, name: &str, value: f32) -> PyResult<()> {
        material_set(self.entity, name, shader_value::ShaderValue::Float(value)).map_err(rt)
    }
}

impl Drop for Material {
    fn drop(&mut self) {
        let _ = material_destroy(self.entity);
    }
}
