use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::glfw::GlfwContext;

#[pyclass(unsendable)]
pub struct Graphics {
    glfw_ctx: GlfwContext,
    surface: Entity,
}

#[pymethods]
impl Graphics {
    #[new]
    pub fn new(width: u32, height: u32) -> PyResult<Self> {
        let glfw_ctx = GlfwContext::new(width, height)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        init().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let window_handle = glfw_ctx.get_window();
        let display_handle = glfw_ctx.get_display();
        let surface = surface_create(window_handle, display_handle, width, height, 1.0)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        Ok(Self { glfw_ctx, surface })
    }

    pub fn background(&self, args: Vec<f32>) -> PyResult<()> {
        let (r, g, b, a) = parse_color(&args)?;
        let color = bevy::color::Color::srgba(r, g, b, a);
        graphics_record_command(self.surface, DrawCommand::BackgroundColor(color))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn fill(&self, args: Vec<f32>) -> PyResult<()> {
        let (r, g, b, a) = parse_color(&args)?;
        let color = bevy::color::Color::srgba(r, g, b, a);
        graphics_record_command(self.surface, DrawCommand::Fill(color))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn no_fill(&self) -> PyResult<()> {
        graphics_record_command(self.surface, DrawCommand::NoFill)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn stroke(&self, args: Vec<f32>) -> PyResult<()> {
        let (r, g, b, a) = parse_color(&args)?;
        let color = bevy::color::Color::srgba(r, g, b, a);
        graphics_record_command(self.surface, DrawCommand::StrokeColor(color))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn no_stroke(&self) -> PyResult<()> {
        graphics_record_command(self.surface, DrawCommand::NoStroke)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn stroke_weight(&self, weight: f32) -> PyResult<()> {
        graphics_record_command(self.surface, DrawCommand::StrokeWeight(weight))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32, tl: f32, tr: f32, br: f32, bl: f32) -> PyResult<()> {
        graphics_record_command(
            self.surface,
            DrawCommand::Rect { x, y, w, h, radii: [tl, tr, br, bl] },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn run(&mut self, draw_fn: Option<Py<PyAny>>) -> PyResult<()> {
        loop {
            if !self.glfw_ctx.poll_events() {
                break;
            }

            graphics_begin_draw(self.surface)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

            if let Some(ref draw) = draw_fn {
                Python::attach(|py| {
                    draw.call0(py).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
                })?;
            }

            graphics_end_draw(self.surface)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        }
        Ok(())
    }
}

// TODO: a real color type. or color parser? idk. color is confusing. let's think
// about how to expose different color spaces in an idiomatic pythonic way
fn parse_color(args: &[f32]) -> PyResult<(f32, f32, f32, f32)> {
    match args.len() {
        1 => {
            let v = args[0] / 255.0;
            Ok((v, v, v, 1.0))
        }
        2 => {
            let v = args[0] / 255.0;
            Ok((v, v, v, args[1] / 255.0))
        }
        3 => Ok((args[0] / 255.0, args[1] / 255.0, args[2] / 255.0, 1.0)),
        4 => Ok((args[0] / 255.0, args[1] / 255.0, args[2] / 255.0, args[3] / 255.0)),
        _ => Err(PyRuntimeError::new_err("color requires 1-4 arguments")),
    }
}

pub fn get_graphics<'py>(module: &Bound<'py, PyModule>) -> PyResult<PyRef<'py, Graphics>> {
    module
        .getattr("_graphics")?
        .cast_into::<Graphics>()
        .map_err(|_| PyRuntimeError::new_err("no graphics context"))?
        .try_borrow()
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn get_graphics_mut<'py>(module: &Bound<'py, PyModule>) -> PyResult<PyRefMut<'py, Graphics>> {
    module
        .getattr("_graphics")?
        .cast_into::<Graphics>()
        .map_err(|_| PyRuntimeError::new_err("no graphics context"))?
        .try_borrow_mut()
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
