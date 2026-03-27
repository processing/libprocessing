//! # processing_pyo3
//!
//! A Python module that exposes libprocessing using pyo3.

//! In processing4 Java, the sketch runs implicitly inside a class that extends PApplet and
//! executes main. This means that all PAplet methods can be called directly without an explicit
//! receiver.
//!
//! To allow Python users to create a similar experience, we provide module-level
//! functions that forward to a singleton Graphics object pub(crate) behind the scenes.
pub(crate) mod color;
mod glfw;
mod gltf;
mod graphics;
mod input;
pub(crate) mod material;
pub(crate) mod math;
mod midi;
pub(crate) mod shader;
#[cfg(feature = "webcam")]
mod webcam;

use graphics::{
    Geometry, Graphics, Image, Light, PyBlendMode, Topology, get_graphics, get_graphics_mut,
};
use material::Material;

use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyDict, PyTuple},
};
use shader::Shader;
use std::ffi::{CStr, CString};

use bevy::log::warn;
use gltf::Gltf;
use std::env;

/// Get a shared ref to the Graphics context, or return Ok(()) if not yet initialized.
macro_rules! graphics {
    ($module:expr) => {
        match get_graphics($module)? {
            Some(g) => g,
            None => return Ok(()),
        }
    };
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

#[pymodule]
mod mewnala {
    use super::*;

    #[pymodule_export]
    use super::Geometry;
    #[pymodule_export]
    use super::Gltf;
    #[pymodule_export]
    use super::Graphics;
    #[pymodule_export]
    use super::Image;
    #[pymodule_export]
    use super::Light;
    #[pymodule_export]
    use super::Material;
    #[pymodule_export]
    use super::PyBlendMode;
    #[pymodule_export]
    use super::Shader;
    #[pymodule_export]
    use super::Topology;

    // Stroke cap/join
    #[pymodule_export]
    const ROUND: u8 = 0;
    #[pymodule_export]
    const SQUARE: u8 = 1;
    #[pymodule_export]
    const PROJECT: u8 = 2;
    #[pymodule_export]
    const MITER: u8 = 1;
    #[pymodule_export]
    const BEVEL: u8 = 2;

    // Mouse buttons
    #[pymodule_export]
    const LEFT: u8 = 0;
    #[pymodule_export]
    const CENTER: u8 = 1;
    #[pymodule_export]
    const RIGHT: u8 = 2;

    // Letters
    #[pymodule_export]
    const KEY_A: u32 = 65;
    #[pymodule_export]
    const KEY_B: u32 = 66;
    #[pymodule_export]
    const KEY_C: u32 = 67;
    #[pymodule_export]
    const KEY_D: u32 = 68;
    #[pymodule_export]
    const KEY_E: u32 = 69;
    #[pymodule_export]
    const KEY_F: u32 = 70;
    #[pymodule_export]
    const KEY_G: u32 = 71;
    #[pymodule_export]
    const KEY_H: u32 = 72;
    #[pymodule_export]
    const KEY_I: u32 = 73;
    #[pymodule_export]
    const KEY_J: u32 = 74;
    #[pymodule_export]
    const KEY_K: u32 = 75;
    #[pymodule_export]
    const KEY_L: u32 = 76;
    #[pymodule_export]
    const KEY_M: u32 = 77;
    #[pymodule_export]
    const KEY_N: u32 = 78;
    #[pymodule_export]
    const KEY_O: u32 = 79;
    #[pymodule_export]
    const KEY_P: u32 = 80;
    #[pymodule_export]
    const KEY_Q: u32 = 81;
    #[pymodule_export]
    const KEY_R: u32 = 82;
    #[pymodule_export]
    const KEY_S: u32 = 83;
    #[pymodule_export]
    const KEY_T: u32 = 84;
    #[pymodule_export]
    const KEY_U: u32 = 85;
    #[pymodule_export]
    const KEY_V: u32 = 86;
    #[pymodule_export]
    const KEY_W: u32 = 87;
    #[pymodule_export]
    const KEY_X: u32 = 88;
    #[pymodule_export]
    const KEY_Y: u32 = 89;
    #[pymodule_export]
    const KEY_Z: u32 = 90;

    // Digits
    #[pymodule_export]
    const KEY_0: u32 = 48;
    #[pymodule_export]
    const KEY_1: u32 = 49;
    #[pymodule_export]
    const KEY_2: u32 = 50;
    #[pymodule_export]
    const KEY_3: u32 = 51;
    #[pymodule_export]
    const KEY_4: u32 = 52;
    #[pymodule_export]
    const KEY_5: u32 = 53;
    #[pymodule_export]
    const KEY_6: u32 = 54;
    #[pymodule_export]
    const KEY_7: u32 = 55;
    #[pymodule_export]
    const KEY_8: u32 = 56;
    #[pymodule_export]
    const KEY_9: u32 = 57;

    // Punctuation/symbols
    #[pymodule_export]
    const SPACE: u32 = 32;
    #[pymodule_export]
    const QUOTE: u32 = 39;
    #[pymodule_export]
    const COMMA: u32 = 44;
    #[pymodule_export]
    const MINUS: u32 = 45;
    #[pymodule_export]
    const PERIOD: u32 = 46;
    #[pymodule_export]
    const SLASH: u32 = 47;
    #[pymodule_export]
    const SEMICOLON: u32 = 59;
    #[pymodule_export]
    const EQUAL: u32 = 61;
    #[pymodule_export]
    const BRACKET_LEFT: u32 = 91;
    #[pymodule_export]
    const BACKSLASH: u32 = 92;
    #[pymodule_export]
    const BRACKET_RIGHT: u32 = 93;
    #[pymodule_export]
    const BACKQUOTE: u32 = 96;

    // Navigation/editing
    #[pymodule_export]
    const ESCAPE: u32 = 256;
    #[pymodule_export]
    const ENTER: u32 = 257;
    #[pymodule_export]
    const TAB: u32 = 258;
    #[pymodule_export]
    const BACKSPACE: u32 = 259;
    #[pymodule_export]
    const INSERT: u32 = 260;
    #[pymodule_export]
    const DELETE: u32 = 261;
    #[pymodule_export]
    const UP: u32 = 265;
    #[pymodule_export]
    const DOWN: u32 = 264;
    #[pymodule_export]
    const LEFT_ARROW: u32 = 263;
    #[pymodule_export]
    const RIGHT_ARROW: u32 = 262;
    #[pymodule_export]
    const PAGE_UP: u32 = 266;
    #[pymodule_export]
    const PAGE_DOWN: u32 = 267;
    #[pymodule_export]
    const HOME: u32 = 268;
    #[pymodule_export]
    const END: u32 = 269;

    // Modifiers
    #[pymodule_export]
    const SHIFT: u32 = 340;
    #[pymodule_export]
    const CONTROL: u32 = 341;
    #[pymodule_export]
    const ALT: u32 = 342;
    #[pymodule_export]
    const SUPER: u32 = 343;

    // Function keys
    #[pymodule_export]
    const F1: u32 = 290;
    #[pymodule_export]
    const F2: u32 = 291;
    #[pymodule_export]
    const F3: u32 = 292;
    #[pymodule_export]
    const F4: u32 = 293;
    #[pymodule_export]
    const F5: u32 = 294;
    #[pymodule_export]
    const F6: u32 = 295;
    #[pymodule_export]
    const F7: u32 = 296;
    #[pymodule_export]
    const F8: u32 = 297;
    #[pymodule_export]
    const F9: u32 = 298;
    #[pymodule_export]
    const F10: u32 = 299;
    #[pymodule_export]
    const F11: u32 = 300;
    #[pymodule_export]
    const F12: u32 = 301;

    // color space constants for color_mode()
    #[pymodule_export]
    const SRGB: u8 = 0;
    #[pymodule_export]
    const LINEAR: u8 = 1;
    #[pymodule_export]
    const HSL: u8 = 2;
    #[pymodule_export]
    const HSV: u8 = 3;
    #[pymodule_export]
    const HWB: u8 = 4;
    #[pymodule_export]
    const OKLAB: u8 = 5;
    #[pymodule_export]
    const OKLCH: u8 = 6;
    #[pymodule_export]
    const LAB: u8 = 7;
    #[pymodule_export]
    const LCH: u8 = 8;
    #[pymodule_export]
    const XYZ: u8 = 9;

    // Blend factor constants (for BlendMode custom constructor)
    #[pymodule_export]
    const ZERO: u8 = 0;
    #[pymodule_export]
    const ONE: u8 = 1;
    #[pymodule_export]
    const SRC_COLOR: u8 = 2;
    #[pymodule_export]
    const ONE_MINUS_SRC_COLOR: u8 = 3;
    #[pymodule_export]
    const SRC_ALPHA: u8 = 4;
    #[pymodule_export]
    const ONE_MINUS_SRC_ALPHA: u8 = 5;
    #[pymodule_export]
    const DST_COLOR: u8 = 6;
    #[pymodule_export]
    const ONE_MINUS_DST_COLOR: u8 = 7;
    #[pymodule_export]
    const DST_ALPHA: u8 = 8;
    #[pymodule_export]
    const ONE_MINUS_DST_ALPHA: u8 = 9;
    #[pymodule_export]
    const SRC_ALPHA_SATURATED: u8 = 10;

    // Blend operation constants (for BlendMode custom constructor)
    #[pymodule_export]
    const OP_ADD: u8 = 0;
    #[pymodule_export]
    const OP_SUBTRACT: u8 = 1;
    #[pymodule_export]
    const OP_REVERSE_SUBTRACT: u8 = 2;
    #[pymodule_export]
    const OP_MIN: u8 = 3;
    #[pymodule_export]
    const OP_MAX: u8 = 4;

    // Blend mode preset constants (added in pymodule_init)
    #[pymodule_init]
    fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
        use processing::prelude::BlendMode;
        module.add("BLEND", PyBlendMode::from_preset(BlendMode::Blend))?;
        module.add("ADD", PyBlendMode::from_preset(BlendMode::Add))?;
        module.add("SUBTRACT", PyBlendMode::from_preset(BlendMode::Subtract))?;
        module.add("DARKEST", PyBlendMode::from_preset(BlendMode::Darkest))?;
        module.add("LIGHTEST", PyBlendMode::from_preset(BlendMode::Lightest))?;
        module.add(
            "DIFFERENCE",
            PyBlendMode::from_preset(BlendMode::Difference),
        )?;
        module.add("EXCLUSION", PyBlendMode::from_preset(BlendMode::Exclusion))?;
        module.add("MULTIPLY", PyBlendMode::from_preset(BlendMode::Multiply))?;
        module.add("SCREEN", PyBlendMode::from_preset(BlendMode::Screen))?;
        module.add("REPLACE", PyBlendMode::from_preset(BlendMode::Replace))?;
        Ok(())
    }

    #[pymodule]
    mod math {
        use super::*;

        #[pymodule_export]
        use crate::math::PyQuat;
        #[pymodule_export]
        use crate::math::PyVec2;
        #[pymodule_export]
        use crate::math::PyVec3;
        #[pymodule_export]
        use crate::math::PyVec4;
        #[pymodule_export]
        use crate::math::PyVecIter;

        #[pyfunction]
        #[pyo3(signature = (*args))]
        fn vec2(args: &Bound<'_, PyTuple>) -> PyResult<PyVec2> {
            PyVec2::py_new(args)
        }

        #[pyfunction]
        #[pyo3(signature = (*args))]
        fn vec3(args: &Bound<'_, PyTuple>) -> PyResult<PyVec3> {
            PyVec3::py_new(args)
        }

        #[pyfunction]
        #[pyo3(signature = (*args))]
        fn vec4(args: &Bound<'_, PyTuple>) -> PyResult<PyVec4> {
            PyVec4::py_new(args)
        }

        #[pyfunction]
        #[pyo3(signature = (*args))]
        fn quat(args: &Bound<'_, PyTuple>) -> PyResult<PyQuat> {
            PyQuat::py_new(args)
        }
    }

    #[pymodule]
    mod color {
        use super::*;

        #[pymodule_export]
        use crate::color::PyColor;

        #[pyfunction]
        fn hex(s: &str) -> PyResult<PyColor> {
            PyColor::hex(s)
        }

        #[pyfunction]
        #[pyo3(signature = (r, g, b, a=1.0))]
        fn srgb(r: f32, g: f32, b: f32, a: f32) -> PyColor {
            PyColor::srgb(r, g, b, a)
        }

        #[pyfunction]
        #[pyo3(signature = (r, g, b, a=1.0))]
        fn linear(r: f32, g: f32, b: f32, a: f32) -> PyColor {
            PyColor::linear(r, g, b, a)
        }

        #[pyfunction]
        #[pyo3(signature = (h, s, l, a=1.0))]
        fn hsla(h: f32, s: f32, l: f32, a: f32) -> PyColor {
            PyColor::hsla(h, s, l, a)
        }

        #[pyfunction]
        #[pyo3(signature = (h, s, v, a=1.0))]
        fn hsva(h: f32, s: f32, v: f32, a: f32) -> PyColor {
            PyColor::hsva(h, s, v, a)
        }

        #[pyfunction]
        #[pyo3(signature = (h, w, b, a=1.0))]
        fn hwba(h: f32, w: f32, b: f32, a: f32) -> PyColor {
            PyColor::hwba(h, w, b, a)
        }

        #[pyfunction]
        #[pyo3(signature = (l, a_axis, b_axis, alpha=1.0))]
        fn oklab(l: f32, a_axis: f32, b_axis: f32, alpha: f32) -> PyColor {
            PyColor::oklab(l, a_axis, b_axis, alpha)
        }

        #[pyfunction]
        #[pyo3(signature = (l, c, h, a=1.0))]
        fn oklch(l: f32, c: f32, h: f32, a: f32) -> PyColor {
            PyColor::oklch(l, c, h, a)
        }

        #[pyfunction]
        #[pyo3(signature = (l, a_axis, b_axis, alpha=1.0))]
        fn lab(l: f32, a_axis: f32, b_axis: f32, alpha: f32) -> PyColor {
            PyColor::lab(l, a_axis, b_axis, alpha)
        }

        #[pyfunction]
        #[pyo3(signature = (l, c, h, a=1.0))]
        fn lch(l: f32, c: f32, h: f32, a: f32) -> PyColor {
            PyColor::lch(l, c, h, a)
        }

        #[pyfunction]
        #[pyo3(signature = (x, y, z, a=1.0))]
        fn xyz(x: f32, y: f32, z: f32, a: f32) -> PyColor {
            PyColor::xyz(x, y, z, a)
        }
    }

    #[cfg(feature = "webcam")]
    #[pymodule_export]
    use super::webcam::Webcam;

    #[pyfunction]
    #[pyo3(pass_module)]
    fn load_gltf(module: &Bound<'_, PyModule>, path: &str) -> PyResult<Gltf> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        let entity = ::processing::prelude::gltf_load(graphics.entity, path)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Gltf::from_entity(entity))
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn _poll_events(module: &Bound<'_, PyModule>) -> PyResult<bool> {
        let Some(mut graphics) = get_graphics_mut(module)? else {
            return Ok(true);
        };
        Ok(graphics.surface.poll_events())
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn _begin_draw(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).begin_draw()
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn _end_draw(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).end_draw()
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn _present(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).present()
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn _readback_png(module: &Bound<'_, PyModule>) -> PyResult<Option<Vec<u8>>> {
        let Some(graphics) = get_graphics(module)? else {
            return Ok(None);
        };
        graphics.readback_png().map(Some)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn redraw(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).present()
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn size(module: &Bound<'_, PyModule>, width: u32, height: u32) -> PyResult<()> {
        let py = module.py();
        let env = detect_environment(py)?;

        let interactive = env != "script";
        let log_level = if interactive { Some("error") } else { None };

        // Check if we already have a graphics context (i.e. size() was called before).
        // Drop the old one first so the window and GPU resources are released.
        let has_existing = module
            .getattr("_graphics")
            .ok()
            .map(|a| !a.is_none())
            .unwrap_or(false);
        if has_existing {
            module.setattr("_graphics", py.None())?;
        }

        match env.as_str() {
            "jupyter" => {
                let asset_path = get_asset_root()?;
                let graphics =
                    Graphics::new_offscreen(width, height, asset_path.as_str(), log_level)?;
                module.setattr("_graphics", graphics)?;

                if !has_existing {
                    let code = CString::new(JUPYTER_POST_EXECUTE_CODE)?;
                    py.run(code.as_c_str(), None, None).map_err(|e| {
                        PyRuntimeError::new_err(format!("Failed to register Jupyter hooks: {e}"))
                    })?;
                }
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

                if !has_existing {
                    let hook_code = CString::new(REGISTER_INPUTHOOK_CODE)?;
                    py.run(hook_code.as_c_str(), None, None).map_err(|e| {
                        PyRuntimeError::new_err(format!("Failed to register inputhook: {e}"))
                    })?;

                    let post_code = CString::new(IPYTHON_POST_EXECUTE_CODE)?;
                    py.run(post_code.as_c_str(), None, None).map_err(|e| {
                        PyRuntimeError::new_err(format!(
                            "Failed to register post-execute hook: {e}"
                        ))
                    })?;
                }
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

            {
                let graphics = get_graphics(module)?
                    .ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
                input::sync_globals(&draw_fn, graphics.surface.entity)?;
            }

            // start draw loop
            loop {
                {
                    let mut graphics = get_graphics_mut(module)?
                        .ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;

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

                {
                    let graphics = get_graphics(module)?
                        .ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
                    input::sync_globals(&draw_fn, graphics.surface.entity)?;
                }

                draw_fn
                    .call0()
                    .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

                get_graphics(module)?
                    .ok_or_else(|| PyRuntimeError::new_err("call size() first"))?
                    .end_draw()?;
            }

            Ok(())
        })
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn mode_3d(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).mode_3d()
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (*args))]
    fn camera_position(module: &Bound<'_, PyModule>, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        graphics!(module).camera_position(args)
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (*args))]
    fn camera_look_at(module: &Bound<'_, PyModule>, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        graphics!(module).camera_look_at(args)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn push_matrix(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).push_matrix()
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn pop_matrix(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).pop_matrix()
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn rotate(module: &Bound<'_, PyModule>, angle: f32) -> PyResult<()> {
        graphics!(module).rotate(angle)
    }

    #[pyfunction(name = "box")]
    #[pyo3(pass_module)]
    fn draw_box(module: &Bound<'_, PyModule>, x: f32, y: f32, z: f32) -> PyResult<()> {
        graphics!(module).draw_box(x, y, z)
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (geometry))]
    fn draw_geometry(module: &Bound<'_, PyModule>, geometry: &Bound<'_, Geometry>) -> PyResult<()> {
        graphics!(module).draw_geometry(&*geometry.extract::<PyRef<Geometry>>()?)
    }

    #[pyfunction(name = "color")]
    #[pyo3(pass_module, signature = (*args))]
    fn create_color(
        module: &Bound<'_, PyModule>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<super::color::PyColor> {
        match get_graphics(module)? {
            Some(g) => g.color(args),
            None => {
                let mode = super::color::ColorMode::default();
                super::color::extract_color_with_mode(args, &mode).map(super::color::PyColor::from)
            }
        }
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (*args))]
    fn background(module: &Bound<'_, PyModule>, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let graphics = graphics!(module);
        let first = args.get_item(0)?;
        if first.is_instance_of::<Image>() {
            graphics.background_image(&*first.extract::<PyRef<Image>>()?)
        } else {
            graphics.background(args)
        }
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (mode, max1=None, max2=None, max3=None, max_alpha=None))]
    fn color_mode<'py>(
        module: &Bound<'py, PyModule>,
        mode: u8,
        max1: Option<&Bound<'py, PyAny>>,
        max2: Option<&Bound<'py, PyAny>>,
        max3: Option<&Bound<'py, PyAny>>,
        max_alpha: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<()> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        graphics.set_color_mode(mode, max1, max2, max3, max_alpha)
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (*args))]
    fn fill(module: &Bound<'_, PyModule>, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        graphics!(module).fill(args)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn no_fill(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).no_fill()
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (*args))]
    fn stroke(module: &Bound<'_, PyModule>, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        graphics!(module).stroke(args)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn no_stroke(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).no_stroke()
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn stroke_weight(module: &Bound<'_, PyModule>, weight: f32) -> PyResult<()> {
        graphics!(module).stroke_weight(weight)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn stroke_cap(module: &Bound<'_, PyModule>, cap: u8) -> PyResult<()> {
        graphics!(module).stroke_cap(cap)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn stroke_join(module: &Bound<'_, PyModule>, join: u8) -> PyResult<()> {
        graphics!(module).stroke_join(join)
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (mode))]
    fn blend_mode(module: &Bound<'_, PyModule>, mode: &Bound<'_, PyBlendMode>) -> PyResult<()> {
        graphics!(module).blend_mode(&*mode.extract::<PyRef<PyBlendMode>>()?)
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
        graphics!(module).rect(x, y, w, h, tl, tr, br, bl)
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (image_file))]
    fn image(module: &Bound<'_, PyModule>, image_file: &str) -> PyResult<Image> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        graphics.image(image_file)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn create_directional_light(
        module: &Bound<'_, PyModule>,
        color: super::color::ColorLike,
        illuminance: f32,
    ) -> PyResult<Light> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        graphics.light_directional(color, illuminance)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn create_point_light(
        module: &Bound<'_, PyModule>,
        color: super::color::ColorLike,
        intensity: f32,
        range: f32,
        radius: f32,
    ) -> PyResult<Light> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        graphics.light_point(color, intensity, range, radius)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn create_spot_light(
        module: &Bound<'_, PyModule>,
        color: super::color::ColorLike,
        intensity: f32,
        range: f32,
        radius: f32,
        inner_angle: f32,
        outer_angle: f32,
    ) -> PyResult<Light> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        graphics.light_spot(color, intensity, range, radius, inner_angle, outer_angle)
    }

    #[pyfunction(name = "sphere")]
    #[pyo3(pass_module, signature = (radius, sectors=32, stacks=18))]
    fn draw_sphere(
        module: &Bound<'_, PyModule>,
        radius: f32,
        sectors: u32,
        stacks: u32,
    ) -> PyResult<()> {
        graphics!(module).draw_sphere(radius, sectors, stacks)
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (material))]
    fn use_material(module: &Bound<'_, PyModule>, material: &Bound<'_, Material>) -> PyResult<()> {
        graphics!(module).use_material(&*material.extract::<PyRef<Material>>()?)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn roughness(module: &Bound<'_, PyModule>, value: f32) -> PyResult<()> {
        graphics!(module).roughness(value)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn metallic(module: &Bound<'_, PyModule>, value: f32) -> PyResult<()> {
        graphics!(module).metallic(value)
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (*args))]
    fn emissive(module: &Bound<'_, PyModule>, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        graphics!(module).emissive(args)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn unlit(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).unlit()
    }

    #[cfg(feature = "webcam")]
    #[pyfunction]
    #[pyo3(signature = (width=None, height=None, framerate=None))]
    fn create_webcam(
        width: Option<u32>,
        height: Option<u32>,
        framerate: Option<u32>,
    ) -> PyResult<webcam::Webcam> {
        webcam::Webcam::new(width, height, framerate)
    }

    #[pyfunction]
    fn midi_connect(port: usize) -> PyResult<()> {
        midi::connect(port)
    }
    #[pyfunction]
    fn midi_disconnect() -> PyResult<()> {
        midi::disconnect()
    }
    #[pyfunction]
    fn midi_refresh_ports() -> PyResult<()> {
        midi::refresh_ports()
    }
    #[pyfunction]
    fn midi_list_ports() -> PyResult<Vec<String>> {
        midi::list_ports()
    }
    #[pyfunction]
    fn midi_play_notes(note: u8, duration: u64) -> PyResult<()> {
        midi::play_notes(note, duration)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn mouse_x(module: &Bound<'_, PyModule>) -> PyResult<f32> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        input::mouse_x(graphics.surface.entity)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn mouse_y(module: &Bound<'_, PyModule>) -> PyResult<f32> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        input::mouse_y(graphics.surface.entity)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn pmouse_x(module: &Bound<'_, PyModule>) -> PyResult<f32> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        input::pmouse_x(graphics.surface.entity)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn pmouse_y(module: &Bound<'_, PyModule>) -> PyResult<f32> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        input::pmouse_y(graphics.surface.entity)
    }

    #[pyfunction]
    fn key_is_down(key_code: u32) -> PyResult<bool> {
        input::key_is_down(key_code)
    }
}
