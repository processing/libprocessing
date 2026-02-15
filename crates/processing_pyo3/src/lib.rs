//! # processing_pyo3
//!
//! A Python module that exposes libprocessing using pyo3.

//! In processing4 Java, the sketch runs implicitly inside a class that extends PApplet and
//! executes main. This means that all PAplet methods can be called directly without an explicit
//! receiver.
//!
//! To allow Python users to create a similar experience, we provide module-level
//! functions that forward to a singleton Graphics object pub(crate) behind the scenes.
mod glfw;
mod graphics;

use graphics::{Geometry, Graphics, Image, Topology, get_graphics, get_graphics_mut};
use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyDict, PyTuple},
};
use std::ffi::{CStr, CString};

use std::env;

#[pymodule]
fn processing(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Graphics>()?;
    m.add_class::<Image>()?;
    m.add_class::<Geometry>()?;
    m.add_class::<Topology>()?;
    m.add_function(wrap_pyfunction!(size, m)?)?;
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(mode_3d, m)?)?;
    m.add_function(wrap_pyfunction!(camera_position, m)?)?;
    m.add_function(wrap_pyfunction!(camera_look_at, m)?)?;
    m.add_function(wrap_pyfunction!(push_matrix, m)?)?;
    m.add_function(wrap_pyfunction!(pop_matrix, m)?)?;
    m.add_function(wrap_pyfunction!(rotate, m)?)?;
    m.add_function(wrap_pyfunction!(draw_box, m)?)?;
    m.add_function(wrap_pyfunction!(background, m)?)?;
    m.add_function(wrap_pyfunction!(fill, m)?)?;
    m.add_function(wrap_pyfunction!(no_fill, m)?)?;
    m.add_function(wrap_pyfunction!(stroke, m)?)?;
    m.add_function(wrap_pyfunction!(no_stroke, m)?)?;
    m.add_function(wrap_pyfunction!(stroke_weight, m)?)?;
    m.add_function(wrap_pyfunction!(rect, m)?)?;
    m.add_function(wrap_pyfunction!(image, m)?)?;
    m.add_function(wrap_pyfunction!(draw_geometry, m)?)?;

    Ok(())
}

fn get_asset_root() -> PyResult<String> {
    if let Ok(val) = env::var("PROCESSING_ASSET_ROOT") {
        return Ok(val);
    }

    Python::attach(|py| {
        let sys = PyModule::import(py, "sys")?;
        let argv: Vec<String> = sys.getattr("argv")?.extract()?;
        let filename: &str = argv[0].as_str();
        let os = PyModule::import(py, "os")?;
        let path = os.getattr("path")?;
        let dirname = path.getattr("dirname")?.call1((filename,))?;
        let abspath = path.getattr("abspath")?.call1((dirname,))?;
        let asset_root = path
            .getattr("join")?
            .call1((abspath, "assets"))?
            .to_string();
        Ok(asset_root)
    })
}

fn get_sketch_info() -> PyResult<(String, String)> {
    Python::attach(|py| {
        let sys = PyModule::import(py, "sys")?;
        let argv: Vec<String> = sys.getattr("argv")?.extract()?;
        let filename: &str = argv[0].as_str();
        let os = PyModule::import(py, "os")?;
        let path = os.getattr("path")?;
        let dirname = path.getattr("dirname")?.call1((filename,))?;
        let abspath = path.getattr("abspath")?.call1((dirname,))?;
        let basename = path.getattr("basename")?.call1((filename,))?;
        Ok((abspath.to_string(), basename.to_string()))
    })
}

#[pyfunction]
#[pyo3(pass_module)]
fn size(module: &Bound<'_, PyModule>, width: u32, height: u32) -> PyResult<()> {
    let asset_path: String = get_asset_root()?;
    let (sketch_root_path, sketch_filename) = get_sketch_info()?;
    // let sketch_root_path: String = get_sketch_root()?;
    // let sketch_filename: String = get_sketch_filename()?;
    let graphics = Graphics::new(
        width,
        height,
        asset_path.as_str(),
        sketch_root_path.as_str(),
        sketch_filename.as_str(),
    )?;
    module.setattr("_graphics", graphics)?;
    Ok(())
}

#[pyfunction]
#[pyo3(pass_module)]
fn run(module: &Bound<'_, PyModule>) -> PyResult<()> {
    Python::attach(|py| {
        let builtins = PyModule::import(py, "builtins")?;
        let locals = builtins.getattr("locals")?.call0()?;

        let mut setup_fn = locals.get_item("setup")?;
        let mut draw_fn = locals.get_item("draw")?;

        // call setup
        setup_fn.call0()?;

        // start draw loop
        loop {
            {
                let mut graphics = get_graphics_mut(module)?;

                // TODO: this shouldn't be on the graphics object
                let sketch = graphics.poll_for_sketch_update()?;
                if !sketch.source.is_empty() {
                    let locals = PyDict::new(py);

                    let ok = CString::new(sketch.source.as_str()).unwrap();
                    let cstr: &CStr = ok.as_c_str();

                    match py.run(cstr, None, Some(&locals)) {
                        Ok(_) => {
                            dbg!("Success of any kind?");
                        }
                        Err(e) => {
                            dbg!(e);
                        }
                    }

                    setup_fn = locals.get_item("setup").unwrap().unwrap();
                    draw_fn = locals.get_item("draw").unwrap().unwrap();

                    dbg!(locals);
                }

                if !graphics.surface.poll_events() {
                    break;
                }
                graphics.begin_draw()?;
            }

            draw_fn
                .call0()
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

            get_graphics(module)?.end_draw()?;
        }

        Ok(())
    })
}

#[pyfunction]
#[pyo3(pass_module)]
fn mode_3d(module: &Bound<'_, PyModule>) -> PyResult<()> {
    get_graphics(module)?.mode_3d()
}

#[pyfunction]
#[pyo3(pass_module)]
fn camera_position(module: &Bound<'_, PyModule>, x: f32, y: f32, z: f32) -> PyResult<()> {
    get_graphics(module)?.camera_position(x, y, z)
}

#[pyfunction]
#[pyo3(pass_module)]
fn camera_look_at(
    module: &Bound<'_, PyModule>,
    target_x: f32,
    target_y: f32,
    target_z: f32,
) -> PyResult<()> {
    get_graphics(module)?.camera_look_at(target_x, target_y, target_z)
}

#[pyfunction]
#[pyo3(pass_module)]
fn push_matrix(module: &Bound<'_, PyModule>) -> PyResult<()> {
    get_graphics(module)?.push_matrix()
}

#[pyfunction]
#[pyo3(pass_module)]
fn pop_matrix(module: &Bound<'_, PyModule>) -> PyResult<()> {
    get_graphics(module)?.push_matrix()
}

#[pyfunction]
#[pyo3(pass_module)]
fn rotate(module: &Bound<'_, PyModule>, angle: f32) -> PyResult<()> {
    get_graphics(module)?.rotate(angle)
}

#[pyfunction]
#[pyo3(pass_module)]
fn draw_box(module: &Bound<'_, PyModule>, x: f32, y: f32, z: f32) -> PyResult<()> {
    get_graphics(module)?.draw_box(x, y, z)
}

#[pyfunction]
#[pyo3(pass_module, signature = (geometry))]
fn draw_geometry(module: &Bound<'_, PyModule>, geometry: &Bound<'_, Geometry>) -> PyResult<()> {
    get_graphics(module)?.draw_geometry(&*geometry.extract::<PyRef<Geometry>>()?)
}

#[pyfunction]
#[pyo3(pass_module, signature = (*args))]
fn background(module: &Bound<'_, PyModule>, args: &Bound<'_, PyTuple>) -> PyResult<()> {
    let first = args.get_item(0)?;
    if first.is_instance_of::<Image>() {
        get_graphics(module)?.background_image(&*first.extract::<PyRef<Image>>()?)
    } else {
        get_graphics(module)?.background(args.extract()?)
    }
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
fn rect(
    module: &Bound<'_, PyModule>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    tl: f32,
    tr: f32,
    br: f32,
    bl: f32,
) -> PyResult<()> {
    get_graphics(module)?.rect(x, y, w, h, tl, tr, br, bl)
}

#[pyfunction]
#[pyo3(pass_module, signature = (image_file))]
fn image(module: &Bound<'_, PyModule>, image_file: &str) -> PyResult<Image> {
    get_graphics(module)?.image(image_file)
}
