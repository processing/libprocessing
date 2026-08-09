use crate::color::{ColorMode, extract_color_with_mode};
use crate::glfw::GlfwContext;
use crate::input;
use crate::math::{extract_vec2, extract_vec3, extract_vec4};
use bevy::{
    color::{ColorToPacked, LinearRgba, Srgba},
    math::{Affine3A, Mat4, Vec3, Vec4},
    prelude::Entity,
    render::render_resource::{Extent3d, TextureFormat},
};
use processing::prelude::*;
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::{PyDict, PyList, PyTuple},
};
use std::sync::Mutex;

#[cfg(feature = "cuda")]
use crate::cuda::CudaImage;

/// Flatten `*args` of numbers (or a single list/tuple of numbers) into a `Vec<f32>`.
fn flatten_floats(args: &Bound<'_, PyTuple>) -> PyResult<Vec<f32>> {
    if args.len() == 1 {
        if let Ok(seq) = args.get_item(0)?.extract::<Vec<f32>>() {
            return Ok(seq);
        }
    }
    let mut out = Vec::with_capacity(args.len());
    for item in args.iter() {
        out.push(item.extract::<f32>()?);
    }
    Ok(out)
}

/// Build a Python list of `Color` objects from linear-RGBA pixels.
fn pixels_to_pylist(py: Python<'_>, pixels: &[LinearRgba]) -> PyResult<Py<PyList>> {
    let colors = pixels
        .iter()
        .map(|p| crate::color::PyColor(bevy::prelude::Color::LinearRgba(*p)));
    Ok(PyList::new(py, colors)?.unbind())
}

/// Convert a Python sequence of colors (as returned by `load_pixels()`, or
/// individual `Color`/`[r,g,b,a]` values) into linear-RGBA pixels.
fn pyseq_to_pixels(seq: &Bound<'_, PyAny>) -> PyResult<Vec<LinearRgba>> {
    let mut out = Vec::with_capacity(seq.len().unwrap_or(0));
    for item in seq.try_iter()? {
        let item = item?;
        let color = if let Ok(c) = item.extract::<crate::color::PyColor>() {
            c.0
        } else if let Ok(rgba) = item.extract::<[f32; 4]>() {
            bevy::prelude::Color::LinearRgba(LinearRgba::new(rgba[0], rgba[1], rgba[2], rgba[3]))
        } else if let Ok(rgb) = item.extract::<[f32; 3]>() {
            bevy::prelude::Color::LinearRgba(LinearRgba::new(rgb[0], rgb[1], rgb[2], 1.0))
        } else {
            return Err(PyValueError::new_err(
                "pixels must contain Color values or [r, g, b, (a)] sequences",
            ));
        };
        out.push(LinearRgba::from(color));
    }
    Ok(out)
}

/// Resolve the sequence `update_pixels()` should write: an explicit argument if
/// given, otherwise the buffer cached by `load_pixels()`.
fn resolve_pixels_arg<'py>(
    py: Python<'py>,
    pixels: Option<&Bound<'py, PyAny>>,
    cache: &Mutex<Option<Py<PyList>>>,
) -> PyResult<Bound<'py, PyAny>> {
    match pixels {
        Some(p) => Ok(p.clone()),
        None => {
            let cached = cache.lock().unwrap().as_ref().map(|l| l.clone_ref(py));
            match cached {
                Some(list) => Ok(list.into_bound(py).into_any()),
                None => Err(PyRuntimeError::new_err(
                    "update_pixels() called before load_pixels()",
                )),
            }
        }
    }
}

fn rt_err(e: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(format!("{e}"))
}

/// Composite mode code for `mask()` (see `filters/composite.wgsl`).
const COMPOSITE_MODE_MASK: u32 = 10;
/// Composite mode code for `copy()` — REPLACE (matches `BlendMode::Replace`).
const COMPOSITE_MODE_REPLACE: u32 = 9;

/// Normalize a pixel rect `(x, y, w, h)` to uv-space `(x0, y0, x1, y1)`; `None`
/// means the full `(0, 0, 1, 1)` extent.
fn normalize_rect(rect: Option<[f32; 4]>, w: f32, h: f32) -> [f32; 4] {
    match rect {
        None => [0.0, 0.0, 1.0, 1.0],
        Some([x, y, rw, rh]) => [x / w, y / h, (x + rw) / w, (y + rh) / h],
    }
}

/// Normalize a source pixel rect against the source image's own dimensions.
fn normalize_src_rect(entity: Entity, rect: Option<[f32; 4]>) -> PyResult<[f32; 4]> {
    match rect {
        None => Ok([0.0, 0.0, 1.0, 1.0]),
        Some(r) => {
            let (w, h) = image_size(entity).map_err(rt_err)?;
            Ok(normalize_rect(Some(r), w as f32, h as f32))
        }
    }
}

/// Resolve a composite source (`Image`, `Webcam`, or `Graphics`) to a
/// texture-bearing entity, plus the graphics entity that must be flushed first
/// when the source is a graphics (so its latest content is what gets sampled).
/// A `Graphics` source uses its render-target surface entity, which carries an
/// `Image` component for offscreen graphics.
fn resolve_composite_source(src: &Bound<'_, PyAny>) -> PyResult<(Entity, Option<Entity>)> {
    if let Ok(img) = src.extract::<ImageRef>() {
        return Ok((img.entity, None));
    }
    if let Ok(g) = src.extract::<PyRef<Graphics>>() {
        return Ok((g.surface.entity, Some(g.entity)));
    }
    Err(PyValueError::new_err(
        "composite source must be an Image, Webcam, or Graphics",
    ))
}

/// Run the built-in composite filter: blend the `src` texture into the `dst`
/// graphics target using `mode`, with normalized src/dst rects. This is the
/// shader-based composite pass shared by `copy`/`blend`/`mask`.
fn composite_apply(
    dst: Entity,
    src: Entity,
    mode: u32,
    src_rect: [f32; 4],
    dst_rect: [f32; 4],
    opacity: f32,
) -> PyResult<()> {
    use shader_value::ShaderValue;
    let filter = filter_composite().map_err(rt_err)?;
    filter_set(filter, "src", ShaderValue::Texture(src)).map_err(rt_err)?;
    filter_set(filter, "mode", ShaderValue::UInt(mode)).map_err(rt_err)?;
    filter_set(filter, "opacity", ShaderValue::Float(opacity)).map_err(rt_err)?;
    filter_set(filter, "src_rect", ShaderValue::Float4(src_rect)).map_err(rt_err)?;
    filter_set(filter, "dst_rect", ShaderValue::Float4(dst_rect)).map_err(rt_err)?;
    graphics_apply_filter(dst, filter).map_err(rt_err)
}

/// Write raw sRGB RGBA bytes to a PNG file on disk.
fn write_png_file(path: &str, width: u32, height: u32, rgba: &[u8]) -> PyResult<()> {
    let lower = path.to_lowercase();
    if !lower.ends_with(".png") {
        return Err(PyValueError::new_err(format!(
            "save() currently supports only .png files, got {path:?}"
        )));
    }
    let file = std::fs::File::create(path)
        .map_err(|e| PyRuntimeError::new_err(format!("create {path}: {e}")))?;
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let mut writer = encoder
        .write_header()
        .map_err(|e| PyRuntimeError::new_err(format!("PNG header: {e}")))?;
    writer
        .write_image_data(rgba)
        .map_err(|e| PyRuntimeError::new_err(format!("PNG write: {e}")))
}

/// Parse Processing-style `applyMatrix`/`setMatrix` arguments into an `Affine3A`:
/// 6 values for a 2D affine (`n00 n01 n02 n10 n11 n12`) or 16 values for a full
/// 3D matrix in row-major order (matching Processing; glam is column-major).
fn affine_from_matrix_args(args: &Bound<'_, PyTuple>) -> PyResult<Affine3A> {
    let v = flatten_floats(args)?;
    let mat = match v.len() {
        6 => Mat4::from_cols_array(&[
            v[0], v[3], 0.0, 0.0, //
            v[1], v[4], 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            v[2], v[5], 0.0, 1.0,
        ]),
        16 => {
            let mut arr = [0.0f32; 16];
            arr.copy_from_slice(&v);
            Mat4::from_cols_array(&arr).transpose()
        }
        n => {
            return Err(PyValueError::new_err(format!(
                "matrix expects 6 (2D) or 16 (3D) values, got {n}"
            )));
        }
    };
    Ok(Affine3A::from_mat4(mat))
}

#[cfg(feature = "cuda")]
fn cuda_import_from_interface(
    entity: bevy::prelude::Entity,
    obj: &pyo3::Bound<'_, pyo3::PyAny>,
) -> PyResult<()> {
    let interface = obj
        .getattr("__cuda_array_interface__")?
        .cast_into::<PyDict>()?;

    let data_tuple: (u64, bool) = interface
        .get_item("data")?
        .ok_or_else(|| PyRuntimeError::new_err("missing 'data' in __cuda_array_interface__"))?
        .extract()?;
    let src_ptr = data_tuple.0;

    let shape: Vec<usize> = interface
        .get_item("shape")?
        .ok_or_else(|| PyRuntimeError::new_err("missing 'shape' in __cuda_array_interface__"))?
        .extract()?;

    let typestr: String = interface
        .get_item("typestr")?
        .ok_or_else(|| PyRuntimeError::new_err("missing 'typestr' in __cuda_array_interface__"))?
        .extract()?;

    let elem_size = processing_cuda::elem_size_for_typestr(&typestr)
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    let total_elements: usize = shape.iter().product();
    let byte_size = (total_elements * elem_size) as u64;

    processing_cuda::cuda_import(entity, src_ptr, byte_size)
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

#[pyclass(name = "BlendMode", from_py_object)]
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

    /// The composite-shader mode code (the `BlendMode` discriminant) for this
    /// blend mode, or `None` for custom modes with no named preset.
    pub(crate) fn composite_mode(&self) -> Option<u32> {
        self.name.and_then(BlendMode::from_name).map(|m| m as u32)
    }
}

#[pymethods]
impl PyBlendMode {
    #[new]
    #[pyo3(signature = (*, color_src, color_dst, color_op, alpha_src, alpha_dst, alpha_op))]
    fn new(
        color_src: u8,
        color_dst: u8,
        color_op: u8,
        alpha_src: u8,
        alpha_dst: u8,
        alpha_op: u8,
    ) -> PyResult<Self> {
        let blend_state = custom_blend_state(
            color_src, color_dst, color_op, alpha_src, alpha_dst, alpha_op,
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Self {
            blend_state: Some(blend_state),
            name: None,
        })
    }

    fn __repr__(&self) -> String {
        match self.name {
            Some(name) => format!("BlendMode.{name}"),
            None => "BlendMode(custom)".to_string(),
        }
    }

    #[classattr]
    const ZERO: u8 = 0;
    #[classattr]
    const ONE: u8 = 1;
    #[classattr]
    const SRC_COLOR: u8 = 2;
    #[classattr]
    const ONE_MINUS_SRC_COLOR: u8 = 3;
    #[classattr]
    const SRC_ALPHA: u8 = 4;
    #[classattr]
    const ONE_MINUS_SRC_ALPHA: u8 = 5;
    #[classattr]
    const DST_COLOR: u8 = 6;
    #[classattr]
    const ONE_MINUS_DST_COLOR: u8 = 7;
    #[classattr]
    const DST_ALPHA: u8 = 8;
    #[classattr]
    const ONE_MINUS_DST_ALPHA: u8 = 9;
    #[classattr]
    const SRC_ALPHA_SATURATED: u8 = 10;

    #[classattr]
    const OP_ADD: u8 = 0;
    #[classattr]
    const OP_SUBTRACT: u8 = 1;
    #[classattr]
    const OP_REVERSE_SUBTRACT: u8 = 2;
    #[classattr]
    const OP_MIN: u8 = 3;
    #[classattr]
    const OP_MAX: u8 = 4;
}

/// Configures how an image is sampled when drawn.
///
/// Controls texture filtering and edge wrapping behavior.
///
/// - `filter` — `LINEAR` (smooth, default) or `NEAREST` (pixelated).
/// - `wrap` — `CLAMP` (default), `REPEAT`, or `MIRROR`.
///   Use `wrap_x`/`wrap_y` to set each axis independently.
#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct Sampler {
    pub(crate) filter: u8,
    pub(crate) wrap_x: u8,
    pub(crate) wrap_y: u8,
}

impl Sampler {
    fn parse_filter(s: &str) -> PyResult<u8> {
        match () {
            _ if s.eq_ignore_ascii_case(constants::LINEAR) => Ok(0),
            _ if s.eq_ignore_ascii_case(constants::NEAREST) => Ok(1),
            _ => Err(PyValueError::new_err(format!(
                "unknown filter: {s:?} (expected {:?} or {:?})",
                constants::LINEAR,
                constants::NEAREST
            ))),
        }
    }

    fn parse_wrap(s: &str) -> PyResult<u8> {
        match () {
            _ if s.eq_ignore_ascii_case(constants::CLAMP) => Ok(0),
            _ if s.eq_ignore_ascii_case(constants::REPEAT) => Ok(1),
            _ if s.eq_ignore_ascii_case(constants::MIRROR) => Ok(2),
            _ => Err(PyValueError::new_err(format!(
                "unknown wrap: {s:?} (expected {:?}, {:?}, or {:?})",
                constants::CLAMP,
                constants::REPEAT,
                constants::MIRROR
            ))),
        }
    }

    fn filter_name(filter: u8) -> &'static str {
        match filter {
            0 => constants::LINEAR,
            1 => constants::NEAREST,
            _ => "?",
        }
    }

    fn wrap_name(wrap: u8) -> &'static str {
        match wrap {
            0 => constants::CLAMP,
            1 => constants::REPEAT,
            2 => constants::MIRROR,
            _ => "?",
        }
    }
}

#[pymethods]
impl Sampler {
    #[new]
    #[pyo3(signature = (*, filter=constants::LINEAR, wrap=constants::CLAMP, wrap_x=None, wrap_y=None))]
    fn new(filter: &str, wrap: &str, wrap_x: Option<&str>, wrap_y: Option<&str>) -> PyResult<Self> {
        let wrap_default = Self::parse_wrap(wrap)?;
        Ok(Self {
            filter: Self::parse_filter(filter)?,
            wrap_x: wrap_x
                .map(Self::parse_wrap)
                .transpose()?
                .unwrap_or(wrap_default),
            wrap_y: wrap_y
                .map(Self::parse_wrap)
                .transpose()?
                .unwrap_or(wrap_default),
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "Sampler(filter={:?}, wrap_x={:?}, wrap_y={:?})",
            Self::filter_name(self.filter),
            Self::wrap_name(self.wrap_x),
            Self::wrap_name(self.wrap_y)
        )
    }
}

pub use crate::surface::Surface;

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
pub struct Font {
    pub(crate) entity: Entity,
}

#[pymethods]
impl Font {
    /// Query variable font axes.
    ///
    /// Returns a list of `(tag, min, max, default)` tuples.
    pub fn variations(&self) -> PyResult<Vec<(String, f32, f32, f32)>> {
        font_variations(self.entity)
            .map(|axes| {
                axes.into_iter()
                    .map(|a| (a.tag, a.min, a.max, a.default))
                    .collect()
            })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Query font metadata.
    ///
    /// Returns a `(family, style, weight, width, is_variable)` tuple.
    pub fn metadata(&self) -> PyResult<(String, String, f32, f32, bool)> {
        font_metadata(self.entity)
            .map(|m| (m.family, m.style, m.weight, m.width, m.is_variable))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }
}

/// Convert glyph outline groups into per-group lists of Python command tuples.
fn path_commands_to_py(
    py: Python<'_>,
    groups: Vec<Vec<processing_render::render::primitive::text::PathCommand>>,
) -> Vec<Vec<Py<PyAny>>> {
    use processing_render::render::primitive::text::PathCommand;

    let to_py = |cmd: PathCommand| -> Py<PyAny> {
        match cmd {
            PathCommand::MoveTo(x, y) => ("M", x, y).into_pyobject(py).unwrap().into_any().unbind(),
            PathCommand::LineTo(x, y) => ("L", x, y).into_pyobject(py).unwrap().into_any().unbind(),
            PathCommand::QuadTo { cx, cy, x, y } => ("Q", cx, cy, x, y)
                .into_pyobject(py)
                .unwrap()
                .into_any()
                .unbind(),
            PathCommand::CubicTo {
                cx1,
                cy1,
                cx2,
                cy2,
                x,
                y,
            } => ("C", cx1, cy1, cx2, cy2, x, y)
                .into_pyobject(py)
                .unwrap()
                .into_any()
                .unbind(),
            PathCommand::Close => ("Z",).into_pyobject(py).unwrap().into_any().unbind(),
        }
    };

    groups
        .into_iter()
        .map(|group| group.into_iter().map(to_py).collect())
        .collect()
}

#[pyclass]
#[derive(Debug)]
pub struct Image {
    pub(crate) entity: Entity,
    /// Cached `pixels` list populated by `load_pixels()` and written back by
    /// `update_pixels()`, mirroring Processing's `PImage.pixels[]`.
    pixel_cache: Mutex<Option<Py<PyList>>>,
}

impl Image {
    pub(crate) fn wrap(entity: Entity) -> Self {
        Self {
            entity,
            pixel_cache: Mutex::new(None),
        }
    }
}

pub(crate) struct ImageRef {
    pub entity: Entity,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ImageRef {
    type Error = PyErr;

    fn extract(ob: pyo3::Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        if let Ok(img) = ob.extract::<PyRef<Image>>() {
            return Ok(ImageRef { entity: img.entity });
        }
        #[cfg(feature = "webcam")]
        if let Ok(cam) = ob.extract::<PyRef<crate::webcam::Webcam>>() {
            return Ok(ImageRef {
                entity: cam.image_entity()?,
            });
        }
        Err(pyo3::exceptions::PyTypeError::new_err(
            "expected an Image or Webcam",
        ))
    }
}

#[pymethods]
impl Image {
    /// Applies a `Sampler` to this image, controlling filtering and wrapping.
    ///
    /// ```python
    /// s = Sampler(filter=NEAREST, wrap=REPEAT)
    /// img.sampler(s)
    /// ```
    fn sampler(&self, sampler: &Sampler) -> PyResult<()> {
        image_set_sampler(self.entity, sampler.filter, sampler.wrap_x, sampler.wrap_y)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Image width in pixels.
    #[getter]
    fn width(&self) -> PyResult<u32> {
        image_size(self.entity)
            .map(|(w, _)| w)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Image height in pixels.
    #[getter]
    fn height(&self) -> PyResult<u32> {
        image_size(self.entity)
            .map(|(_, h)| h)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Read a single pixel as a `Color` (Processing `get`).
    fn get(&self, x: u32, y: u32) -> PyResult<crate::color::PyColor> {
        let (w, h) = image_size(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        if x >= w || y >= h {
            return Err(PyValueError::new_err(format!(
                "pixel ({x}, {y}) out of bounds for {w}x{h} image"
            )));
        }
        let pixels =
            image_readback(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let p = pixels[(y * w + x) as usize];
        Ok(crate::color::PyColor(bevy::prelude::Color::LinearRgba(p)))
    }

    /// Write a single pixel (Processing `set`). Accepts a `Color`, hex string,
    /// or color components in the default (sRGB) color mode.
    #[pyo3(signature = (x, y, *args))]
    fn set(&self, x: u32, y: u32, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let color = extract_color_with_mode(args, &ColorMode::default())?;
        image_update_region(self.entity, x, y, 1, 1, &[LinearRgba::from(color)])
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Read all pixels into the `pixels` list (Processing `loadPixels`).
    fn load_pixels(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let pixels =
            image_readback(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let list = pixels_to_pylist(py, &pixels)?;
        *self.pixel_cache.lock().unwrap() = Some(list.clone_ref(py));
        Ok(list)
    }

    /// The pixel buffer as a list of `Color` (Processing `pixels[]`). Auto-loads
    /// on first access if `load_pixels()` has not been called.
    #[getter]
    fn pixels(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let cached = self
            .pixel_cache
            .lock()
            .unwrap()
            .as_ref()
            .map(|list| list.clone_ref(py));
        match cached {
            Some(list) => Ok(list),
            None => self.load_pixels(py),
        }
    }

    /// Write the `pixels` list back to the image (Processing `updatePixels`). If
    /// `pixels` is omitted, the cached buffer from `load_pixels()` is used.
    #[pyo3(signature = (pixels=None))]
    fn update_pixels(&self, py: Python<'_>, pixels: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        let seq = resolve_pixels_arg(py, pixels, &self.pixel_cache)?;
        let data = pyseq_to_pixels(&seq)?;
        image_update(self.entity, &data).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Copies a source into this image via a GPU blit (a sampling copy that
    /// handles differing size and format). `src` may be another `Image` or a
    /// `Graphics` render target — e.g. render into an offscreen graphics, then
    /// `img.copy_from(g)` to pull the result into a plain, sampleable image.
    fn copy_from(&self, src: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Ok(img) = src.extract::<PyRef<Image>>() {
            image_copy_from(self.entity, img.entity, false).map_err(rt_err)
        } else if let Ok(g) = src.extract::<PyRef<Graphics>>() {
            image_copy_from(self.entity, g.entity, true).map_err(rt_err)
        } else {
            Err(PyValueError::new_err(
                "copy_from() expects an Image or Graphics source",
            ))
        }
    }

    /// Save the image to a PNG file (Processing `save`).
    fn save(&self, filename: &str) -> PyResult<()> {
        let (w, h) = image_size(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let pixels =
            image_readback(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let rgba: Vec<u8> = pixels
            .iter()
            .flat_map(|p| Srgba::from(*p).to_u8_array())
            .collect();
        write_png_file(filename, w, h, &rgba)
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        let _ = image_destroy(self.entity);
    }
}

#[cfg(feature = "cuda")]
#[pymethods]
impl Image {
    pub fn cuda(&self) -> PyResult<CudaImage> {
        processing_cuda::cuda_export(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(CudaImage::new(self.entity))
    }

    pub fn update_from(&self, obj: &Bound<'_, pyo3::PyAny>) -> PyResult<()> {
        cuda_import_from_interface(self.entity, obj)
    }
}

#[pyclass(unsendable)]
pub struct Geometry {
    pub(crate) entity: Entity,
}

#[pyclass]
pub struct Sketch {
    pub source: String,
}

impl Geometry {
    pub(crate) fn create(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let topology = match kwargs.and_then(|k| k.get_item("topology").ok().flatten()) {
            Some(t) => {
                let s = t.extract::<String>()?;
                geometry::Topology::parse(&s)
                    .ok_or_else(|| PyValueError::new_err(format!("unknown topology: {s:?}")))?
            }
            None => geometry::Topology::TriangleList,
        };

        let geometry =
            geometry_create(topology).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Self { entity: geometry })
    }
}

#[pymethods]
impl Geometry {
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

    #[pyo3(signature = (*args))]
    pub fn uv(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = extract_vec2(args)?;
        geometry_uv(self.entity, v.x, v.y).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn index(&self, i: u32) -> PyResult<()> {
        geometry_index(self.entity, i).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (i, *args))]
    pub fn set_vertex(&self, i: u32, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = extract_vec3(args)?;
        geometry_set_vertex(self.entity, i, v).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn vertex_count(&self) -> PyResult<u32> {
        geometry_vertex_count(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[staticmethod]
    #[pyo3(signature = (radius, sectors=32, stacks=18))]
    pub fn sphere(radius: f32, sectors: u32, stacks: u32) -> PyResult<Self> {
        let entity = geometry_sphere(radius, sectors, stacks)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Self { entity })
    }

    #[staticmethod]
    pub fn r#box(width: f32, height: f32, depth: f32) -> PyResult<Self> {
        let entity = geometry_box(width, height, depth)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Self { entity })
    }

    /// lattice centered at the origin; topology is `POINTS`, intended as a
    /// position source for `Particles(geometry=...)` rather than rasterized.
    #[staticmethod]
    #[pyo3(signature = (nx, ny, nz, spacing=1.0))]
    pub fn grid(nx: u32, ny: u32, nz: u32, spacing: f32) -> PyResult<Self> {
        let entity = geometry_grid(nx, ny, nz, spacing)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Self { entity })
    }
}

#[pyclass(unsendable)]
pub struct Graphics {
    pub(crate) entity: Entity,
    pub surface: Surface,
    #[pyo3(get)]
    pub width: u32,
    #[pyo3(get)]
    pub height: u32,
    /// Cached `pixels` list populated by `load_pixels()` and written back by
    /// `update_pixels()`, mirroring Processing's `pixels[]`.
    pixel_cache: Mutex<Option<Py<PyList>>>,
}

impl Drop for Graphics {
    fn drop(&mut self) {
        let _ = graphics_destroy(self.entity);
    }
}

impl Graphics {
    /// Create an offscreen graphics buffer in the already-running app (no window,
    /// no `init`). Backs `create_graphics()` / `new_offscreen()`. The caller must
    /// ensure the app exists (i.e. `size()` was called first).
    pub(crate) fn wrap_offscreen(width: u32, height: u32) -> PyResult<Self> {
        // sRGB by default: it plays well with PNG export and blits.
        let texture_format = TextureFormat::Rgba8UnormSrgb;
        let surface_entity = surface_create_offscreen(width, height, 1.0, texture_format)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Self::from_surface(surface_entity, width, height, texture_format, None)
    }

    /// Wrap an existing surface entity in a `Graphics` (camera + draw state). The
    /// `glfw_ctx` is `None` for offscreen and secondary windows — the main
    /// surface owns the shared `GlfwContext` that drives every window.
    pub(crate) fn from_surface(
        surface_entity: Entity,
        width: u32,
        height: u32,
        texture_format: TextureFormat,
        glfw_ctx: Option<GlfwContext>,
    ) -> PyResult<Self> {
        let graphics = graphics_create(surface_entity, width, height, texture_format)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Self {
            entity: graphics,
            surface: Surface {
                entity: surface_entity,
                glfw_ctx,
            },
            width,
            height,
            pixel_cache: Mutex::new(None),
        })
    }

    /// Wrap a secondary window's surface in a `Graphics` (HDR window target). The
    /// window lives on the main surface's shared `GlfwContext`, so this holds no
    /// `glfw_ctx` of its own.
    pub(crate) fn wrap_window(surface_entity: Entity, width: u32, height: u32) -> PyResult<Self> {
        Self::from_surface(
            surface_entity,
            width,
            height,
            TextureFormat::Rgba16Float,
            None,
        )
    }
}

#[pymethods]
impl Graphics {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        width: u32,
        height: u32,
        asset_path: &str,
        sketch_root_path: &str,
        sketch_file_name: &str,
        log_level: Option<&str>,
        transparent: bool,
    ) -> PyResult<Self> {
        let mut glfw_ctx = GlfwContext::new(width, height, transparent)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let mut config = Config::new();
        config.set(ConfigKey::AssetRootPath, asset_path.to_string());
        config.set(ConfigKey::SketchRootPath, sketch_root_path.to_string());
        config.set(ConfigKey::SketchFileName, sketch_file_name.to_string());
        if let Some(level) = log_level {
            config.set(ConfigKey::LogLevel, level.to_string());
        }
        init(config).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let surface = glfw_ctx
            .create_surface(width, height, transparent)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let surface = Surface {
            entity: surface,
            glfw_ctx: Some(glfw_ctx),
        };

        let graphics = graphics_create(surface.entity, width, height, TextureFormat::Rgba16Float)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        // The Window component (applied each frame by the sync loop) otherwise
        // defaults its title; match the GLFW create-time title.
        surface_set_title(surface.entity, "Processing".to_string())
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        Ok(Self {
            entity: graphics,
            surface,
            width,
            height,
            pixel_cache: Mutex::new(None),
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
        Self::wrap_offscreen(width, height)
    }

    #[getter]
    pub fn focused(&self) -> PyResult<bool> {
        self.surface.focused()
    }

    #[getter]
    pub fn pixel_density(&self) -> PyResult<f32> {
        self.surface.pixel_density()
    }

    #[getter]
    pub fn pixel_width(&self) -> PyResult<u32> {
        self.surface.pixel_width()
    }

    #[getter]
    pub fn pixel_height(&self) -> PyResult<u32> {
        self.surface.pixel_height()
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

    /// Save the current canvas to a PNG file (Processing `save`).
    pub fn save(&self, filename: &str) -> PyResult<()> {
        let raw = graphics_readback_raw(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let rgba = match raw.format {
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
        write_png_file(filename, raw.width, raw.height, &rgba)
    }

    /// Read a single pixel as a `Color` (Processing `get`).
    pub fn get(&self, x: u32, y: u32) -> PyResult<crate::color::PyColor> {
        if x >= self.width || y >= self.height {
            return Err(PyValueError::new_err(format!(
                "pixel ({x}, {y}) out of bounds for {}x{} canvas",
                self.width, self.height
            )));
        }
        let pixels = graphics_readback(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let p = pixels
            .get((y * self.width + x) as usize)
            .ok_or_else(|| PyValueError::new_err("pixel out of bounds"))?;
        Ok(crate::color::PyColor(bevy::prelude::Color::LinearRgba(*p)))
    }

    /// Write a single pixel (Processing `set`).
    #[pyo3(signature = (x, y, *args))]
    pub fn set(&self, x: u32, y: u32, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let color = extract_color_with_mode(
            args,
            &graphics_get_color_mode(self.entity)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?,
        )?;
        graphics_update_region(self.entity, x, y, 1, 1, &[LinearRgba::from(color)])
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Read all pixels into the `pixels` list (Processing `loadPixels`).
    pub fn load_pixels(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let pixels = graphics_readback(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let list = pixels_to_pylist(py, &pixels)?;
        *self.pixel_cache.lock().unwrap() = Some(list.clone_ref(py));
        Ok(list)
    }

    /// The pixel buffer as a list of `Color` (Processing `pixels[]`). Auto-loads
    /// on first access if `load_pixels()` has not been called.
    #[getter]
    pub fn pixels(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let cached = self
            .pixel_cache
            .lock()
            .unwrap()
            .as_ref()
            .map(|list| list.clone_ref(py));
        match cached {
            Some(list) => Ok(list),
            None => self.load_pixels(py),
        }
    }

    /// Write the `pixels` list back to the canvas (Processing `updatePixels`).
    /// If `pixels` is omitted, the cached buffer from `load_pixels()` is used.
    #[pyo3(signature = (pixels=None))]
    pub fn update_pixels(&self, py: Python<'_>, pixels: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        let seq = resolve_pixels_arg(py, pixels, &self.pixel_cache)?;
        let data = pyseq_to_pixels(&seq)?;
        graphics_update(self.entity, &data).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
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

    #[pyo3(signature = (kind, *args, **kwargs))]
    pub fn filter(
        &self,
        kind: Bound<'_, PyAny>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let filter = crate::filter::resolve_filter(&kind, args, kwargs)?;
        graphics_apply_filter(self.entity, filter)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
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

    pub fn clear(&self) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::BackgroundColor(bevy::prelude::Color::NONE),
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (*args))]
    pub fn fill(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        if args.len() == 1
            && let Ok(buf) = args.get_item(0)?.extract::<PyRef<crate::compute::Buffer>>()
        {
            return graphics_record_command(self.entity, DrawCommand::FillBuffer(buf.entity))
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")));
        }
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

    pub fn rect_mode(&self, mode: &str) -> PyResult<()> {
        let mode = ShapeMode::parse(mode)
            .ok_or_else(|| PyValueError::new_err(format!("unknown rect mode: {mode:?}")))?;
        graphics_record_command(self.entity, DrawCommand::RectMode(mode))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn ellipse_mode(&self, mode: &str) -> PyResult<()> {
        let mode = ShapeMode::parse(mode)
            .ok_or_else(|| PyValueError::new_err(format!("unknown ellipse mode: {mode:?}")))?;
        graphics_record_command(self.entity, DrawCommand::EllipseMode(mode))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn stroke_cap(&self, cap: &str) -> PyResult<()> {
        let cap = StrokeCapMode::parse(cap)
            .ok_or_else(|| PyValueError::new_err(format!("unknown stroke cap: {cap:?}")))?;
        graphics_record_command(self.entity, DrawCommand::StrokeCap(cap))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn stroke_join(&self, join: &str) -> PyResult<()> {
        let join = StrokeJoinMode::parse(join)
            .ok_or_else(|| PyValueError::new_err(format!("unknown stroke join: {join:?}")))?;
        graphics_record_command(self.entity, DrawCommand::StrokeJoin(join))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Draws a rectangle. Accepts `(x, y, w, h)`, `(x, y, w, h, radius)` for a
    /// uniform corner radius, or `(x, y, w, h, tl, tr, br, bl)` for per-corner
    /// radii — matching Processing's `rect()` overloads.
    #[pyo3(signature = (*args))]
    pub fn rect(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let (x, y, w, h, tl, tr, br, bl) = match args.len() {
            4 => {
                let x = args.get_item(0)?.extract()?;
                let y = args.get_item(1)?.extract()?;
                let w = args.get_item(2)?.extract()?;
                let h = args.get_item(3)?.extract()?;
                (x, y, w, h, 0.0, 0.0, 0.0, 0.0)
            }
            5 => {
                let x = args.get_item(0)?.extract()?;
                let y = args.get_item(1)?.extract()?;
                let w = args.get_item(2)?.extract()?;
                let h = args.get_item(3)?.extract()?;
                let r = args.get_item(4)?.extract()?;
                (x, y, w, h, r, r, r, r)
            }
            8 => {
                let x = args.get_item(0)?.extract()?;
                let y = args.get_item(1)?.extract()?;
                let w = args.get_item(2)?.extract()?;
                let h = args.get_item(3)?.extract()?;
                let tl = args.get_item(4)?.extract()?;
                let tr = args.get_item(5)?.extract()?;
                let br = args.get_item(6)?.extract()?;
                let bl = args.get_item(7)?.extract()?;
                (x, y, w, h, tl, tr, br, bl)
            }
            n => {
                return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                    "rect() takes 4, 5, or 8 arguments ({n} given)"
                )));
            }
        };
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

    pub fn ellipse(&self, cx: f32, cy: f32, w: f32, h: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Ellipse { cx, cy, w, h })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn circle(&self, cx: f32, cy: f32, d: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Ellipse { cx, cy, w: d, h: d })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Line { x1, y1, x2, y2 })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn triangle(&self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Triangle {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn quad(
        &self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x4: f32,
        y4: f32,
    ) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Quad {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
                x4,
                y4,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn point(&self, x: f32, y: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Point { x, y })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn square(&self, x: f32, y: f32, s: f32) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Rect {
                x,
                y,
                w: s,
                h: s,
                radii: [0.0; 4],
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn arc(
        &self,
        cx: f32,
        cy: f32,
        w: f32,
        h: f32,
        start: f32,
        stop: f32,
        mode: &str,
    ) -> PyResult<()> {
        let mode = ArcMode::parse(mode)
            .ok_or_else(|| PyValueError::new_err(format!("unknown arc mode: {mode:?}")))?;
        graphics_record_command(
            self.entity,
            DrawCommand::Arc {
                cx,
                cy,
                w,
                h,
                start,
                stop,
                mode,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn bezier(
        &self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x4: f32,
        y4: f32,
    ) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Bezier {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
                x4,
                y4,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn curve(
        &self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x4: f32,
        y4: f32,
    ) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Curve {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
                x4,
                y4,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn begin_shape(&self, kind: &str) -> PyResult<()> {
        let kind = ShapeKind::parse(kind)
            .ok_or_else(|| PyValueError::new_err(format!("unknown shape kind: {kind:?}")))?;
        graphics_record_command(self.entity, DrawCommand::BeginShape { kind })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn end_shape(&self, close: bool) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::EndShape { close })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn vertex(&self, x: f32, y: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::ShapeVertex { x, y })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn bezier_vertex(
        &self,
        cx1: f32,
        cy1: f32,
        cx2: f32,
        cy2: f32,
        x: f32,
        y: f32,
    ) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::ShapeBezierVertex {
                cx1,
                cy1,
                cx2,
                cy2,
                x,
                y,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn quadratic_vertex(&self, cx: f32, cy: f32, x: f32, y: f32) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::ShapeQuadraticVertex { cx, cy, x, y },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn curve_vertex(&self, x: f32, y: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::ShapeCurveVertex { x, y })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn begin_contour(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::BeginContour)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn end_contour(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::EndContour)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    // --- Font ---

    pub fn load_font(&self, path: &str) -> PyResult<Font> {
        font_load(path)
            .map(|entity| Font { entity })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn create_font(&self, name: &str) -> PyResult<Font> {
        font_create(name)
            .map(|entity| Font { entity })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn list_fonts(&self) -> PyResult<Vec<String>> {
        font_list().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (font=None))]
    pub fn text_font(&self, font: Option<&Font>) -> PyResult<()> {
        graphics_text_font(self.entity, font.map(|f| f.entity))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    // --- Text ---

    #[pyo3(signature = (content, x, y, *args, max_w=None, max_h=None))]
    pub fn text(
        &self,
        content: &str,
        x: f32,
        y: f32,
        args: &Bound<'_, pyo3::types::PyTuple>,
        max_w: Option<f32>,
        max_h: Option<f32>,
    ) -> PyResult<()> {
        // text(content, x, y) or text(content, x, y, z) or text(content, x, y, max_w, max_h)
        let (z, mw, mh) = match args.len() {
            0 => (0.0, max_w, max_h),
            1 => {
                let z: f32 = args.get_item(0)?.extract()?;
                (z, max_w, max_h)
            }
            2 => {
                let w: f32 = args.get_item(0)?.extract()?;
                let h: f32 = args.get_item(1)?.extract()?;
                (0.0, Some(w), Some(h))
            }
            3 => {
                let z: f32 = args.get_item(0)?.extract()?;
                let w: f32 = args.get_item(1)?.extract()?;
                let h: f32 = args.get_item(2)?.extract()?;
                (z, Some(w), Some(h))
            }
            _ => {
                return Err(PyRuntimeError::new_err(
                    "text() takes 3-6 positional arguments",
                ));
            }
        };
        graphics_record_command(
            self.entity,
            DrawCommand::Text {
                content: content.to_string(),
                x,
                y,
                z,
                max_w: mw,
                max_h: mh,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn text_style(&self, style: &str) -> PyResult<()> {
        use processing::prelude::TextStyle;
        let style = TextStyle::parse(style)
            .ok_or_else(|| PyValueError::new_err(format!("unknown text style: {style:?}")))?;
        graphics_text_style(self.entity, style as u8)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (content, x, y, max_w=None, max_h=None))]
    pub fn text_bounds(
        &self,
        content: &str,
        x: f32,
        y: f32,
        max_w: Option<f32>,
        max_h: Option<f32>,
    ) -> PyResult<(f32, f32, f32, f32)> {
        graphics_text_bounds(self.entity, content, x, y, max_w, max_h)
            .map(|b| (b[0], b[1], b[2], b[3]))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn text_weight(&self, weight: f32) -> PyResult<()> {
        graphics_text_weight(self.entity, weight)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn text_variation(&self, tag: &str, value: f32) -> PyResult<()> {
        graphics_text_variation(self.entity, tag, value)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn clear_text_variations(&self) -> PyResult<()> {
        graphics_clear_text_variations(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Enable/configure an OpenType font feature.
    /// text_feature("smcp")          -> enable (value=1)
    /// text_feature("smcp", True)    -> enable (value=1)
    /// text_feature("smcp", False)   -> disable (value=0)
    /// text_feature("salt", 3)       -> select alternate 3
    #[pyo3(signature = (tag, value=None))]
    pub fn text_feature(&self, tag: &str, value: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        let v: u16 = match value {
            None => 1,
            Some(val) => {
                if let Ok(b) = val.extract::<bool>() {
                    if b { 1 } else { 0 }
                } else if let Ok(i) = val.extract::<u16>() {
                    i
                } else {
                    return Err(PyRuntimeError::new_err(
                        "text_feature value must be bool or int",
                    ));
                }
            }
        };
        graphics_text_feature(self.entity, tag, v)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn no_text_feature(&self, tag: &str) -> PyResult<()> {
        graphics_no_text_feature(self.entity, tag)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn clear_text_features(&self) -> PyResult<()> {
        graphics_clear_text_features(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Extract glyph outlines as path commands (one list per glyph).
    /// Each command is a tuple: ("M", x, y), ("L", x, y), ("Q", cx, cy, x, y),
    /// ("C", cx1, cy1, cx2, cy2, x, y), or ("Z",).
    pub fn text_to_paths(&self, content: &str, x: f32, y: f32) -> PyResult<Vec<Vec<Py<PyAny>>>> {
        let paths = graphics_text_to_paths(self.entity, content, x, y)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Python::attach(|py| Ok(path_commands_to_py(py, paths)))
    }

    /// Extract glyph outlines as per-contour path commands.
    /// Each contour (MoveTo...Close sequence) is a separate list.
    /// Commands use the same tuple shapes as `text_to_paths`.
    pub fn text_to_contours(&self, content: &str, x: f32, y: f32) -> PyResult<Vec<Vec<Py<PyAny>>>> {
        let contours = graphics_text_to_contours(self.entity, content, x, y)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Python::attach(|py| Ok(path_commands_to_py(py, contours)))
    }

    /// Sample points along text outlines.
    /// Returns list of [x, y] points.
    #[pyo3(signature = (content, x, y, sample_factor=None))]
    pub fn text_to_points(
        &self,
        content: &str,
        x: f32,
        y: f32,
        sample_factor: Option<f32>,
    ) -> PyResult<Vec<[f32; 2]>> {
        graphics_text_to_points(self.entity, content, x, y, sample_factor)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Generate a 3D extruded mesh from text outlines.
    pub fn text_to_model(&self, content: &str, x: f32, y: f32, depth: f32) -> PyResult<Geometry> {
        let mesh = graphics_text_to_model(self.entity, content, x, y, depth)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let entity =
            geometry_create_from_mesh(mesh).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Geometry { entity })
    }

    /// Set per-glyph colors for the next text() call.
    ///
    /// `colors` is a list of color objects (as built by `color(...)`); they are
    /// cycled across the glyphs of the next `text()` call.
    pub fn text_glyph_colors(&self, colors: Vec<PyRef<crate::color::PyColor>>) -> PyResult<()> {
        let colors: Vec<bevy::color::Color> = colors.iter().map(|c| c.0).collect();
        graphics_text_glyph_colors(self.entity, colors)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn text_size(&self, size: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::TextSize(size))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (h, v=None))]
    pub fn text_align(&self, h: &str, v: Option<&str>) -> PyResult<()> {
        use processing::prelude::{TextAlignH, TextAlignV};
        let h = TextAlignH::parse(h)
            .ok_or_else(|| PyValueError::new_err(format!("unknown horizontal text align: {h:?}")))?;
        let v = match v {
            Some(v) => TextAlignV::parse(v).ok_or_else(|| {
                PyValueError::new_err(format!("unknown vertical text align: {v:?}"))
            })?,
            None => TextAlignV::default(),
        };
        graphics_record_command(self.entity, DrawCommand::TextAlign { h, v })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn text_leading(&self, leading: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::TextLeading(leading))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn text_wrap(&self, mode: &str) -> PyResult<()> {
        use processing::prelude::TextWrapMode;
        let mode = TextWrapMode::parse(mode)
            .ok_or_else(|| PyValueError::new_err(format!("unknown text wrap mode: {mode:?}")))?;
        graphics_record_command(self.entity, DrawCommand::TextWrap(mode))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Number of lines `content` wraps to.
    pub fn text_line_count(&self, content: &str) -> PyResult<usize> {
        graphics_text_line_count(self.entity, content)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Per-line info as a list of `(text, (x, y, w, h))` tuples.
    #[pyo3(signature = (content, x, y, max_w=None, max_h=None))]
    pub fn text_lines(
        &self,
        content: &str,
        x: f32,
        y: f32,
        max_w: Option<f32>,
        max_h: Option<f32>,
    ) -> PyResult<Vec<(String, (f32, f32, f32, f32))>> {
        graphics_text_lines(self.entity, content, x, y, max_w, max_h)
            .map(|lines| {
                lines
                    .into_iter()
                    .map(|li| (li.text, (li.rect[0], li.rect[1], li.rect[2], li.rect[3])))
                    .collect()
            })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Per-glyph bounding rects as a list of `(x, y, w, h)` tuples.
    #[pyo3(signature = (content, x, y, max_w=None, max_h=None))]
    pub fn text_glyph_rects(
        &self,
        content: &str,
        x: f32,
        y: f32,
        max_w: Option<f32>,
        max_h: Option<f32>,
    ) -> PyResult<Vec<(f32, f32, f32, f32)>> {
        graphics_text_glyph_rects(self.entity, content, x, y, max_w, max_h)
            .map(|glyphs| {
                glyphs
                    .into_iter()
                    .map(|g| (g.rect[0], g.rect[1], g.rect[2], g.rect[3]))
                    .collect()
            })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn text_width(&self, content: &str) -> PyResult<f32> {
        graphics_text_width(self.entity, content)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn text_ascent(&self) -> PyResult<f32> {
        graphics_text_ascent(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn text_descent(&self) -> PyResult<f32> {
        graphics_text_descent(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Loads an image from a file and returns an Image object.
    ///
    /// The path is relative to the sketch's assets directory.
    pub fn load_image(&self, file: &str) -> PyResult<Image> {
        match image_load(file) {
            Ok(image) => Ok(Image::wrap(image)),
            Err(e) => Err(PyRuntimeError::new_err(format!("{e}"))),
        }
    }

    /// Draws an image to the screen.
    ///
    /// Optional `d_width` and `d_height` resize the image on screen. If omitted,
    /// the image's original dimensions are used.
    ///
    /// Optional `sx`, `sy`, `s_width`, and `s_height` define a sub-region
    /// of the source image to draw, specified in pixels.
    ///
    /// Affected by `image_mode()`, `tint()`, and the current transform.
    #[pyo3(signature = (source, dx, dy, d_width=None, d_height=None, sx=None, sy=None, s_width=None, s_height=None))]
    pub fn image(
        &self,
        source: ImageRef,
        dx: f32,
        dy: f32,
        d_width: Option<f32>,
        d_height: Option<f32>,
        sx: Option<f32>,
        sy: Option<f32>,
        s_width: Option<f32>,
        s_height: Option<f32>,
    ) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Image {
                entity: source.entity,
                dx,
                dy,
                d_width,
                d_height,
                sx,
                sy,
                s_width,
                s_height,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Sets a tint color applied when drawing images.
    ///
    /// Accepts the same color arguments as `fill()`. The tint is multiplied
    /// with the image's pixel colors. Use `no_tint()` to remove.
    #[pyo3(signature = (*args))]
    pub fn tint(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let color = extract_color_with_mode(
            args,
            &graphics_get_color_mode(self.entity)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?,
        )?;
        graphics_record_command(self.entity, DrawCommand::Tint(color))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Removes the current tint color so images draw without color modification.
    pub fn no_tint(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::NoTint)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Changes how image position arguments are interpreted.
    ///
    /// - `CORNER` (default) — `dx`, `dy` is the top-left corner.
    /// - `CORNERS` — `dx`, `dy` and `d_width`, `d_height` are opposite corners.
    /// - `CENTER` — `dx`, `dy` is the center of the image.
    pub fn image_mode(&self, mode: &str) -> PyResult<()> {
        let mode = ShapeMode::parse(mode)
            .ok_or_else(|| PyValueError::new_err(format!("unknown image mode: {mode:?}")))?;
        graphics_record_command(self.entity, DrawCommand::ImageMode(mode))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn create_image(&self, width: u32, height: u32) -> PyResult<Image> {
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let data = vec![0u8; (width * height * 4) as usize];
        let entity = image_create(size, data, TextureFormat::Rgba8UnormSrgb)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Image::wrap(entity))
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

    pub fn push_style(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::PushStyle)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn pop_style(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::PopStyle)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn push(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::PushStyle)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        graphics_record_command(self.entity, DrawCommand::PushMatrix)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn pop(&self) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::PopStyle)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        graphics_record_command(self.entity, DrawCommand::PopMatrix)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (*args))]
    pub fn translate(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = if args.len() == 3 {
            extract_vec3(args)?
        } else {
            extract_vec2(args)?.extend(0.0)
        };
        graphics_record_command(self.entity, DrawCommand::Translate(v))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn rotate(&self, angle: f32) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Rotate {
                angle,
                axis: Vec3::Z,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn rotate_x(&self, angle: f32) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Rotate {
                angle,
                axis: Vec3::X,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn rotate_y(&self, angle: f32) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Rotate {
                angle,
                axis: Vec3::Y,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn rotate_z(&self, angle: f32) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Rotate {
                angle,
                axis: Vec3::Z,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (angle, *args))]
    pub fn rotate_axis(&self, angle: f32, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let axis = extract_vec3(args)?;
        graphics_record_command(self.entity, DrawCommand::Rotate { angle, axis })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Multiply the current matrix by another (Processing `applyMatrix`).
    /// Accepts 6 values (2D affine) or 16 values (3D, row-major).
    #[pyo3(signature = (*args))]
    pub fn apply_matrix(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let affine = affine_from_matrix_args(args)?;
        graphics_record_command(self.entity, DrawCommand::ApplyMatrix(affine))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Replace the current matrix (Processing `setMatrix`).
    /// Accepts 6 values (2D affine) or 16 values (3D, row-major).
    #[pyo3(signature = (*args))]
    pub fn set_matrix(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let affine = affine_from_matrix_args(args)?;
        graphics_record_command(self.entity, DrawCommand::ResetMatrix)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        graphics_record_command(self.entity, DrawCommand::ApplyMatrix(affine))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Return the current transformation matrix as 16 floats in row-major order.
    pub fn get_matrix(&self) -> PyResult<[f32; 16]> {
        let m = graphics_get_matrix(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(m.transpose().to_cols_array())
    }

    /// Screen-space x of a model-space coordinate (Processing `screenX`).
    #[pyo3(signature = (x, y, z=0.0))]
    pub fn screen_x(&self, x: f32, y: f32, z: f32) -> PyResult<f32> {
        graphics_screen_point(self.entity, Vec3::new(x, y, z))
            .map(|p| p.x)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Screen-space y of a model-space coordinate (Processing `screenY`).
    #[pyo3(signature = (x, y, z=0.0))]
    pub fn screen_y(&self, x: f32, y: f32, z: f32) -> PyResult<f32> {
        graphics_screen_point(self.entity, Vec3::new(x, y, z))
            .map(|p| p.y)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Screen-space depth of a model-space coordinate (Processing `screenZ`).
    #[pyo3(signature = (x, y, z=0.0))]
    pub fn screen_z(&self, x: f32, y: f32, z: f32) -> PyResult<f32> {
        graphics_screen_point(self.entity, Vec3::new(x, y, z))
            .map(|p| p.z)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// World-space x of a model-space coordinate (Processing `modelX`).
    #[pyo3(signature = (x, y, z=0.0))]
    pub fn model_x(&self, x: f32, y: f32, z: f32) -> PyResult<f32> {
        graphics_model_point(self.entity, Vec3::new(x, y, z))
            .map(|p| p.x)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// World-space y of a model-space coordinate (Processing `modelY`).
    #[pyo3(signature = (x, y, z=0.0))]
    pub fn model_y(&self, x: f32, y: f32, z: f32) -> PyResult<f32> {
        graphics_model_point(self.entity, Vec3::new(x, y, z))
            .map(|p| p.y)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// World-space z of a model-space coordinate (Processing `modelZ`).
    #[pyo3(signature = (x, y, z=0.0))]
    pub fn model_z(&self, x: f32, y: f32, z: f32) -> PyResult<f32> {
        graphics_model_point(self.entity, Vec3::new(x, y, z))
            .map(|p| p.z)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// World-space x of a screen coordinate at the given depth (Processing `worldX`).
    #[pyo3(signature = (sx, sy, depth=0.0))]
    pub fn world_x(&self, sx: f32, sy: f32, depth: f32) -> PyResult<f32> {
        graphics_world_from_screen(self.entity, sx, sy, depth)
            .map(|p| p.x)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// World-space y of a screen coordinate at the given depth (Processing `worldY`).
    #[pyo3(signature = (sx, sy, depth=0.0))]
    pub fn world_y(&self, sx: f32, sy: f32, depth: f32) -> PyResult<f32> {
        graphics_world_from_screen(self.entity, sx, sy, depth)
            .map(|p| p.y)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// World-space z of a screen coordinate at the given depth (Processing `worldZ`).
    #[pyo3(signature = (sx, sy, depth=0.0))]
    pub fn world_z(&self, sx: f32, sy: f32, depth: f32) -> PyResult<f32> {
        graphics_world_from_screen(self.entity, sx, sy, depth)
            .map(|p| p.z)
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

    pub fn draw_cylinder(&self, radius: f32, height: f32, detail: u32) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Cylinder {
                radius,
                height,
                detail,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn draw_cone(&self, radius: f32, height: f32, detail: u32) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Cone {
                radius,
                height,
                detail,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn draw_torus(
        &self,
        radius: f32,
        tube_radius: f32,
        major_segments: u32,
        minor_segments: u32,
    ) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Torus {
                radius,
                tube_radius,
                major_segments,
                minor_segments,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn draw_plane(&self, width: f32, height: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Plane { width, height })
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn draw_capsule(&self, radius: f32, length: f32, detail: u32) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Capsule {
                radius,
                length,
                detail,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn draw_conical_frustum(
        &self,
        radius_top: f32,
        radius_bottom: f32,
        height: f32,
        detail: u32,
    ) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::ConicalFrustum {
                radius_top,
                radius_bottom,
                height,
                detail,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn draw_tetrahedron(&self, radius: f32) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Tetrahedron { radius })
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

    pub fn particles(
        &self,
        particles: &crate::particles::Particles,
        geometry: &Geometry,
    ) -> PyResult<()> {
        graphics_record_command(
            self.entity,
            DrawCommand::Particles {
                particles: particles.entity,
                geometry: geometry.entity,
            },
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn material(&self, material: &crate::material::Material) -> PyResult<()> {
        graphics_record_command(self.entity, DrawCommand::Material(material.entity))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (*args))]
    pub fn scale(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = if args.len() == 3 {
            extract_vec3(args)?
        } else if args.len() == 1 {
            match args.get_item(0)?.extract::<f32>() {
                Ok(s) => Vec3::splat(s),
                Err(_) => extract_vec2(args)?.extend(1.0),
            }
        } else {
            extract_vec2(args)?.extend(1.0)
        };
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

    /// Composites a source onto this graphics with a blend mode (Processing
    /// `blend`). `src` is any sampleable input — an `Image`, webcam, or another
    /// `Graphics` (which is flushed first). Optional `src_rect`/`dst_rect` are
    /// `(x, y, w, h)` pixel regions (default full extent); the source is scaled
    /// into the dst region.
    #[pyo3(signature = (src, mode, *, src_rect=None, dst_rect=None, opacity=1.0))]
    pub fn blend(
        &self,
        src: &Bound<'_, PyAny>,
        mode: &PyBlendMode,
        src_rect: Option<[f32; 4]>,
        dst_rect: Option<[f32; 4]>,
        opacity: f32,
    ) -> PyResult<()> {
        let code = mode.composite_mode().ok_or_else(|| {
            PyValueError::new_err(
                "blend(): custom blend modes are unsupported here; use a named mode \
                 (BLEND, ADD, SUBTRACT, DARKEST, LIGHTEST, DIFFERENCE, EXCLUSION, \
                 MULTIPLY, SCREEN, REPLACE)",
            )
        })?;
        let (src_entity, flush) = resolve_composite_source(src)?;
        if let Some(g) = flush {
            graphics_flush(g).map_err(rt_err)?;
        }
        let sr = normalize_src_rect(src_entity, src_rect)?;
        let dr = normalize_rect(dst_rect, self.width as f32, self.height as f32);
        composite_apply(self.entity, src_entity, code, sr, dr, opacity)
    }

    /// Copies a source onto this graphics, replacing pixels (Processing `copy`).
    /// `src` may be an `Image`, webcam, or `Graphics`. See `blend` for the
    /// `src_rect`/`dst_rect` semantics.
    #[pyo3(signature = (src, *, src_rect=None, dst_rect=None))]
    pub fn copy(
        &self,
        src: &Bound<'_, PyAny>,
        src_rect: Option<[f32; 4]>,
        dst_rect: Option<[f32; 4]>,
    ) -> PyResult<()> {
        let (src_entity, flush) = resolve_composite_source(src)?;
        if let Some(g) = flush {
            graphics_flush(g).map_err(rt_err)?;
        }
        let sr = normalize_src_rect(src_entity, src_rect)?;
        let dr = normalize_rect(dst_rect, self.width as f32, self.height as f32);
        composite_apply(self.entity, src_entity, COMPOSITE_MODE_REPLACE, sr, dr, 1.0)
    }

    /// Feedback pass: re-samples this graphics' previous frame with a
    /// zoom/rotate/offset transform and a per-frame `decay`, writing it back.
    /// On a context you don't clear each frame, call this at the start of
    /// `draw()` and then draw new content on top to get feedback trails.
    #[pyo3(signature = (*, decay=0.95, zoom=1.0, angle=0.0, offset=(0.0, 0.0)))]
    pub fn feedback(
        &self,
        decay: f32,
        zoom: f32,
        angle: f32,
        offset: (f32, f32),
    ) -> PyResult<()> {
        use shader_value::ShaderValue;
        let filter = filter_feedback().map_err(rt_err)?;
        filter_set(filter, "decay", ShaderValue::Float(decay)).map_err(rt_err)?;
        filter_set(filter, "zoom", ShaderValue::Float(zoom)).map_err(rt_err)?;
        filter_set(filter, "angle", ShaderValue::Float(angle)).map_err(rt_err)?;
        filter_set(filter, "offset", ShaderValue::Float2([offset.0, offset.1])).map_err(rt_err)?;
        graphics_apply_filter(self.entity, filter).map_err(rt_err)
    }

    /// Applies a mask source's brightness as this graphics' alpha channel
    /// (Processing `mask`). `mask` may be an `Image`, webcam, or `Graphics`.
    pub fn mask(&self, mask: &Bound<'_, PyAny>) -> PyResult<()> {
        let (src_entity, flush) = resolve_composite_source(mask)?;
        if let Some(g) = flush {
            graphics_flush(g).map_err(rt_err)?;
        }
        composite_apply(
            self.entity,
            src_entity,
            COMPOSITE_MODE_MASK,
            [0.0, 0.0, 1.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            1.0,
        )
    }

    #[pyo3(name = "color_mode", signature = (mode, max1=None, max2=None, max3=None, max_alpha=None))]
    pub fn set_color_mode<'py>(
        &self,
        mode: &str,
        max1: Option<&Bound<'py, PyAny>>,
        max2: Option<&Bound<'py, PyAny>>,
        max3: Option<&Bound<'py, PyAny>>,
        max_alpha: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<()> {
        let space = crate::color::ColorSpace::parse(mode)
            .ok_or_else(|| PyValueError::new_err(format!("unknown color space: {mode:?}")))?;
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

    pub fn flush(&self) -> PyResult<()> {
        graphics_flush(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
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

    pub fn orbit_camera(&self) -> PyResult<()> {
        graphics_orbit_camera(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn free_camera(&self) -> PyResult<()> {
        graphics_free_camera(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn pan_camera(&self) -> PyResult<()> {
        graphics_pan_camera(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn disable_camera(&self) -> PyResult<()> {
        graphics_disable_camera_controller(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn camera_distance(&self, distance: f32) -> PyResult<()> {
        camera_set_distance(self.entity, distance)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (*args))]
    pub fn camera_center(&self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let v = extract_vec3(args)?;
        camera_set_center(self.entity, v).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn camera_min_distance(&self, min: f32) -> PyResult<()> {
        camera_set_min_distance(self.entity, min)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn camera_max_distance(&self, max: f32) -> PyResult<()> {
        camera_set_max_distance(self.entity, max)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn camera_speed(&self, speed: f32) -> PyResult<()> {
        camera_set_speed(self.entity, speed).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn camera_reset(&self) -> PyResult<()> {
        camera_reset(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
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
        input::mouse_x(self.surface.entity, self.width)
    }

    #[getter]
    fn mouse_y(&self) -> PyResult<f32> {
        input::mouse_y(self.surface.entity, self.height)
    }

    #[getter]
    fn pmouse_x(&self) -> PyResult<f32> {
        input::pmouse_x(self.surface.entity, self.width)
    }

    #[getter]
    fn pmouse_y(&self) -> PyResult<f32> {
        input::pmouse_y(self.surface.entity, self.height)
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

#[cfg(feature = "cuda")]
#[pymethods]
impl Graphics {
    pub fn cuda(&self) -> PyResult<CudaImage> {
        processing_cuda::cuda_export(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(CudaImage::new(self.entity))
    }

    pub fn update_from(&self, obj: &Bound<'_, pyo3::PyAny>) -> PyResult<()> {
        cuda_import_from_interface(self.entity, obj)
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
