//! Host API for applications that embed these bindings
//! and drive several sketches over the one app.
//!
//! Two rules make multiple host binaries and multiple sketches coexist:
//!
//! 1. All engine access goes through this module. The app lives in a
//!    thread-local of whichever binary registered the lib in sys.modules.
//!    A second binary that links its own copy of libprocessing must not call
//!    it directly but call these functions on the registered module instead.
//!    Entities cross as raw bits.
//! 2. Each sketch has a [`SketchContext`]. The canvas graphics, extra
//!    windows, `loop()`/`no_loop()` state, the tracked-globals cache and the
//!    frame counter are otherwise process-global. The host enters a sketch's
//!    context before touching it and exits after.

use std::collections::HashMap;

use bevy::prelude::Entity;
use processing::prelude::*;
use processing_render::geometry::AttributeFormat;
use pyo3::buffer::PyBuffer;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::{LAST_GLOBALS, LOOP_STATE, LoopState};

fn err(e: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(format!("{e}"))
}

fn bytes_of(data: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    let buffer = PyBuffer::<u8>::get(data)?;
    buffer.to_vec(data.py())
}

/// Per-sketch state swapped in and out by `_context_enter` / `_context_exit`.
#[pyclass(unsendable)]
pub struct SketchContext {
    graphics: Option<Py<PyAny>>,
    windows: Option<Py<PyAny>>,
    loop_state: LoopState,
    last_globals: HashMap<&'static str, Py<PyAny>>,
    frame_count: u32,
    entered: bool,
}

pub(crate) fn context_new() -> SketchContext {
    SketchContext {
        graphics: None,
        windows: None,
        loop_state: LoopState::default(),
        last_globals: HashMap::new(),
        frame_count: 0,
        entered: false,
    }
}

fn take_attr(module: &Bound<'_, PyModule>, name: &str) -> PyResult<Option<Py<PyAny>>> {
    let value = match module.getattr(name) {
        Ok(v) if !v.is_none() => Some(v.unbind()),
        _ => None,
    };
    module.setattr(name, module.py().None())?;
    Ok(value)
}

pub(crate) fn context_enter(module: &Bound<'_, PyModule>, ctx: &mut SketchContext) -> PyResult<()> {
    if ctx.entered {
        return Err(PyRuntimeError::new_err("sketch context entered twice"));
    }
    let py = module.py();
    module.setattr(
        "_graphics",
        ctx.graphics.take().unwrap_or_else(|| py.None()),
    )?;
    module.setattr("_windows", ctx.windows.take().unwrap_or_else(|| py.None()))?;
    LOOP_STATE.with(|s| s.set(ctx.loop_state));
    LAST_GLOBALS.with(|c| *c.borrow_mut() = std::mem::take(&mut ctx.last_globals));
    // The app may not exist yet (first sketch, before `size()`).
    let _ = set_frame_count(ctx.frame_count);
    ctx.entered = true;
    Ok(())
}

pub(crate) fn context_exit(module: &Bound<'_, PyModule>, ctx: &mut SketchContext) -> PyResult<()> {
    if !ctx.entered {
        return Ok(());
    }
    ctx.graphics = take_attr(module, "_graphics")?;
    ctx.windows = take_attr(module, "_windows")?;
    ctx.loop_state = LOOP_STATE.with(|s| s.replace(LoopState::default()));
    ctx.last_globals = LAST_GLOBALS.with(|c| std::mem::take(&mut *c.borrow_mut()));
    ctx.frame_count = frame_count().unwrap_or(ctx.frame_count);
    ctx.entered = false;
    Ok(())
}

pub(crate) fn readback<'py>(
    py: Python<'py>,
    graphics: u64,
) -> PyResult<(Bound<'py, PyBytes>, u32, u32, &'static str)> {
    let frame = graphics_readback_raw(Entity::from_bits(graphics)).map_err(err)?;
    let format = match frame.format {
        TextureFormat::Rgba8UnormSrgb => "rgba8_srgb",
        TextureFormat::Rgba8Unorm => "rgba8",
        TextureFormat::Bgra8UnormSrgb => "bgra8_srgb",
        TextureFormat::Bgra8Unorm => "bgra8",
        TextureFormat::Rgba16Float => "rgba16f",
        TextureFormat::Rgba32Float => "rgba32f",
        other => {
            return Err(PyValueError::new_err(format!(
                "unsupported canvas format {other:?}"
            )));
        }
    };
    Ok((
        PyBytes::new(py, &frame.bytes),
        frame.width,
        frame.height,
        format,
    ))
}

pub(crate) fn mouse_move(surface: u64, x: f32, y: f32) -> PyResult<()> {
    input_set_mouse_move(Entity::from_bits(surface), x, y).map_err(err)
}

pub(crate) fn mouse_button(surface: u64, button: u8, pressed: bool) -> PyResult<()> {
    let button = match button {
        0 => MouseButton::Left,
        1 => MouseButton::Middle,
        2 => MouseButton::Right,
        other => {
            return Err(PyValueError::new_err(format!(
                "unknown mouse button {other}"
            )));
        }
    };
    input_set_mouse_button(Entity::from_bits(surface), button, pressed).map_err(err)
}

pub(crate) fn scroll(surface: u64, x: f32, y: f32) -> PyResult<()> {
    input_set_scroll(Entity::from_bits(surface), x, y).map_err(err)
}

pub(crate) fn flush_input() -> PyResult<()> {
    input_flush().map_err(err)
}

pub(crate) fn image_update(image: u64, data: &Bound<'_, PyAny>) -> PyResult<()> {
    image_update_raw(Entity::from_bits(image), &bytes_of(data)?).map_err(err)
}

pub(crate) fn capacity(particles: u64) -> PyResult<u32> {
    particles_capacity(Entity::from_bits(particles)).map_err(err)
}

pub(crate) fn attributes(particles: u64) -> PyResult<Vec<(String, u32, u64)>> {
    Ok(particles_attributes(Entity::from_bits(particles))
        .map_err(err)?
        .into_iter()
        .map(|(name, format, buffer)| (name, format.components() as u32, buffer.to_bits()))
        .collect())
}

pub(crate) fn attribute_buffer(particles: u64, name: String, components: u32) -> PyResult<u64> {
    let format = match components {
        1 => AttributeFormat::Float,
        2 => AttributeFormat::Float2,
        3 => AttributeFormat::Float3,
        4 => AttributeFormat::Float4,
        other => {
            return Err(PyValueError::new_err(format!(
                "attributes have 1..4 components, got {other}"
            )));
        }
    };
    let attribute = geometry_attribute_create(name, format).map_err(err)?;
    let buffer =
        particles_ensure_attribute(Entity::from_bits(particles), attribute).map_err(err)?;
    Ok(buffer.to_bits())
}

pub(crate) fn read_buffer<'py>(py: Python<'py>, buffer: u64) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = buffer_read(Entity::from_bits(buffer)).map_err(err)?;
    Ok(PyBytes::new(py, &bytes))
}

pub(crate) fn write_buffer(buffer: u64, data: &Bound<'_, PyAny>) -> PyResult<()> {
    buffer_write(Entity::from_bits(buffer), bytes_of(data)?).map_err(err)
}
