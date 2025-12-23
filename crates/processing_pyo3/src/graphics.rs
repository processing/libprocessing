use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::{exceptions::PyRuntimeError, prelude::*};

use crate::glfw::GlfwContext;

#[pyclass(unsendable)]
pub struct Surface {
    entity: Entity,
    glfw_ctx: GlfwContext,
}

#[pymethods]
impl Surface {
    pub fn poll_events(&mut self) -> bool {
        self.glfw_ctx.poll_events()
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        let _ = surface_destroy(self.entity);
    }
}

#[pyclass(unsendable)]
pub struct Graphics {
    entity: Entity,
    pub surface: Surface,
}

impl Drop for Graphics {
    fn drop(&mut self) {
        let _ = graphics_destroy(self.entity);
    }
}

#[pymethods]
impl Graphics {
    #[new]
    pub fn new(width: u32, height: u32) -> PyResult<Self> {
        let glfw_ctx =
            GlfwContext::new(width, height).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        init().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let surface = glfw_ctx
            .create_surface(width, height, 1.0)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let surface = Surface {
            entity: surface,
            glfw_ctx,
        };

        let graphics = graphics_create(surface.entity, width, height)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        Ok(Self {
            entity: graphics,
            surface,
        })
    }

    pub fn background(&self, args: Vec<f32>) -> PyResult<()> {
        let (r, g, b, a) = parse_color(&args)?;
        let color = bevy::color::Color::srgba(r, g, b, a);
        graphics_record_command(self.entity, DrawCommand::BackgroundColor(color))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn fill(&self, args: Vec<f32>) -> PyResult<()> {
        let (r, g, b, a) = parse_color(&args)?;
        let color = bevy::color::Color::srgba(r, g, b, a);
        graphics_record_command(self.entity, DrawCommand::Fill(color))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn no_fill(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::NoFill)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn stroke(&self, args: Vec<f32>) -> PyResult<()> {
        let (r, g, b, a) = parse_color(&args)?;
        let color = bevy::color::Color::srgba(r, g, b, a);
        graphics_record_command(self.entity, DrawCommand::StrokeColor(color))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn no_stroke(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::NoStroke)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn stroke_weight(&self, weight: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::StrokeWeight(weight))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn rect(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tl: f32,
        tr: f32,
        br: f32,
        bl: f32,
    ) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Rect {
                x,
                y,
                w,
                h,
                radii: [tl, tr, br, bl],
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn push_matrix(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::PushMatrix)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn pop_matrix(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::PopMatrix)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn reset_matrix(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::ResetMatrix)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn translate(&self, x: f32, y: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Translate { x, y })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn rotate(&self, angle: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Rotate { angle })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn scale(&self, x: f32, y: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Scale { x, y })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn shear_x(&self, angle: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::ShearX { angle })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn shear_y(&self, angle: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::ShearY { angle })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn begin_draw(&self) -> PyResult<()> {
        graphics_begin_draw(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn end_draw(&self) -> PyResult<()> {
        graphics_end_draw(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn mode_3d(&self) -> PyResult<()> {
        graphics_mode_3d(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn mode_2d(&self) -> PyResult<()> {
        graphics_mode_2d(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn camera_position(&self, x: f32, y: f32, z: f32) -> PyResult<()> {
        graphics_camera_position(self.entity, x, y, z)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn camera_look_at(&self, target_x: f32, target_y: f32, target_z: f32) -> PyResult<()> {
        graphics_camera_look_at(self.entity, target_x, target_y, target_z)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn perspective(&self, fov: f32, aspect: f32, near: f32, far: f32) -> PyResult<()> {
        graphics_perspective(self.entity, fov, aspect, near, far)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ortho(
        &self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> PyResult<()> {
        graphics_ortho(self.entity, left, right, bottom, top, near, far)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
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
        4 => Ok((
            args[0] / 255.0,
            args[1] / 255.0,
            args[2] / 255.0,
            args[3] / 255.0,
        )),
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
