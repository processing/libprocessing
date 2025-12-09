mod graphics;
mod glfw;

use pyo3::prelude::*;
use crate::graphics::{with_graphics, with_graphics_mut};

#[pymodule]
fn processing(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<graphics::Graphics>()?;

    // settings / lifecycle
    m.add_function(wrap_pyfunction!(size, m)?)?;
    m.add_function(wrap_pyfunction!(run, m)?)?;

    // draw state
    m.add_function(wrap_pyfunction!(background, m)?)?;
    m.add_function(wrap_pyfunction!(fill, m)?)?;
    m.add_function(wrap_pyfunction!(no_fill, m)?)?;
    m.add_function(wrap_pyfunction!(stroke, m)?)?;
    m.add_function(wrap_pyfunction!(no_stroke, m)?)?;
    m.add_function(wrap_pyfunction!(stroke_weight, m)?)?;

    // drawing prims
    m.add_function(wrap_pyfunction!(rect, m)?)?;


    Ok(())
}

// these are all our module-level functions
//
// in processing4 java, the sketch runs implicitly inside a class that extends PApplet and
// executes main. here we have do a little magic trick to get similar behavior. if we required
// users to create a Graphics object and call its methods, it would be ugly because we lack
// an implicit receiver like 'this' in java. so instead we create a singleton Graphics object
// behind the scenes and have these module-level functions forward to that object.

#[pyfunction]
fn size(width: u32, height: u32) -> PyResult<()> {
    graphics::create_context(width, height)
}

#[pyfunction]
#[pyo3(signature = (draw_fn=None))]
fn run(draw_fn: Option<Py<PyAny>>) -> PyResult<()> {
    with_graphics_mut(|mut g| g.run(draw_fn))
}


#[pyfunction]
#[pyo3(signature = (*args))]
fn background(args: Vec<f32>) -> PyResult<()> {
    with_graphics(|g| g.background(args.to_vec()))
}

#[pyfunction]
#[pyo3(signature = (*args))]
fn fill(args: Vec<f32>) -> PyResult<()> {
    with_graphics(|g| g.fill(args.to_vec()))
}

#[pyfunction]
fn no_fill() -> PyResult<()> {
    with_graphics(|g| g.no_fill())
}

#[pyfunction]
#[pyo3(signature = (*args))]
fn stroke(args: Vec<f32>) -> PyResult<()> {
    with_graphics(|g| g.stroke(args.to_vec()))
}

#[pyfunction]
fn no_stroke() -> PyResult<()> {
    with_graphics(|g| g.no_stroke())
}

#[pyfunction]
fn stroke_weight(weight: f32) -> PyResult<()> {
    with_graphics(|g| g.stroke_weight(weight))
}

#[pyfunction]
#[pyo3(signature = (x, y, w, h, tl=0.0, tr=0.0, br=0.0, bl=0.0))]
fn rect(x: f32, y: f32, w: f32, h: f32, tl: f32, tr: f32, br: f32, bl: f32) -> PyResult<()> {
    with_graphics(|g| g.rect(x, y, w, h, tl, tr, br, bl))
}
