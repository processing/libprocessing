use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyDict};

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

#[pyclass]
#[derive(Debug)]
pub struct Light {
    entity: Entity,
}

#[pymethods]
impl Light {
    pub fn position(&self, x: f32, y: f32, z: f32) -> PyResult<()> {
        transform_set_position(self.entity, x, y, z)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn look_at(&self, x: f32, y: f32, z: f32) -> PyResult<()> {
        transform_look_at(self.entity, x, y, z).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }
}

// TODO: implement `light_destroy`
// impl Drop for Light {
//     fn drop(&mut self) {
//         let _ = light_destroy(self.entity);
//     }
// }

#[pyclass]
#[derive(Debug)]
pub struct Image {
    entity: Entity,
}

impl Drop for Image {
    fn drop(&mut self) {
        let _ = image_destroy(self.entity);
    }
}

#[pyclass(unsendable)]
pub struct Geometry {
    entity: Entity,
}

#[pyclass]
pub enum Topology {
    PointList = 0,
    LineList = 1,
    LineStrip = 2,
    TriangleList = 3,
    TriangleStrip = 4,
}

impl Topology {
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::PointList => 0,
            Self::LineList => 1,
            Self::LineStrip => 2,
            Self::TriangleList => 3,
            Self::TriangleStrip => 4,
        }
    }
}

#[pyclass]
pub struct Sketch {
    pub source: String,
}

#[pymethods]
impl Geometry {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn new(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let topology = kwargs
            .and_then(|k| k.get_item("topology").ok().flatten())
            .and_then(|t| t.cast_into::<Topology>().ok())
            .and_then(|t| geometry::Topology::from_u8(t.borrow().as_u8()))
            .unwrap_or(geometry::Topology::TriangleList);

        let geometry =
            geometry_create(topology).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Self { entity: geometry })
    }

    pub fn color(&self, r: f32, g: f32, b: f32, a: f32) -> PyResult<()> {
        geometry_color(self.entity, r, g, b, a).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn normal(&self, nx: f32, ny: f32, nz: f32) -> PyResult<()> {
        geometry_normal(self.entity, nx, ny, nz)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn vertex(&self, x: f32, y: f32, z: f32) -> PyResult<()> {
        geometry_vertex(self.entity, x, y, z).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn index(&self, i: u32) -> PyResult<()> {
        geometry_index(self.entity, i).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn set_vertex(&self, i: u32, x: f32, y: f32, z: f32) -> PyResult<()> {
        geometry_set_vertex(self.entity, i, x, y, z)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
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
    pub fn new(
        width: u32,
        height: u32,
        asset_path: &str,
        sketch_root_path: &str,
        sketch_file_name: &str,
    ) -> PyResult<Self> {
        let glfw_ctx =
            GlfwContext::new(width, height).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let mut config = Config::new();
        config.set(ConfigKey::AssetRootPath, asset_path.to_string());
        config.set(ConfigKey::SketchRootPath, sketch_root_path.to_string());
        config.set(ConfigKey::SketchFileName, sketch_file_name.to_string());
        init(config).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

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

    pub fn poll_for_sketch_update(&self) -> PyResult<Sketch> {
        match poll_for_sketch_updates().map_err(|_| PyRuntimeError::new_err("SKETCH UPDATE ERR"))? {
            Some(sketch) => Ok(Sketch {
                source: sketch.source,
            }),
            None => Ok(Sketch {
                source: "".to_string(),
            }),
        }
    }

    pub fn background(&self, args: Vec<f32>) -> PyResult<()> {
        let (r, g, b, a) = parse_color(&args)?;
        let color = bevy::color::Color::srgba(r, g, b, a);
        graphics_record_command(self.entity, DrawCommand::BackgroundColor(color))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn background_image(&self, image: &Image) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::BackgroundImage(image.entity))
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

    pub fn image(&self, file: &str) -> PyResult<Image> {
        match image_load(file) {
            Ok(image) => Ok(Image { entity: image }),
            Err(e) => Err(PyRuntimeError::new_err(format!("{e}"))),
        }
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

    pub fn draw_box(&self, x: f32, y: f32, z: f32) -> PyResult<()> {
        let box_geo = geometry_box(x, y, z).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        graphics_record_command(self.entity, DrawCommand::Geometry(box_geo))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn draw_sphere(&self, radius: f32, sectors: u32, stacks: u32) -> PyResult<()> {
        let sphere_geo = geometry_sphere(radius, sectors, stacks)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        graphics_record_command(self.entity, DrawCommand::Geometry(sphere_geo))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn draw_geometry(&self, geometry: &Geometry) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Geometry(geometry.entity))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn use_material(&self, material: &crate::material::Material) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Material(material.entity))
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

    pub fn set_material(&self, material: &crate::material::Material) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Material(material.entity))
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
        transform_set_position(self.entity, x, y, z)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn camera_look_at(&self, target_x: f32, target_y: f32, target_z: f32) -> PyResult<()> {
        transform_look_at(self.entity, target_x, target_y, target_z)
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

    pub fn light_directional(&self, r: f32, g: f32, b: f32, illuminance: f32) -> PyResult<Light> {
        let color = bevy::color::Color::srgb(r, g, b);
        match light_create_directional(self.entity, color, illuminance) {
            Ok(light) => Ok(Light { entity: light }),
            Err(e) => Err(PyRuntimeError::new_err(format!("{e}"))),
        }
    }

    pub fn light_point(
        &self,
        r: f32,
        g: f32,
        b: f32,
        intensity: f32,
        range: f32,
        radius: f32,
    ) -> PyResult<Light> {
        let color = bevy::color::Color::srgb(r, g, b);
        match light_create_point(self.entity, color, intensity, range, radius) {
            Ok(light) => Ok(Light { entity: light }),
            Err(e) => Err(PyRuntimeError::new_err(format!("{e}"))),
        }
    }

    pub fn light_spot(
        &self,
        r: f32,
        g: f32,
        b: f32,
        intensity: f32,
        range: f32,
        radius: f32,
        inner_angle: f32,
        outer_angle: f32,
    ) -> PyResult<Light> {
        let color = bevy::color::Color::srgb(r, g, b);
        match light_create_spot(
            self.entity,
            color,
            intensity,
            range,
            radius,
            inner_angle,
            outer_angle,
        ) {
            Ok(light) => Ok(Light { entity: light }),
            Err(e) => Err(PyRuntimeError::new_err(format!("{e}"))),
        }
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
