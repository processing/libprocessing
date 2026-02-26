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
mod gltf;
mod graphics;
pub(crate) mod material;

use graphics::{Geometry, Graphics, Image, Light, Topology, get_graphics, get_graphics_mut};
use material::Material;
use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    py_run,
    types::{PyDict, PyTuple},
};
use std::ffi::{CStr, CString};

use gltf::Gltf;
use bevy::log::warn;
use std::env;

#[pymodule]
fn processing(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Graphics>()?;
    m.add_class::<Image>()?;
    m.add_class::<Light>()?;
    m.add_class::<Topology>()?;
    m.add_class::<Material>()?;
    m.add_class::<Gltf>()?;
    m.add_function(wrap_pyfunction!(gltf::load_gltf, m)?)?;
    m.add_function(wrap_pyfunction!(size, m)?)?;
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(_poll_events, m)?)?;
    m.add_function(wrap_pyfunction!(_begin_draw, m)?)?;
    m.add_function(wrap_pyfunction!(_end_draw, m)?)?;
    m.add_function(wrap_pyfunction!(_present, m)?)?;
    m.add_function(wrap_pyfunction!(_readback_png, m)?)?;
    m.add_function(wrap_pyfunction!(redraw, m)?)?;
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
    m.add_function(wrap_pyfunction!(create_directional_light, m)?)?;
    m.add_function(wrap_pyfunction!(create_point_light, m)?)?;
    m.add_function(wrap_pyfunction!(create_spot_light, m)?)?;
    m.add_function(wrap_pyfunction!(draw_sphere, m)?)?;
    m.add_function(wrap_pyfunction!(use_material, m)?)?;
    m.add_function(wrap_pyfunction!(roughness, m)?)?;
    m.add_function(wrap_pyfunction!(metallic, m)?)?;
    m.add_function(wrap_pyfunction!(emissive, m)?)?;
    m.add_function(wrap_pyfunction!(unlit, m)?)?;

    Ok(())
}

fn get_asset_root() -> PyResult<String> {
    if let Ok(val) = env::var("PROCESSING_ASSET_ROOT") {
        return Ok(val);
    }

    Python::attach(|py| {
        let sys = PyModule::import(py, "sys")?;
        let argv: Vec<String> = sys.getattr("argv")?.extract()?;
        let filename = argv.first().map(|s| s.as_str()).unwrap_or("");
        let os = PyModule::import(py, "os")?;
        let path = os.getattr("path")?;

        // in ipython/jupyter argv[0] is weird so we use cwd
        // todo: what is the correct way to get notebook path
        if filename.is_empty() || !path.getattr("isfile")?.call1((filename,))?.is_truthy()? {
            let cwd = os.getattr("getcwd")?.call0()?.to_string();
            let asset_root = path.getattr("join")?.call1((cwd, "assets"))?.to_string();
            return Ok(asset_root);
        }

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
        let filename = argv.first().map(|s| s.as_str()).unwrap_or("");
        let os = PyModule::import(py, "os")?;
        let path = os.getattr("path")?;

        if filename.is_empty() || !path.getattr("isfile")?.call1((filename,))?.is_truthy()? {
            let cwd = os.getattr("getcwd")?.call0()?.to_string();
            return Ok((cwd, String::new()));
        }

        let dirname = path.getattr("dirname")?.call1((filename,))?;
        let abspath = path.getattr("abspath")?.call1((dirname,))?;
        let basename = path.getattr("basename")?.call1((filename,))?;
        Ok((abspath.to_string(), basename.to_string()))
    })
}

#[pyfunction]
#[pyo3(pass_module)]
fn _poll_events(module: &Bound<'_, PyModule>) -> PyResult<bool> {
    let mut graphics = get_graphics_mut(module)?;
    Ok(graphics.surface.poll_events())
}

#[pyfunction]
#[pyo3(pass_module)]
fn _begin_draw(module: &Bound<'_, PyModule>) -> PyResult<()> {
    get_graphics(module)?.begin_draw()
}

#[pyfunction]
#[pyo3(pass_module)]
fn _end_draw(module: &Bound<'_, PyModule>) -> PyResult<()> {
    get_graphics(module)?.end_draw()
}

#[pyfunction]
#[pyo3(pass_module)]
fn _present(module: &Bound<'_, PyModule>) -> PyResult<()> {
    get_graphics(module)?.present()
}

#[pyfunction]
#[pyo3(pass_module)]
fn _readback_png(module: &Bound<'_, PyModule>) -> PyResult<Vec<u8>> {
    get_graphics(module)?.readback_png()
}

#[pyfunction]
#[pyo3(pass_module)]
fn redraw(module: &Bound<'_, PyModule>) -> PyResult<()> {
    get_graphics(module)?.present()
}

const DETECT_ENV_CODE: &str = include_str!("python/detect_env.py");
const REGISTER_INPUTHOOK_CODE: &str = include_str!("python/register_inputhook.py");
const IPYTHON_POST_EXECUTE_CODE: &str = include_str!("python/ipython_post_execute.py");
const JUPYTER_POST_EXECUTE_CODE: &str = include_str!("python/jupyter_post_execute.py");

fn detect_environment(py: Python<'_>) -> PyResult<String> {
    let locals = PyDict::new(py);
    let code = CString::new(DETECT_ENV_CODE)?;
    py.run(code.as_c_str(), None, Some(&locals))?;
    locals
        .get_item("_env")?
        .ok_or_else(|| PyRuntimeError::new_err("Failed to detect environment"))?
        .extract()
}

#[pyfunction]
#[pyo3(pass_module)]
fn size(module: &Bound<'_, PyModule>, width: u32, height: u32) -> PyResult<()> {
    let py = module.py();
    let env = detect_environment(py)?;

    let interactive = env != "script";
    let log_level = if interactive { Some("error") } else { None };

    match env.as_str() {
        "jupyter" => {
            let asset_path = get_asset_root()?;
            let graphics = Graphics::new_offscreen(width, height, asset_path.as_str(), log_level)?;
            module.setattr("_graphics", graphics)?;

            let code = CString::new(JUPYTER_POST_EXECUTE_CODE)?;
            py.run(code.as_c_str(), None, None).map_err(|e| {
                PyRuntimeError::new_err(format!("Failed to register Jupyter hooks: {e}"))
            })?;
        }
        "ipython" => {
            let asset_path = get_asset_root()?;
            let (sketch_root, sketch_file) = get_sketch_info()?;
            let graphics = Graphics::new(
                width,
                height,
                asset_path.as_str(),
                sketch_root.as_str(),
                sketch_file.as_str(),
                log_level,
            )?;
            module.setattr("_graphics", graphics)?;

            let hook_code = CString::new(REGISTER_INPUTHOOK_CODE)?;
            py.run(hook_code.as_c_str(), None, None).map_err(|e| {
                PyRuntimeError::new_err(format!("Failed to register inputhook: {e}"))
            })?;

            let post_code = CString::new(IPYTHON_POST_EXECUTE_CODE)?;
            py.run(post_code.as_c_str(), None, None).map_err(|e| {
                PyRuntimeError::new_err(format!("Failed to register post-execute hook: {e}"))
            })?;
        }

        // this is the default "script" mode where we assume the user will call run() to start the draw loop
        _ => {
            let asset_path = get_asset_root()?;
            let (sketch_root, sketch_file) = get_sketch_info()?;
            let graphics = Graphics::new(
                width,
                height,
                asset_path.as_str(),
                sketch_root.as_str(),
                sketch_file.as_str(),
                log_level,
            )?;
            module.setattr("_graphics", graphics)?;
        }
    }

    Ok(())
}

#[pyfunction]
#[pyo3(pass_module)]
fn run(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = module.py();
    let env = detect_environment(py)?;

    if env != "script" {
        warn!("run() was called, but we're in an interactive environment ({env}).");
        return Ok(());
    }

    Python::attach(|py| {
        let builtins = PyModule::import(py, "builtins")?;
        let locals = builtins.getattr("locals")?.call0()?;

        let setup_fn = locals.get_item("setup")?;
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

                    // setup_fn = locals.get_item("setup").unwrap().unwrap();
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

#[pyfunction(name = "box")]
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

#[pyfunction]
#[pyo3(pass_module, signature = (r, g, b, illuminance))]
fn create_directional_light(
    module: &Bound<'_, PyModule>,
    r: f32,
    g: f32,
    b: f32,
    illuminance: f32,
) -> PyResult<Light> {
    get_graphics(module)?.light_directional(r, g, b, illuminance)
}

#[pyfunction]
#[pyo3(pass_module, signature = (r, g, b, intensity, range, radius))]
fn create_point_light(
    module: &Bound<'_, PyModule>,
    r: f32,
    g: f32,
    b: f32,
    intensity: f32,
    range: f32,
    radius: f32,
) -> PyResult<Light> {
    get_graphics(module)?.light_point(r, g, b, intensity, range, radius)
}

#[pyfunction]
#[pyo3(pass_module, signature = (r, g, b, intensity, range, radius, inner_angle, outer_angle))]
fn create_spot_light(
    module: &Bound<'_, PyModule>,
    r: f32,
    g: f32,
    b: f32,
    intensity: f32,
    range: f32,
    radius: f32,
    inner_angle: f32,
    outer_angle: f32,
) -> PyResult<Light> {
    get_graphics(module)?.light_spot(r, g, b, intensity, range, radius, inner_angle, outer_angle)
}

#[pyfunction(name = "sphere")]
#[pyo3(pass_module, signature = (radius, sectors=32, stacks=18))]
fn draw_sphere(
    module: &Bound<'_, PyModule>,
    radius: f32,
    sectors: u32,
    stacks: u32,
) -> PyResult<()> {
    get_graphics(module)?.draw_sphere(radius, sectors, stacks)
}

#[pyfunction]
#[pyo3(pass_module, signature = (material))]
fn use_material(module: &Bound<'_, PyModule>, material: &Bound<'_, Material>) -> PyResult<()> {
    get_graphics(module)?.use_material(&*material.extract::<PyRef<Material>>()?)
}

#[pyfunction]
#[pyo3(pass_module)]
fn roughness(module: &Bound<'_, PyModule>, value: f32) -> PyResult<()> {
    get_graphics(module)?.roughness(value)
}

#[pyfunction]
#[pyo3(pass_module)]
fn metallic(module: &Bound<'_, PyModule>, value: f32) -> PyResult<()> {
    get_graphics(module)?.metallic(value)
}

#[pyfunction]
#[pyo3(pass_module, signature = (*args))]
fn emissive(module: &Bound<'_, PyModule>, args: &Bound<'_, PyTuple>) -> PyResult<()> {
    let args: Vec<f32> = args.extract()?;
    get_graphics(module)?.emissive(args)
}

#[pyfunction]
#[pyo3(pass_module)]
fn unlit(module: &Bound<'_, PyModule>) -> PyResult<()> {
    get_graphics(module)?.unlit()
}
