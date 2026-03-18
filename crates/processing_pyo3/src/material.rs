use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::types::PyDict;
use pyo3::{exceptions::PyRuntimeError, prelude::*};

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

    // Accept PyVec types
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

    // Fall back to raw arrays
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

        let mat = Self { entity };
        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs.iter() {
                let name: String = key.extract()?;
                let value = py_to_shader_value(&value)?;
                material_set(mat.entity, &name, value)
                    .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
            }
        }
        Ok(mat)
    }

    #[pyo3(signature = (**kwargs))]
    pub fn set(&self, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let Some(kwargs) = kwargs else {
            return Ok(());
        };
        for (key, value) in kwargs.iter() {
            let name: String = key.extract()?;
            let value = py_to_shader_value(&value)?;
            material_set(self.entity, &name, value)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        }
        Ok(())
    }
}

impl Drop for Material {
    fn drop(&mut self) {
        let _ = material_destroy(self.entity);
    }
}
