use crate::color::{ColorMode, extract_color_with_mode};
use crate::color::{ColorMode, extract_color_with_mode};
use crate::glfw::GlfwContext;
use crate::glfw::GlfwContext;
use crate::input;
use crate::math::{extract_vec2, extract_vec3, extract_vec4};
use crate::math::{extract_vec2, extract_vec3, extract_vec4};
use bevy::{
    color::{ColorToPacked, Srgba},
    math::Vec4,
    prelude::Entity,
    render::render_resource::TextureFormat,
};
use processing::prelude::*;
use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyDict, PyTuple},
};

use crate::glfw::GlfwContext;
use crate::math::{extract_vec2, extract_vec3, extract_vec4};

#[pyclass(name = "BlendMode")]
#[derive(Clone)]
pub struct PyBlendMode {
    pub(crate) blend_state: Option<bevy::render::render_resource::BlendState>,
    name: Option<&'static str>,
}

impl PyBlendMode {
    pub(crate) fn from_preset(mode: BlendMode) -> Self {
        Self {
            blend_state: mode.to_blend_state(),
            name: Some(mode.name()),
        }
    }
}

#[pymethods]
impl PyBlendMode {
    /// Create a custom blend mode by specifying individual blend components.
    ///
    /// All arguments are keyword-only. Use the blend factor constants (ZERO, ONE,
    /// SRC_COLOR, SRC_ALPHA, DST_COLOR, etc.) and blend operation constants
    /// (OP_ADD, OP_SUBTRACT, OP_REVERSE_SUBTRACT, OP_MIN, OP_MAX).
    #[new]
    #[pyo3(signature = (*, color_src, color_dst, color_op, alpha_src, alpha_dst, alpha_op))]
    fn new(
        color_src: u8,
        color_dst: u8,
        color_op: u8,
        alpha_src: u8,
        alpha_dst: u8,
        alpha_op: u8,
    ) -> Self {
        Self {
            blend_state: Some(custom_blend_state(
                color_src, color_dst, color_op, alpha_src, alpha_dst, alpha_op,
            )),
            name: None,
        }
    }

    fn __repr__(&self) -> String {
        match self.name {
            Some(name) => format!("BlendMode.{name}"),
            None => "BlendMode(custom)".to_string(),
        }
    }
}


#[pyclass(unsendable)]
pub struct Surface {
    pub(crate) entity: Entity,
    glfw_ctx: Option<GlfwContext>,
}

#[pymethods]
impl Surface {
    pub fn poll_events(&mut self) -> bool {
        match &mut self.glfw_ctx {
            Some(ctx) => ctx.poll_events(),
            None => true, // no-op, offscreen surfaces never close
        }
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
    pub(crate) entity: Entity,
}

#[pymethods]
impl Light {
    #[pyo3(signature = (*args))]
    pub fn position(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = extract_vec3(args)?;
        transform_set_position(self.entity, v).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (*args))]
    pub fn look_at(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = extract_vec3(args)?;
        transform_look_at(self.entity, v).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
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
    pub(crate) entity: Entity,
}

impl Image {
    #[expect(dead_code)] // it's only used by webcam atm
    pub(crate) fn from_entity(entity: Entity) -> Self {
        Self { entity }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        let _ = image_destroy(self.entity);
    }
}

#[pyclass(unsendable)]
pub struct Geometry {
    pub(crate) entity: Entity,
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

    #[pyo3(signature = (*args))]
    pub fn color(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = extract_vec4(args)?;
        geometry_color(self.entity, v).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (*args))]
    pub fn normal(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = extract_vec3(args)?;
        geometry_normal(self.entity, v).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (*args))]
    pub fn vertex(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = extract_vec3(args)?;
        geometry_vertex(self.entity, v).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn index(&self, i: u32) -> PyResult<()> {
        geometry_index(self.entity, i).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (i, *args))]
    pub fn set_vertex(&self, i: u32, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = extract_vec3(args)?;
        geometry_set_vertex(self.entity, i, v).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }
}

#[pyclass(unsendable)]
pub struct Graphics {
    pub(crate) entity: Entity,
    pub surface: Surface,
    pub width: u32,
    pub height: u32,
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
        log_level: Option<&str>,
    ) -> PyResult<Self> {
        let mut glfw_ctx =
            GlfwContext::new(width, height).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let mut config = Config::new();
        config.set(ConfigKey::AssetRootPath, asset_path.to_string());
        config.set(ConfigKey::SketchRootPath, sketch_root_path.to_string());
        config.set(ConfigKey::SketchFileName, sketch_file_name.to_string());
        if let Some(level) = log_level {
            config.set(ConfigKey::LogLevel, level.to_string());
        }
        init(config).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let surface = glfw_ctx
            .create_surface(width, height)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let surface = Surface {
            entity: surface,
            glfw_ctx: Some(glfw_ctx),
        };

        let graphics = graphics_create(surface.entity, width, height, TextureFormat::Rgba16Float)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        Ok(Self {
            entity: graphics,
            surface,
            width,
            height,
        })
    }

    #[staticmethod]
    pub fn new_offscreen(
        width: u32,
        height: u32,
        asset_path: &str,
        log_level: Option<&str>,
    ) -> PyResult<Self> {
        let mut config = Config::new();
        config.set(ConfigKey::AssetRootPath, asset_path.to_string());
        if let Some(level) = log_level {
            config.set(ConfigKey::LogLevel, level.to_string());
        }
        init(config).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        // todo: allow caller to specify texture format? we use an sRGB format by default since
        // it plays well with converting to PNG
        let texture_format = TextureFormat::Rgba8UnormSrgb;

        let surface_entity = surface_create_offscreen(width, height, 1.0, texture_format)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let surface = Surface {
            entity: surface_entity,
            glfw_ctx: None,
        };

        let graphics = graphics_create(surface.entity, width, height, texture_format)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        Ok(Self {
            entity: graphics,
            surface,
            width,
            height,
        })
    }

    pub fn readback_png(&self) -> PyResult<Vec<u8>> {
        let raw = graphics_readback_raw(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        // png-ify our raw data, for srgb formats we're already good
        let rgba_bytes = match raw.format {
            TextureFormat::Rgba8UnormSrgb => raw.bytes,
            _ => {
                let pixels = graphics_readback(self.entity)
                    .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
                pixels
                    .iter()
                    .flat_map(|pixel| Srgba::from(*pixel).to_u8_array())
                    .collect()
            }
        };

        let mut png_buf: Vec<u8> = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_buf, raw.width, raw.height);
            // todo: infer these from the texture format instead of hardcoding
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
            let mut writer = encoder
                .write_header()
                .map_err(|e| PyRuntimeError::new_err(format!("PNG header: {e}")))?;
            writer
                .write_image_data(&rgba_bytes)
                .map_err(|e| PyRuntimeError::new_err(format!("PNG write: {e}")))?;
        }

        Ok(png_buf)
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

    #[pyo3(signature = (*args))]
    pub fn color(&self, args: &Bound<'_, PyTuple>) -> PyResult<crate::color::PyColor> {
        extract_color_with_mode(
            args,
            &graphics_get_color_mode(self.entity)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?,
        )
        .map(crate::color::PyColor::from)
    }

    #[pyo3(signature = (*args))]
    pub fn background(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let color = extract_color_with_mode(
            args,
            &graphics_get_color_mode(self.entity)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?,
        )?;
        graphics_record_command(self.entity, DrawCommand::BackgroundColor(color))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn background_image(&self, image: &Image) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::BackgroundImage(image.entity))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (*args))]
    pub fn fill(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let color = extract_color_with_mode(
            args,
            &graphics_get_color_mode(self.entity)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?,
        )?;
        graphics_record_command(self.entity, DrawCommand::Fill(color))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn no_fill(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::NoFill)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (*args))]
    pub fn stroke(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let color = extract_color_with_mode(
            args,
            &graphics_get_color_mode(self.entity)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?,
        )?;
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

    pub fn stroke_cap(&self, cap: u8) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::StrokeCap(processing::prelude::StrokeCapMode::from(cap)),
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn stroke_join(&self, join: u8) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::StrokeJoin(processing::prelude::StrokeJoinMode::from(join)),
        )
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

    #[pyo3(signature = (*args))]
    pub fn translate(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = extract_vec2(args)?;
        graphics_record_command(self.entity, DrawCommand::Translate(v))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn rotate(&self, angle: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Rotate { angle })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn draw_box(&self, width: f32, height: f32, depth: f32) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Box {
                width,
                height,
                depth,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn draw_sphere(&self, radius: f32, sectors: u32, stacks: u32) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Sphere {
                radius,
                sectors,
                stacks,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn roughness(&self, value: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Roughness(value))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn metallic(&self, value: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Metallic(value))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (*args))]
    pub fn emissive(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let color = extract_color_with_mode(
            args,
            &graphics_get_color_mode(self.entity)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?,
        )?;
        graphics_record_command(self.entity, DrawCommand::Emissive(color))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn unlit(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Unlit)
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

    #[pyo3(signature = (*args))]
    pub fn scale(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = extract_vec2(args)?;
        graphics_record_command(self.entity, DrawCommand::Scale(v))
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

    pub fn blend_mode(&self, mode: &PyBlendMode) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::BlendMode(mode.blend_state))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn set_material(&self, material: &crate::material::Material) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Material(material.entity))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(name = "color_mode", signature = (mode, max1=None, max2=None, max3=None, max_alpha=None))]
    pub fn set_color_mode<'py>(
        &self,
        mode: u8,
        max1: Option<&Bound<'py, PyAny>>,
        max2: Option<&Bound<'py, PyAny>>,
        max3: Option<&Bound<'py, PyAny>>,
        max_alpha: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<()> {
        let space = crate::color::ColorSpace::from_u8(mode)
            .ok_or_else(|| PyRuntimeError::new_err(format!("unknown color space: {mode}")))?;
        let parse =
            |obj: &Bound<'py, PyAny>, ch: usize| crate::color::parse_numeric(&space, obj, ch);
        let new_mode = match (max1, max2, max3, max_alpha) {
            // color_mode(MODE)
            (None, _, _, _) => ColorMode::with_defaults(space),
            // color_mode(MODE, max)
            (Some(m), None, _, _) => ColorMode::with_uniform_max(space, parse(m, 0)?),
            // color_mode(MODE, max1, max2, max3)
            (Some(m1), Some(m2), Some(m3), None) => {
                let defaults = space.default_maxes();
                ColorMode::new(
                    space,
                    parse(m1, 0)?,
                    parse(m2, 1)?,
                    parse(m3, 2)?,
                    defaults[3],
                )
            }
            // color_mode(MODE, max1, max2, max3, maxA)
            (Some(m1), Some(m2), Some(m3), Some(ma)) => ColorMode::new(
                space,
                parse(m1, 0)?,
                parse(m2, 1)?,
                parse(m3, 2)?,
                parse(ma, 3)?,
            ),
            _ => return Err(PyRuntimeError::new_err("expected 1, 2, 4, or 5 arguments")),
        };
        graphics_set_color_mode(self.entity, new_mode)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn begin_draw(&self) -> PyResult<()> {
        graphics_begin_draw(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn present(&self) -> PyResult<()> {
        graphics_present(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
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

    #[pyo3(signature = (*args))]
    pub fn camera_position(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = extract_vec3(args)?;
        transform_set_position(self.entity, v).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (*args))]
    pub fn camera_look_at(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = extract_vec3(args)?;
        transform_look_at(self.entity, v).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn perspective(&self, fov: f32, aspect: f32, near: f32, far: f32) -> PyResult<()> {
        graphics_perspective(
            self.entity,
            fov,
            aspect,
            near,
            far,
            Vec4::new(0.0, 0.0, -1.0, -near),
        )
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

    pub fn light_directional(
        &self,
        color: crate::color::ColorLike,
        illuminance: f32,
    ) -> PyResult<Light> {
        let color = color.into_color()?;
        match light_create_directional(self.entity, color, illuminance) {
            Ok(light) => Ok(Light { entity: light }),
            Err(e) => Err(PyRuntimeError::new_err(format!("{e}"))),
        }
    }

    pub fn light_point(
        &self,
        color: crate::color::ColorLike,
        intensity: f32,
        range: f32,
        radius: f32,
    ) -> PyResult<Light> {
        let color = color.into_color()?;
        match light_create_point(self.entity, color, intensity, range, radius) {
            Ok(light) => Ok(Light { entity: light }),
            Err(e) => Err(PyRuntimeError::new_err(format!("{e}"))),
        }
    }

    #[getter]
    fn mouse_x(&self) -> PyResult<f32> {
        input::mouse_x(self.surface.entity)
    }

    #[getter]
    fn mouse_y(&self) -> PyResult<f32> {
        input::mouse_y(self.surface.entity)
    }

    #[getter]
    fn pmouse_x(&self) -> PyResult<f32> {
        input::pmouse_x(self.surface.entity)
    }

    #[getter]
    fn pmouse_y(&self) -> PyResult<f32> {
        input::pmouse_y(self.surface.entity)
    }

    #[getter]
    fn mouse_is_pressed(&self) -> PyResult<bool> {
        input::mouse_is_pressed()
    }

    #[getter]
    fn mouse_button(&self) -> PyResult<Option<String>> {
        input::mouse_button()
    }

    #[getter]
    fn moved_x(&self) -> PyResult<f32> {
        input::moved_x()
    }

    #[getter]
    fn moved_y(&self) -> PyResult<f32> {
        input::moved_y()
    }

    #[getter]
    fn mouse_wheel(&self) -> PyResult<f32> {
        input::mouse_wheel()
    }

    #[getter]
    fn key(&self) -> PyResult<Option<String>> {
        input::key()
    }

    #[getter]
    fn key_code(&self) -> PyResult<Option<u32>> {
        input::key_code()
    }

    #[getter]
    fn key_is_pressed(&self) -> PyResult<bool> {
        input::key_is_pressed()
    }

    pub fn light_spot(
        &self,
        color: crate::color::ColorLike,
        intensity: f32,
        range: f32,
        radius: f32,
        inner_angle: f32,
        outer_angle: f32,
    ) -> PyResult<Light> {
        let color = color.into_color()?;
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

pub fn get_graphics<'py>(module: &Bound<'py, PyModule>) -> PyResult<Option<PyRef<'py, Graphics>>> {
    let Ok(attr) = module.getattr("_graphics") else {
        return Ok(None);
    };
    if attr.is_none() {
        return Ok(None);
    }
    let g = attr
        .cast_into::<Graphics>()
        .map_err(|_| PyRuntimeError::new_err("invalid graphics context"))?
        .try_borrow()
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    Ok(Some(g))
}

pub fn get_graphics_mut<'py>(
    module: &Bound<'py, PyModule>,
) -> PyResult<Option<PyRefMut<'py, Graphics>>> {
    let Ok(attr) = module.getattr("_graphics") else {
        return Ok(None);
    };
    if attr.is_none() {
        return Ok(None);
    }
    let g = attr
        .cast_into::<Graphics>()
        .map_err(|_| PyRuntimeError::new_err("invalid graphics context"))?
        .try_borrow_mut()
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    Ok(Some(g))
}
