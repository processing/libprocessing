use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};

pub fn mouse_x(surface: Entity, width: u32) -> PyResult<f32> {
    let raw = processing::prelude::input_mouse_x(surface)
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    Ok(raw.clamp(0.0, width as f32))
}

pub fn mouse_y(surface: Entity, height: u32) -> PyResult<f32> {
    let raw = processing::prelude::input_mouse_y(surface)
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    Ok(raw.clamp(0.0, height as f32))
}

pub fn pmouse_x(surface: Entity, width: u32) -> PyResult<f32> {
    let raw = processing::prelude::input_pmouse_x(surface)
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    Ok(raw.clamp(0.0, width as f32))
}

pub fn pmouse_y(surface: Entity, height: u32) -> PyResult<f32> {
    let raw = processing::prelude::input_pmouse_y(surface)
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    Ok(raw.clamp(0.0, height as f32))
}

pub fn mouse_is_pressed() -> PyResult<bool> {
    processing::prelude::input_mouse_is_pressed()
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn mouse_button() -> PyResult<Option<String>> {
    processing::prelude::input_mouse_button()
        .map(|opt| {
            opt.map(|b| match b {
                MouseButton::Left => constants::LEFT.to_string(),
                MouseButton::Right => constants::RIGHT.to_string(),
                MouseButton::Middle => constants::CENTER.to_string(),
                _ => format!("{b:?}").to_lowercase(),
            })
        })
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn moved_x() -> PyResult<f32> {
    processing::prelude::input_moved_x().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn moved_y() -> PyResult<f32> {
    processing::prelude::input_moved_y().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn mouse_wheel() -> PyResult<f32> {
    processing::prelude::input_mouse_wheel().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn key_is_pressed() -> PyResult<bool> {
    processing::prelude::input_key_is_pressed().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn key_is_down(key_code: u32) -> PyResult<bool> {
    let kc = key_code_from_u32(key_code).map_err(|e| PyValueError::new_err(format!("{e}")))?;
    processing::prelude::input_key_is_down(kc).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn key_just_pressed(key_code: u32) -> PyResult<bool> {
    let kc = key_code_from_u32(key_code).map_err(|e| PyValueError::new_err(format!("{e}")))?;
    processing::prelude::input_key_just_pressed(kc)
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn key() -> PyResult<Option<String>> {
    processing::prelude::input_key()
        .map(|opt| opt.map(String::from))
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn key_code() -> PyResult<Option<u32>> {
    processing::prelude::input_key_code()
        .map(|opt| opt.and_then(key_code_to_u32))
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn sync_globals(
    globals: &Bound<'_, PyAny>,
    surface: Entity,
    canvas_width: u32,
    canvas_height: u32,
) -> PyResult<()> {
    use crate::set_tracked;
    set_tracked(globals, "mouse_x", mouse_x(surface, canvas_width)?)?;
    set_tracked(globals, "mouse_y", mouse_y(surface, canvas_height)?)?;
    set_tracked(globals, "pmouse_x", pmouse_x(surface, canvas_width)?)?;
    set_tracked(globals, "pmouse_y", pmouse_y(surface, canvas_height)?)?;
    set_tracked(globals, "mouse_is_pressed", mouse_is_pressed()?)?;
    set_tracked(globals, "mouse_button", mouse_button()?)?;
    set_tracked(globals, "moved_x", moved_x()?)?;
    set_tracked(globals, "moved_y", moved_y()?)?;
    set_tracked(globals, "key", key()?)?;
    set_tracked(globals, "key_code", key_code()?)?;
    set_tracked(globals, "key_is_pressed", key_is_pressed()?)?;
    Ok(())
}

/// Passed to mouse callbacks that take a parameter, e.g. `def mouse_wheel(e)`.
#[pyclass(frozen, get_all)]
pub struct MouseEvent {
    pub x: f32,
    pub y: f32,
    pub button: Option<String>,
    /// Vertical scroll since the last frame.
    pub wheel: f32,
}

impl MouseEvent {
    pub fn current(surface: Entity, width: u32, height: u32) -> PyResult<Self> {
        Ok(Self {
            x: mouse_x(surface, width)?,
            y: mouse_y(surface, height)?,
            button: mouse_button()?,
            wheel: mouse_wheel()?,
        })
    }
}

#[pymethods]
impl MouseEvent {
    fn __repr__(&self) -> String {
        format!(
            "MouseEvent(x={}, y={}, button={:?}, wheel={})",
            self.x, self.y, self.button, self.wheel
        )
    }
}

/// Passed to key callbacks that take a parameter, e.g. `def key_pressed(e)`.
#[pyclass(frozen, get_all)]
pub struct KeyEvent {
    pub key: Option<String>,
    pub key_code: Option<u32>,
}

impl KeyEvent {
    pub fn current() -> PyResult<Self> {
        Ok(Self {
            key: key()?,
            key_code: key_code()?,
        })
    }
}

#[pymethods]
impl KeyEvent {
    fn __repr__(&self) -> String {
        format!("KeyEvent(key={:?}, key_code={:?})", self.key, self.key_code)
    }
}
