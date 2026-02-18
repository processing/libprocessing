use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::{exceptions::PyRuntimeError, prelude::*};

#[pyclass(unsendable)]
pub struct Material {
    pub(crate) entity: Entity,
}

#[pymethods]
impl Material {
    #[new]
    pub fn new() -> PyResult<Self> {
        let entity = material_create_pbr().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Self { entity })
    }

    pub fn set_float(&self, name: &str, value: f32) -> PyResult<()> {
        material_set(self.entity, name, material::MaterialValue::Float(value))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn set_float4(&self, name: &str, r: f32, g: f32, b: f32, a: f32) -> PyResult<()> {
        material_set(
            self.entity,
            name,
            material::MaterialValue::Float4([r, g, b, a]),
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }
}

impl Drop for Material {
    fn drop(&mut self) {
        let _ = material_destroy(self.entity);
    }
}
