use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::{exceptions::PyRuntimeError, prelude::*};

#[pyclass(unsendable)]
pub struct Shader {
    pub(crate) entity: Entity,
}

impl Shader {
    pub(crate) fn from_source(source: &str) -> PyResult<Self> {
        let entity = shader_create(source).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Self { entity })
    }

    pub(crate) fn from_path(path: &str) -> PyResult<Self> {
        let entity = shader_load(path).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Self { entity })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        let _ = shader_destroy(self.entity);
    }
}
