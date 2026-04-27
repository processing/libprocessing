use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::{exceptions::PyRuntimeError, prelude::*};

#[pyclass(unsendable)]
pub struct Monitor {
    pub(crate) entity: Entity,
}

#[pymethods]
impl Monitor {
    #[getter]
    pub fn width(&self) -> PyResult<u32> {
        monitor_width(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[getter]
    pub fn height(&self) -> PyResult<u32> {
        monitor_height(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[getter]
    pub fn scale_factor(&self) -> PyResult<f64> {
        monitor_scale_factor(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[getter]
    pub fn refresh_rate_millihertz(&self) -> PyResult<Option<u32>> {
        monitor_refresh_rate_millihertz(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[getter]
    pub fn name(&self) -> PyResult<Option<String>> {
        monitor_name(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[getter]
    pub fn position(&self) -> PyResult<(i32, i32)> {
        let p =
            monitor_position(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok((p.x, p.y))
    }

    #[getter]
    pub fn workarea(&self) -> PyResult<(i32, i32, i32, i32)> {
        let r =
            monitor_workarea(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok((r.min.x, r.min.y, r.width(), r.height()))
    }
}

pub fn primary() -> PyResult<Option<Monitor>> {
    let entity = monitor_primary().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    Ok(entity.map(|entity| Monitor { entity }))
}

pub fn list() -> PyResult<Vec<Monitor>> {
    let entities = monitor_list().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    Ok(entities
        .into_iter()
        .map(|entity| Monitor { entity })
        .collect())
}
