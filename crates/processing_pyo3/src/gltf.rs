use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::{exceptions::PyRuntimeError, prelude::*};

use crate::graphics::{Geometry, Light, get_graphics};
use crate::material::Material;

#[pyclass(unsendable)]
pub struct Gltf {
    entity: Entity,
}

#[pymethods]
impl Gltf {
    pub fn geometry(&self, name: &str) -> PyResult<Geometry> {
        let entity = gltf_geometry(self.entity, name)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Geometry { entity })
    }

    pub fn material(&self, name: &str) -> PyResult<Material> {
        let entity = gltf_material(self.entity, name)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Material { entity })
    }

    pub fn mesh_names(&self) -> PyResult<Vec<String>> {
        gltf_mesh_names(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn material_names(&self) -> PyResult<Vec<String>> {
        gltf_material_names(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn camera(&self, index: usize) -> PyResult<()> {
        gltf_camera(self.entity, index)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn light(&self, index: usize) -> PyResult<Light> {
        let entity = gltf_light(self.entity, index)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Light { entity })
    }
}

#[pyfunction]
#[pyo3(pass_module)]
pub fn load_gltf(module: &Bound<'_, PyModule>, path: &str) -> PyResult<Gltf> {
    let graphics = get_graphics(module)?;
    let entity = gltf_load(graphics.entity, path)
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    Ok(Gltf { entity })
}
