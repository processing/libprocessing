use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::types::PyDict;
use pyo3::{exceptions::PyRuntimeError, prelude::*};

use crate::color::PyColor;
use crate::compute::Buffer;
use crate::math::{PyVec2, PyVec3, PyVec4};
use crate::shader::Shader;

#[pyclass(unsendable)]
pub struct Material {
    pub(crate) entity: Entity,
}

pub(crate) fn py_to_shader_value(value: &Bound<'_, PyAny>) -> PyResult<shader_value::ShaderValue> {
    if let Ok(v) = value.extract::<f32>() {
        return Ok(shader_value::ShaderValue::Float(v));
    }
    if let Ok(v) = value.extract::<i32>() {
        return Ok(shader_value::ShaderValue::Int(v));
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

    if let Ok(buf) = value.extract::<PyRef<Buffer>>() {
        return Ok(shader_value::ShaderValue::Buffer(buf.entity));
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

/// Dispatch `albedo=` by Python type to the matching Rust setter. May swap
/// the backing asset; all other StandardMaterial state survives.
fn apply_albedo(entity: Entity, value: &Bound<'_, PyAny>) -> PyResult<()> {
    if let Ok(buf) = value.extract::<PyRef<Buffer>>() {
        return material_set_albedo_buffer(entity, buf.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")));
    }
    if let Ok(c) = value.extract::<PyRef<PyColor>>() {
        let srgba: bevy::color::Srgba = c.0.into();
        return material_set_albedo_color(entity, [srgba.red, srgba.green, srgba.blue, srgba.alpha])
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")));
    }
    if let Ok(rgba) = value.extract::<[f32; 4]>() {
        return material_set_albedo_color(entity, rgba)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")));
    }
    if let Ok(rgb) = value.extract::<[f32; 3]>() {
        return material_set_albedo_color(entity, [rgb[0], rgb[1], rgb[2], 1.0])
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")));
    }
    Err(PyRuntimeError::new_err(format!(
        "unsupported albedo type: {} (expected Color, Buffer, or [r,g,b,(a)])",
        value.get_type().name()?
    )))
}

fn apply_kwargs(entity: Entity, kwargs: &Bound<'_, PyDict>) -> PyResult<()> {
    for (key, value) in kwargs.iter() {
        let name: String = key.extract()?;
        if name == "albedo" {
            apply_albedo(entity, &value)?;
            continue;
        }
        let v = py_to_shader_value(&value)?;
        material_set(entity, &name, v).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    }
    Ok(())
}

#[pymethods]
impl Material {
    /// No args: default PBR. With `shader`: custom material. Kwargs are
    /// applied via `set` after construction.
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

    /// PBR-lit material. `albedo` accepts a `Color` or a `Buffer` (the latter
    /// being per-particle, used with `Particles`).
    #[staticmethod]
    #[pyo3(signature = (**kwargs))]
    pub fn pbr(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let entity = material_create_pbr().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        if let Some(kwargs) = kwargs {
            apply_kwargs(entity, kwargs)?;
        }
        Ok(Self { entity })
    }

    /// Like `pbr` but skips lighting; albedo is the final output color.
    #[staticmethod]
    #[pyo3(signature = (**kwargs))]
    pub fn unlit(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let entity = material_create_pbr().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        material_set(entity, "unlit", shader_value::ShaderValue::Float(1.0))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        if let Some(kwargs) = kwargs {
            apply_kwargs(entity, kwargs)?;
        }
        Ok(Self { entity })
    }

    /// Patch material properties. `albedo` may swap the backing asset between
    /// color and buffer variants; other StandardMaterial fields are preserved.
    #[pyo3(signature = (**kwargs))]
    pub fn set(&self, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let Some(kwargs) = kwargs else {
            return Ok(());
        };
        apply_kwargs(self.entity, kwargs)
    }
}

impl Drop for Material {
    fn drop(&mut self) {
        let _ = material_destroy(self.entity);
    }
}
