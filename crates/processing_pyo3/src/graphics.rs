use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::cell::RefCell;

use crate::glfw::GlfwContext;

// The global graphics context
// we use thread_local! to ensure that the context is specific to the current thread, particularly
// becausae glfw requires that all window operations happen on the main thread.
// TODO: we'll want to think about how to enable multi-window / multi-threaded usage in the future
thread_local! {
    static GRAPHICS_CTX: RefCell<Option<Py<Graphics>>> = const { RefCell::new(None) };
}

#[pyclass(unsendable)]
pub struct Graphics {
    // this makes our object !Send, hence unsendable above
    glfw_ctx: GlfwContext,
    surface: Entity,
}

#[pymethods]
impl Graphics {
    // TODO: in theory users can create multiple Graphics objects, which they manually manage themselves.
    // right now we just support the single global window via the module-level functions.
    #[new]
    fn new(width: u32, height: u32) -> PyResult<Self> {
        let glfw_ctx = GlfwContext::new(width, height)
            .map_err(|e| PyRuntimeError::new_err(format!("Couold not create window {e}")))?;

        init().map_err(|e| PyRuntimeError::new_err(format!("Failed to initialize processing {e}")))?;

        let window_handle = glfw_ctx.get_window();
        let display_handle = glfw_ctx.get_display();
        let surface = surface_create(window_handle, display_handle, width, height, 1.0)
            .map_err(|e| PyRuntimeError::new_err(format!("Could not create surface {e}")))?;

        Ok(Self { glfw_ctx, surface })
    }

    pub fn background(&self, args: Vec<f32>) -> PyResult<()> {
        let (r, g, b, a) = parse_color(&args)?;
        let color = bevy::color::Color::srgba(r, g, b, a);
        record_command(self.surface, DrawCommand::BackgroundColor(color))
            .map_err(|e| PyRuntimeError::new_err(format!("background failed {e}")))
    }

    pub fn fill(&self, args: Vec<f32>) -> PyResult<()> {
        let (r, g, b, a) = parse_color(&args)?;
        let color = bevy::color::Color::srgba(r, g, b, a);
        record_command(self.surface, DrawCommand::Fill(color))
            .map_err(|e| PyRuntimeError::new_err(format!("fill failed {e}")))
    }

    pub fn no_fill(&self) -> PyResult<()> {
        record_command(self.surface, DrawCommand::NoFill)
            .map_err(|e| PyRuntimeError::new_err(format!("no_fill failed {e}")))
    }

    pub fn stroke(&self, args: Vec<f32>) -> PyResult<()> {
        let (r, g, b, a) = parse_color(&args)?;
        let color = bevy::color::Color::srgba(r, g, b, a);
        record_command(self.surface, DrawCommand::StrokeColor(color))
            .map_err(|e| PyRuntimeError::new_err(format!("stroke failed {e}")))
    }

    pub fn no_stroke(&self) -> PyResult<()> {
        record_command(self.surface, DrawCommand::NoStroke)
            .map_err(|e| PyRuntimeError::new_err(format!("no_stroke failed {e}")))
    }

    pub fn stroke_weight(&self, weight: f32) -> PyResult<()> {
        record_command(self.surface, DrawCommand::StrokeWeight(weight))
            .map_err(|e| PyRuntimeError::new_err(format!("stroke_weight failed {e}")))
    }

    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32, tl: f32, tr: f32, br: f32, bl: f32) -> PyResult<()> {
        record_command(
            self.surface,
            DrawCommand::Rect {
                x,
                y,
                w,
                h,
                radii: [tl, tr, br, bl],
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("rect failed {e}")))
    }

    pub fn run(&mut self, draw_fn: Option<Py<PyAny>>) -> PyResult<()> {
        loop {
            let running = self.glfw_ctx.poll_events();
            if !running {
                break;
            }

            begin_draw(self.surface)
                .map_err(|e| PyRuntimeError::new_err(format!("begin_draw failed {e}")))?;

            if let Some(ref draw) = draw_fn {
                Python::attach(|py| {
                    draw.call0(py)
                        .map_err(|e| PyRuntimeError::new_err(format!("draw failed {e}")))
                })?;
            }

            end_draw(self.surface)
                .map_err(|e| PyRuntimeError::new_err(format!("end_draw failed {e}")))?;
        }

        Ok(())
    }
}

// TODO: a real color type. or color parser? idk. color is confusing. let's think
// about how to expose different color spaces in an idiomatic pythonic way
fn parse_color(args: &[f32]) -> PyResult<(f32, f32, f32, f32)> {
    match args.len() {
        4 => Ok((
            args[0] / 255.0,
            args[1] / 255.0,
            args[2] / 255.0,
            args[3] / 255.0,
        )),
        _ => Err(PyRuntimeError::new_err(
            "Color requires 4 arguments",
        )),
    }
}

/// Run inside the current graphics context
pub fn with_graphics<F, T>(f: F) -> PyResult<T>
where
    F: FnOnce(PyRef<'_, Graphics>) -> PyResult<T>,
{
    GRAPHICS_CTX.with(|cell| {
        let opt = cell.borrow();
        match opt.as_ref() {
            Some(py_graphics) => Python::attach(|py| {
                let graphics = py_graphics.bind(py).borrow();
                f(graphics)
            }),
            None => Err(PyRuntimeError::new_err(
                "No graphics context",
            )),
        }
    })
}

/// Run inside the current graphics context with mutable access
pub fn with_graphics_mut<F, T>(f: F) -> PyResult<T>
where
    F: FnOnce(PyRefMut<'_, Graphics>) -> PyResult<T>,
{
    GRAPHICS_CTX.with(|cell| {
        let opt = cell.borrow();
        match opt.as_ref() {
            Some(py_graphics) => Python::attach(|py| {
                let graphics = py_graphics.bind(py).borrow_mut();
                f(graphics)
            }),
            None => Err(PyRuntimeError::new_err(
                "No graphics context",
            )),
        }
    })
}

/// Create the module level graphics context
pub fn create_context(width: u32, height: u32) -> PyResult<()> {
    let already_exists = GRAPHICS_CTX.with(|cell| cell.borrow().is_some());
    if already_exists {
        return Err(PyRuntimeError::new_err("A context already exists"));
    }

    Python::attach(|py| {
        let graphics = Py::new(py, Graphics::new(width, height)?)?;
        GRAPHICS_CTX.with(|cell| {
            *cell.borrow_mut() = Some(graphics);
        });
        Ok(())
    })
}