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
#[cfg(feature = "cuda")]
pub(crate) mod cuda;
mod glfw;
mod gltf;
mod graphics;
mod input;
pub(crate) mod material;
pub(crate) mod math;
mod midi;
mod monitor;
pub(crate) mod shader;
mod surface;
mod time;
#[cfg(feature = "webcam")]
mod webcam;

use graphics::{
    Geometry, Graphics, Image, Light, PyBlendMode, Topology, get_graphics, get_graphics_mut,
};
use material::Material;

use pyo3::{
    BoundObject,
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyDict, PyTuple},
};
use shader::Shader;
use std::ffi::{CStr, CString};

use bevy::log::warn;
use gltf::Gltf;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env;

#[derive(Clone, Copy)]
struct LoopState {
    looping: bool,
    redraw_requested: bool,
}

impl Default for LoopState {
    fn default() -> Self {
        Self {
            looping: true,
            redraw_requested: true,
        }
    }
}

thread_local! {
    static LAST_GLOBALS: RefCell<HashMap<&'static str, Py<PyAny>>> = RefCell::new(HashMap::new());
    static LOOP_STATE: Cell<LoopState> = Cell::new(LoopState::default());
}

fn update_loop_state(f: impl FnOnce(&mut LoopState)) {
    LOOP_STATE.with(|s| {
        let mut state = s.get();
        f(&mut state);
        s.set(state);
    });
}

/// Writes a new value to globals, iff the new value does not match a previous tracked value.
pub(crate) fn set_tracked<'py, V>(
    globals: &Bound<'py, PyAny>,
    name: &'static str,
    new_value: V,
) -> PyResult<()>
where
    V: IntoPyObject<'py>,
    PyErr: From<V::Error>,
{
    let py = globals.py();
    let owned: Py<PyAny> = new_value.into_pyobject(py)?.into_any().unbind();

    let user_shadowed = LAST_GLOBALS.with(|cache| -> PyResult<bool> {
        let cache = cache.borrow();
        let Some(last) = cache.get(name) else {
            return Ok(false);
        };
        match globals.get_item(name) {
            Ok(current) => Ok(!current.eq(last.bind(py))?),
            // key isn't in globals, either because the dict is fresh (livecode reload etc)
            // or the user deleted it so we can safely repopulate
            Err(_) => Ok(false),
        }
    })?;

    if !user_shadowed {
        globals.set_item(name, owned.clone_ref(py))?;
        LAST_GLOBALS.with(|cache| {
            cache.borrow_mut().insert(name, owned);
        });
    }

    Ok(())
}

pub(crate) fn reset_tracked_globals() {
    LAST_GLOBALS.with(|cache| cache.borrow_mut().clear());
}

fn sync_globals(module: &Bound<'_, PyModule>, globals: &Bound<'_, PyAny>) -> PyResult<()> {
    let graphics =
        get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
    input::sync_globals(
        globals,
        graphics.surface.entity,
        graphics.width,
        graphics.height,
    )?;
    surface::sync_globals(globals, &graphics.surface, graphics.width, graphics.height)?;
    time::sync_globals(globals)?;
    Ok(())
}

fn try_call(locals: &Bound<'_, PyAny>, name: &str) -> PyResult<()> {
    if let Ok(cb) = locals.get_item(name)
        && cb.is_callable()
    {
        cb.call0()
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    }
    Ok(())
}

fn dispatch_event_callbacks(locals: &Bound<'_, PyAny>) -> PyResult<()> {
    use processing::prelude::*;
    let err =
        |e: processing::prelude::error::ProcessingError| PyRuntimeError::new_err(format!("{e}"));

    if input_mouse_any_just_pressed().map_err(err)? {
        try_call(locals, "mouse_pressed")?;
    }
    if input_mouse_any_just_released().map_err(err)? {
        try_call(locals, "mouse_released")?;
    }
    if input_mouse_moved().map_err(err)? {
        if input_mouse_is_pressed().map_err(err)? {
            try_call(locals, "mouse_dragged")?;
        } else {
            try_call(locals, "mouse_moved")?;
        }
    }
    if input_mouse_scrolled().map_err(err)? {
        try_call(locals, "mouse_wheel")?;
    }
    if input_key_any_just_pressed().map_err(err)? {
        try_call(locals, "key_pressed")?;
    }
    if input_key_any_just_released().map_err(err)? {
        try_call(locals, "key_released")?;
    }
    Ok(())
}

fn create_graphics_context(module: &Bound<'_, PyModule>, width: u32, height: u32) -> PyResult<()> {
    let py = module.py();
    let env = detect_environment(py)?;

    let interactive = env != "script";
    let log_level = if interactive { Some("error") } else { None };

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
            let graphics = Graphics::new_offscreen(width, height, asset_path.as_str(), log_level)?;
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
                    PyRuntimeError::new_err(format!("Failed to register post-execute hook: {e}"))
                })?;
            }
        }
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

const DEFAULT_WIDTH: u32 = 100;
const DEFAULT_HEIGHT: u32 = 100;

fn ensure_graphics(module: &Bound<'_, PyModule>) -> PyResult<()> {
    if get_graphics(module)?.is_some() {
        return Ok(());
    }
    create_graphics_context(module, DEFAULT_WIDTH, DEFAULT_HEIGHT)
}

macro_rules! graphics {
    ($module:expr) => {{
        ensure_graphics($module)?;
        get_graphics($module)?.expect("ensure_graphics guarantees Some")
    }};
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
    #[cfg(feature = "cuda")]
    #[pymodule_export]
    use super::cuda::CudaImage;
    #[pymodule_export]
    use super::monitor::Monitor;
    #[pymodule_export]
    use super::surface::Surface;

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

    // Shape kinds
    #[pymodule_export]
    const POLYGON: u8 = 0;
    #[pymodule_export]
    const POINTS: u8 = 1;
    #[pymodule_export]
    const LINES: u8 = 2;
    #[pymodule_export]
    const TRIANGLES: u8 = 3;
    #[pymodule_export]
    const TRIANGLE_FAN: u8 = 4;
    #[pymodule_export]
    const TRIANGLE_STRIP: u8 = 5;
    #[pymodule_export]
    const QUADS: u8 = 6;
    #[pymodule_export]
    const QUAD_STRIP: u8 = 7;

    // Shape modes
    #[pymodule_export]
    const CORNER: u8 = 0;
    #[pymodule_export]
    const CORNERS: u8 = 1;
    // CENTER = 1
    #[pymodule_export]
    const RADIUS: u8 = 3;

    // Arc modes
    #[pymodule_export]
    const OPEN: u8 = 0;
    #[pymodule_export]
    const CHORD: u8 = 1;
    #[pymodule_export]
    const PIE: u8 = 2;

    #[pymodule_export]
    const CLOSE: bool = true;

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

    #[pymodule_init]
    fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
        use processing::prelude::BlendMode;

        module.add("width", super::DEFAULT_WIDTH)?;
        module.add("height", super::DEFAULT_HEIGHT)?;

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
    fn flush(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).flush()
    }

    #[cfg(feature = "cuda")]
    #[pyfunction]
    #[pyo3(pass_module)]
    fn cuda(module: &Bound<'_, PyModule>) -> PyResult<crate::cuda::CudaImage> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        graphics.cuda()
    }

    #[cfg(feature = "cuda")]
    #[pyfunction]
    #[pyo3(pass_module)]
    fn update_graphics_from(
        module: &Bound<'_, PyModule>,
        obj: &Bound<'_, pyo3::PyAny>,
    ) -> PyResult<()> {
        graphics!(module).update_from(obj)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn _tick(module: &Bound<'_, PyModule>, ns: &Bound<'_, PyAny>) -> PyResult<()> {
        if get_graphics(module)?.is_none() {
            return Ok(());
        }
        sync_globals(module, ns)?;
        dispatch_event_callbacks(ns)?;
        Ok(())
    }

    #[pyfunction]
    fn redraw() -> PyResult<()> {
        update_loop_state(|s| {
            if !s.looping {
                s.redraw_requested = true;
            }
        });
        Ok(())
    }

    #[pyfunction]
    #[pyo3(name = "loop")]
    fn loop_() -> PyResult<()> {
        update_loop_state(|s| s.looping = true);
        Ok(())
    }

    #[pyfunction]
    fn no_loop() -> PyResult<()> {
        update_loop_state(|s| s.looping = false);
        Ok(())
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn size(module: &Bound<'_, PyModule>, width: u32, height: u32) -> PyResult<()> {
        create_graphics_context(module, width, height)?;

        let py = module.py();
        let sys = PyModule::import(py, "sys")?;
        let frame = sys.getattr("_getframe")?.call1((0,))?;
        let caller_globals = frame.getattr("f_globals")?;
        sync_globals(module, &caller_globals)?;

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

            let setup_fn = locals.get_item("setup").ok();
            let mut draw_fn = locals.get_item("draw").ok();

            if let Some(ref setup) = setup_fn {
                setup.call0()?;
            }

            ensure_graphics(module)?;

            let mut globals = if let Some(ref draw) = draw_fn {
                draw.getattr("__globals__")?
            } else if let Some(ref setup) = setup_fn {
                setup.getattr("__globals__")?
            } else {
                let sys = PyModule::import(py, "sys")?;
                let frame = sys.getattr("_getframe")?.call1((0,))?;
                frame.getattr("f_globals")?
            };
            sync_globals(module, &globals)?;

            // no draw is defined. flush any top level code and then idle
            if draw_fn.is_none() {
                {
                    let mut graphics = get_graphics_mut(module)?
                        .ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
                    graphics.surface.poll_events();
                }

                get_graphics(module)?
                    .ok_or_else(|| PyRuntimeError::new_err("call size() first"))?
                    .end_draw()?;

                loop {
                    {
                        let mut graphics = get_graphics_mut(module)?
                            .ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
                        if !graphics.surface.poll_events() {
                            break;
                        }
                    }
                    dispatch_event_callbacks(&locals)?;
                    std::thread::sleep(std::time::Duration::from_millis(16));
                }

                return Ok(());
            }
            let draw_fn_ref = draw_fn.as_mut().expect("checked above");

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
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("sketch reload error: {e}");
                            }
                        }

                        *draw_fn_ref = locals.get_item("draw").unwrap().unwrap();
                        globals = draw_fn_ref.getattr("__globals__")?;
                        reset_tracked_globals();

                        dbg!(locals);
                    }

                    if !graphics.surface.poll_events() {
                        break;
                    }
                }

                dispatch_event_callbacks(&locals)?;

                let should_draw = LOOP_STATE.with(|s| {
                    let state = s.get();
                    state.looping || state.redraw_requested
                });

                if !should_draw {
                    std::thread::sleep(std::time::Duration::from_millis(16));
                    continue;
                }

                get_graphics_mut(module)?
                    .ok_or_else(|| PyRuntimeError::new_err("call size() first"))?
                    .begin_draw()?;

                processing::prelude::advance_frame_count()
                    .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

                sync_globals(module, &globals)?;

                draw_fn_ref
                    .call0()
                    .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

                get_graphics(module)?
                    .ok_or_else(|| PyRuntimeError::new_err("call size() first"))?
                    .end_draw()?;

                update_loop_state(|s| s.redraw_requested = false);
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
    #[pyo3(pass_module)]
    fn mode_2d(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).mode_2d()
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
    fn orbit_camera(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).orbit_camera()
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn free_camera(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).free_camera()
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn pan_camera(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).pan_camera()
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn disable_camera(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).disable_camera()
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn camera_distance(module: &Bound<'_, PyModule>, distance: f32) -> PyResult<()> {
        graphics!(module).camera_distance(distance)
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (*args))]
    fn camera_center(module: &Bound<'_, PyModule>, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        graphics!(module).camera_center(args)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn camera_min_distance(module: &Bound<'_, PyModule>, min: f32) -> PyResult<()> {
        graphics!(module).camera_min_distance(min)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn camera_max_distance(module: &Bound<'_, PyModule>, max: f32) -> PyResult<()> {
        graphics!(module).camera_max_distance(max)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn camera_speed(module: &Bound<'_, PyModule>, speed: f32) -> PyResult<()> {
        graphics!(module).camera_speed(speed)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn camera_reset(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).camera_reset()
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
    #[pyo3(pass_module, signature = (*args))]
    fn translate(module: &Bound<'_, PyModule>, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        graphics!(module).translate(args)
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
    fn create_image(module: &Bound<'_, PyModule>, width: u32, height: u32) -> PyResult<Image> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        graphics.create_image(width, height)
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

    #[pyfunction]
    #[pyo3(pass_module)]
    fn ellipse(module: &Bound<'_, PyModule>, cx: f32, cy: f32, w: f32, h: f32) -> PyResult<()> {
        graphics!(module).ellipse(cx, cy, w, h)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn circle(module: &Bound<'_, PyModule>, cx: f32, cy: f32, d: f32) -> PyResult<()> {
        graphics!(module).circle(cx, cy, d)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn line(module: &Bound<'_, PyModule>, x1: f32, y1: f32, x2: f32, y2: f32) -> PyResult<()> {
        graphics!(module).line(x1, y1, x2, y2)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn triangle(
        module: &Bound<'_, PyModule>,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
    ) -> PyResult<()> {
        graphics!(module).triangle(x1, y1, x2, y2, x3, y3)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn quad(
        module: &Bound<'_, PyModule>,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x4: f32,
        y4: f32,
    ) -> PyResult<()> {
        graphics!(module).quad(x1, y1, x2, y2, x3, y3, x4, y4)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn point(module: &Bound<'_, PyModule>, x: f32, y: f32) -> PyResult<()> {
        graphics!(module).point(x, y)
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (x, y, s))]
    fn square(module: &Bound<'_, PyModule>, x: f32, y: f32, s: f32) -> PyResult<()> {
        graphics!(module).square(x, y, s)
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (cx, cy, w, h, start, stop, mode=0))]
    fn arc(
        module: &Bound<'_, PyModule>,
        cx: f32,
        cy: f32,
        w: f32,
        h: f32,
        start: f32,
        stop: f32,
        mode: u8,
    ) -> PyResult<()> {
        graphics!(module).arc(cx, cy, w, h, start, stop, mode)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn bezier(
        module: &Bound<'_, PyModule>,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x4: f32,
        y4: f32,
    ) -> PyResult<()> {
        graphics!(module).bezier(x1, y1, x2, y2, x3, y3, x4, y4)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn curve(
        module: &Bound<'_, PyModule>,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x4: f32,
        y4: f32,
    ) -> PyResult<()> {
        graphics!(module).curve(x1, y1, x2, y2, x3, y3, x4, y4)
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (kind=0))]
    fn begin_shape(module: &Bound<'_, PyModule>, kind: u8) -> PyResult<()> {
        graphics!(module).begin_shape(kind)
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (close=false))]
    fn end_shape(module: &Bound<'_, PyModule>, close: bool) -> PyResult<()> {
        graphics!(module).end_shape(close)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn vertex(module: &Bound<'_, PyModule>, x: f32, y: f32) -> PyResult<()> {
        graphics!(module).vertex(x, y)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn bezier_vertex(
        module: &Bound<'_, PyModule>,
        cx1: f32,
        cy1: f32,
        cx2: f32,
        cy2: f32,
        x: f32,
        y: f32,
    ) -> PyResult<()> {
        graphics!(module).bezier_vertex(cx1, cy1, cx2, cy2, x, y)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn quadratic_vertex(
        module: &Bound<'_, PyModule>,
        cx: f32,
        cy: f32,
        x: f32,
        y: f32,
    ) -> PyResult<()> {
        graphics!(module).quadratic_vertex(cx, cy, x, y)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn curve_vertex(module: &Bound<'_, PyModule>, x: f32, y: f32) -> PyResult<()> {
        graphics!(module).curve_vertex(x, y)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn begin_contour(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).begin_contour()
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn end_contour(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).end_contour()
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn reset_matrix(module: &Bound<'_, PyModule>) -> PyResult<()> {
        graphics!(module).reset_matrix()
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (*args))]
    fn scale(module: &Bound<'_, PyModule>, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        graphics!(module).scale(args)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn shear_x(module: &Bound<'_, PyModule>, angle: f32) -> PyResult<()> {
        graphics!(module).shear_x(angle)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn shear_y(module: &Bound<'_, PyModule>, angle: f32) -> PyResult<()> {
        graphics!(module).shear_y(angle)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn rect_mode(module: &Bound<'_, PyModule>, mode: u8) -> PyResult<()> {
        graphics!(module).rect_mode(mode)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn ellipse_mode(module: &Bound<'_, PyModule>, mode: u8) -> PyResult<()> {
        graphics!(module).ellipse_mode(mode)
    }

    #[pyfunction(name = "cylinder")]
    #[pyo3(pass_module, signature = (radius, height, detail=24))]
    fn draw_cylinder(
        module: &Bound<'_, PyModule>,
        radius: f32,
        height: f32,
        detail: u32,
    ) -> PyResult<()> {
        graphics!(module).draw_cylinder(radius, height, detail)
    }

    #[pyfunction(name = "cone")]
    #[pyo3(pass_module, signature = (radius, height, detail=24))]
    fn draw_cone(
        module: &Bound<'_, PyModule>,
        radius: f32,
        height: f32,
        detail: u32,
    ) -> PyResult<()> {
        graphics!(module).draw_cone(radius, height, detail)
    }

    #[pyfunction(name = "torus")]
    #[pyo3(pass_module, signature = (radius, tube_radius, major_segments=24, minor_segments=16))]
    fn draw_torus(
        module: &Bound<'_, PyModule>,
        radius: f32,
        tube_radius: f32,
        major_segments: u32,
        minor_segments: u32,
    ) -> PyResult<()> {
        graphics!(module).draw_torus(radius, tube_radius, major_segments, minor_segments)
    }

    #[pyfunction(name = "plane")]
    #[pyo3(pass_module)]
    fn draw_plane(module: &Bound<'_, PyModule>, width: f32, height: f32) -> PyResult<()> {
        graphics!(module).draw_plane(width, height)
    }

    #[pyfunction(name = "capsule")]
    #[pyo3(pass_module, signature = (radius, length, detail=24))]
    fn draw_capsule(
        module: &Bound<'_, PyModule>,
        radius: f32,
        length: f32,
        detail: u32,
    ) -> PyResult<()> {
        graphics!(module).draw_capsule(radius, length, detail)
    }

    #[pyfunction(name = "conical_frustum")]
    #[pyo3(pass_module, signature = (radius_top, radius_bottom, height, detail=24))]
    fn draw_conical_frustum(
        module: &Bound<'_, PyModule>,
        radius_top: f32,
        radius_bottom: f32,
        height: f32,
        detail: u32,
    ) -> PyResult<()> {
        graphics!(module).draw_conical_frustum(radius_top, radius_bottom, height, detail)
    }

    #[pyfunction(name = "tetrahedron")]
    #[pyo3(pass_module)]
    fn draw_tetrahedron(module: &Bound<'_, PyModule>, radius: f32) -> PyResult<()> {
        graphics!(module).draw_tetrahedron(radius)
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
        input::mouse_x(graphics.surface.entity, graphics.width)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn mouse_y(module: &Bound<'_, PyModule>) -> PyResult<f32> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        input::mouse_y(graphics.surface.entity, graphics.height)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn pmouse_x(module: &Bound<'_, PyModule>) -> PyResult<f32> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        input::pmouse_x(graphics.surface.entity, graphics.width)
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn pmouse_y(module: &Bound<'_, PyModule>) -> PyResult<f32> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        input::pmouse_y(graphics.surface.entity, graphics.height)
    }

    #[pyfunction]
    fn key_is_down(key_code: u32) -> PyResult<bool> {
        input::key_is_down(key_code)
    }

    #[pyfunction]
    fn key_just_pressed(key_code: u32) -> PyResult<bool> {
        input::key_just_pressed(key_code)
    }

    #[pyfunction]
    #[pyo3(pass_module, signature = (density=None))]
    fn pixel_density<'py>(
        module: &Bound<'py, PyModule>,
        density: Option<f32>,
    ) -> PyResult<Py<PyAny>> {
        let py = module.py();
        match density {
            Some(d) => {
                graphics!(module).surface.set_pixel_density(d)?;
                Ok(py.None())
            }
            None => {
                let graphics = get_graphics(module)?
                    .ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
                let current = graphics.surface.pixel_density()?;
                Ok(current.into_pyobject(py)?.into_any().unbind())
            }
        }
    }

    #[pyfunction]
    #[pyo3(pass_module)]
    fn display_density(module: &Bound<'_, PyModule>) -> PyResult<f32> {
        let graphics =
            get_graphics(module)?.ok_or_else(|| PyRuntimeError::new_err("call size() first"))?;
        graphics.surface.display_density()
    }

    #[pyfunction]
    fn frame_count() -> PyResult<u32> {
        time::frame_count()
    }

    #[pyfunction]
    fn delta_time() -> PyResult<f32> {
        time::delta_time()
    }

    #[pyfunction]
    fn elapsed_time() -> PyResult<f32> {
        time::elapsed_time()
    }

    #[pyfunction]
    fn monitors() -> PyResult<Vec<monitor::Monitor>> {
        monitor::list()
    }

    #[pyfunction]
    fn primary_monitor() -> PyResult<Option<monitor::Monitor>> {
        monitor::primary()
    }
}
