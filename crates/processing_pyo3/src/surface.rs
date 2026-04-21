use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::{exceptions::PyRuntimeError, prelude::*};

use crate::glfw::GlfwContext;
use crate::monitor;
use crate::set_tracked;

#[pyclass(unsendable)]
pub struct Surface {
    pub(crate) entity: Entity,
    pub(crate) glfw_ctx: Option<GlfwContext>,
}

#[pymethods]
impl Surface {
    pub fn poll_events(&mut self) -> bool {
        match &mut self.glfw_ctx {
            Some(ctx) => ctx.poll_events(),
            None => true, // no-op, offscreen surfaces never close
        }
    }

    #[getter]
    pub fn focused(&self) -> PyResult<bool> {
        surface_focused(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[getter]
    pub fn pixel_density(&self) -> PyResult<f32> {
        surface_scale_factor(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[getter]
    pub fn pixel_width(&self) -> PyResult<u32> {
        surface_physical_width(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[getter]
    pub fn pixel_height(&self) -> PyResult<u32> {
        surface_physical_height(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[getter]
    pub fn display_density(&self) -> PyResult<f32> {
        match &self.glfw_ctx {
            Some(ctx) => Ok(ctx.content_scale()),
            None => Ok(1.0),
        }
    }

    pub fn set_pixel_density(&self, density: f32) -> PyResult<()> {
        surface_set_pixel_density(self.entity, density)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        let _ = surface_destroy(self.entity);
    }
}

pub fn sync_globals(
    globals: &Bound<'_, PyAny>,
    surface: &Surface,
    canvas_width: u32,
    canvas_height: u32,
) -> PyResult<()> {
    set_tracked(globals, "width", canvas_width)?;
    set_tracked(globals, "height", canvas_height)?;
    set_tracked(globals, "focused", surface.focused()?)?;
    set_tracked(globals, "pixel_density", surface.pixel_density()?)?;
    set_tracked(globals, "pixel_width", surface.pixel_width()?)?;
    set_tracked(globals, "pixel_height", surface.pixel_height()?)?;

    let (dw, dh) = match monitor::primary()? {
        Some(m) => (m.width()?, m.height()?),
        None => (0, 0),
    };
    set_tracked(globals, "display_width", dw)?;
    set_tracked(globals, "display_height", dh)?;
    Ok(())
}
