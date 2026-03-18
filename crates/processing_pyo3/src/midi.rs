use processing::prelude::*;
use pyo3::{exceptions::PyRuntimeError, prelude::*};

pub fn connect(port: usize) -> PyResult<()> {
    midi_connect(port).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
pub fn disconnect() -> PyResult<()> {
    midi_disconnect().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
pub fn refresh_ports() -> PyResult<()> {
    midi_refresh_ports().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
pub fn list_ports() -> PyResult<Vec<String>> {
    midi_list_ports().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
pub fn play_notes(note: u8, duration: u64) -> PyResult<()> {
    midi_play_notes(note, duration).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
