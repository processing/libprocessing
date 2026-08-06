use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::types::PyDict;
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
        return Ok(shader_value::ShaderValue::Texture(img_ref.entity));
    }
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
}

impl Drop for Material {
    fn drop(&mut self) {
        let _ = material_destroy(self.entity);
    }
}
