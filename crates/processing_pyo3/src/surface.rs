use bevy::prelude::Entity;
use bevy::window::{MonitorSelection, WindowLevel, WindowMode};
use processing::prelude::*;
use pyo3::{exceptions::PyRuntimeError, prelude::*};

use crate::glfw::GlfwContext;
use crate::monitor::{self, Monitor};
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

    pub fn set_title(&self, title: &str) -> PyResult<()> {
        surface_set_title(self.entity, title.to_string())
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[getter]
    pub fn position(&self) -> PyResult<(i32, i32)> {
        let p =
            surface_position(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok((p.x, p.y))
    }

    pub fn set_position(&self, x: i32, y: i32) -> PyResult<()> {
        surface_set_position(self.entity, x, y).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn set_visible(&self, visible: bool) -> PyResult<()> {
        surface_set_visible(self.entity, visible)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn show(&self) -> PyResult<()> {
        self.set_visible(true)
    }

    pub fn hide(&self) -> PyResult<()> {
        self.set_visible(false)
    }

    pub fn set_resizable(&self, resizable: bool) -> PyResult<()> {
        surface_set_resizable(self.entity, resizable)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn set_decorated(&self, decorated: bool) -> PyResult<()> {
        surface_set_decorated(self.entity, decorated)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn set_always_on_top(&self, on_top: bool) -> PyResult<()> {
        let level = if on_top {
            WindowLevel::AlwaysOnTop
        } else {
            WindowLevel::Normal
        };
        surface_set_window_level(self.entity, level)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn set_opacity(&self, opacity: f32) -> PyResult<()> {
        surface_set_opacity(self.entity, opacity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn iconify(&self) -> PyResult<()> {
        surface_iconify(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn restore(&self) -> PyResult<()> {
        surface_restore(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn maximize(&self) -> PyResult<()> {
        surface_maximize(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn focus(&self) -> PyResult<()> {
        surface_focus(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (monitor=None))]
    pub fn set_fullscreen(&self, monitor: Option<&Monitor>) -> PyResult<()> {
        let mode = match monitor {
            Some(m) => WindowMode::BorderlessFullscreen(MonitorSelection::Entity(m.entity)),
            None => WindowMode::Windowed,
        };
        surface_set_window_mode(self.entity, mode)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn position_on(&self, monitor: &Monitor, x: i32, y: i32) -> PyResult<()> {
        surface_position_on_monitor(self.entity, monitor.entity, x, y)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn center_on(&self, monitor: &Monitor) -> PyResult<()> {
        surface_center_on_monitor(self.entity, monitor.entity)
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

    let (wx, wy) = surface.position().unwrap_or((0, 0));
    set_tracked(globals, "window_x", wx)?;
    set_tracked(globals, "window_y", wy)?;

    let (dw, dh) = match monitor::primary()? {
        Some(m) => (m.width()?, m.height()?),
        None => (0, 0),
    };
    set_tracked(globals, "display_width", dw)?;
    set_tracked(globals, "display_height", dh)?;
    Ok(())
}
