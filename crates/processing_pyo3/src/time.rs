use pyo3::{exceptions::PyRuntimeError, prelude::*};

pub fn frame_count() -> PyResult<u32> {
    processing::prelude::frame_count().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn delta_time() -> PyResult<f32> {
    processing::prelude::delta_time().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn elapsed_time() -> PyResult<f32> {
    processing::prelude::elapsed_time().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn sync_globals(globals: &Bound<'_, PyAny>) -> PyResult<()> {
    crate::set_tracked(globals, "frame_count", frame_count()?)?;
    crate::set_tracked(globals, "delta_time", delta_time()?)?;
    crate::set_tracked(globals, "elapsed_time", elapsed_time()?)?;
    Ok(())
}
