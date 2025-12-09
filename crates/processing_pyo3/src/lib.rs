mod glfw;
mod graphics;

use graphics::{get_graphics, get_graphics_mut, Graphics};
use pyo3::prelude::*;
use pyo3::types::PyAny;

#[pymodule]
fn processing(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Graphics>()?;
    m.add_function(wrap_pyfunction!(size, m)?)?;
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(background, m)?)?;
    m.add_function(wrap_pyfunction!(fill, m)?)?;
    m.add_function(wrap_pyfunction!(no_fill, m)?)?;
    m.add_function(wrap_pyfunction!(stroke, m)?)?;
    m.add_function(wrap_pyfunction!(no_stroke, m)?)?;
    m.add_function(wrap_pyfunction!(stroke_weight, m)?)?;
    m.add_function(wrap_pyfunction!(rect, m)?)?;
    Ok(())
}

#[pyfunction]
#[pyo3(pass_module)]
fn size(module: &Bound<'_, PyModule>, width: u32, height: u32) -> PyResult<()> {
    let graphics = Graphics::new(width, height)?;
    module.setattr("_graphics", graphics)?;
    Ok(())
}

#[pyfunction]
#[pyo3(pass_module, signature = (draw_fn=None))]
fn run(module: &Bound<'_, PyModule>, draw_fn: Option<Py<PyAny>>) -> PyResult<()> {
    get_graphics_mut(module)?.run(draw_fn)
}

#[pyfunction]
#[pyo3(pass_module, signature = (*args))]
fn background(module: &Bound<'_, PyModule>, args: Vec<f32>) -> PyResult<()> {
    get_graphics(module)?.background(args)
}

#[pyfunction]
#[pyo3(pass_module, signature = (*args))]
fn fill(module: &Bound<'_, PyModule>, args: Vec<f32>) -> PyResult<()> {
    get_graphics(module)?.fill(args)
}

#[pyfunction]
#[pyo3(pass_module)]
fn no_fill(module: &Bound<'_, PyModule>) -> PyResult<()> {
    get_graphics(module)?.no_fill()
}

#[pyfunction]
#[pyo3(pass_module, signature = (*args))]
fn stroke(module: &Bound<'_, PyModule>, args: Vec<f32>) -> PyResult<()> {
    get_graphics(module)?.stroke(args)
}

#[pyfunction]
#[pyo3(pass_module)]
fn no_stroke(module: &Bound<'_, PyModule>) -> PyResult<()> {
    get_graphics(module)?.no_stroke()
}

#[pyfunction]
#[pyo3(pass_module)]
fn stroke_weight(module: &Bound<'_, PyModule>, weight: f32) -> PyResult<()> {
    get_graphics(module)?.stroke_weight(weight)
}

#[pyfunction]
#[pyo3(pass_module, signature = (x, y, w, h, tl=0.0, tr=0.0, br=0.0, bl=0.0))]
fn rect(module: &Bound<'_, PyModule>, x: f32, y: f32, w: f32, h: f32, tl: f32, tr: f32, br: f32, bl: f32) -> PyResult<()> {
    get_graphics(module)?.rect(x, y, w, h, tl, tr, br, bl)
}
