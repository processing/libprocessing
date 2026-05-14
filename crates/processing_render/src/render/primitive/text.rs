use std::borrow::Cow;

use bevy::prelude::*;
use bevy::mesh::{Indices, VertexAttributeValues};
use lyon::{
    geom::Point,
    path::Path,
    tessellation::{
        FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator, VertexId,
        geometry_builder::{FillGeometryBuilder, GeometryBuilder, GeometryBuilderError},
    },
};
use parley::{
    Alignment, AlignmentOptions, FontContext, Layout, LayoutContext, PositionedLayoutItem,
    StyleProperty,
    style::{
        FontFamily, FontFeature, FontSettings, FontStack, FontStyle as ParleyFontStyle,
        FontVariation, FontWeight as ParleyFontWeight, LineHeight, WordBreakStrength,
    },
};
use skrifa::{
    instance::{LocationRef, NormalizedCoord, Size},
    outline::{DrawSettings, OutlinePen},
    FontRef, MetadataProvider,
};

use crate::render::{
    RenderState,
    command::{TextAlignH, TextAlignV, TextStyle, TextWrapMode},
    mesh_builder::MeshBuilder,
};
use crate::text::font::{DEFAULT_FONT_FAMILY, TextContext};

/// A path command for text outline data.
#[derive(Debug, Clone)]
pub enum PathCommand {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadTo { cx: f32, cy: f32, x: f32, y: f32 },
    CubicTo { cx1: f32, cy1: f32, cx2: f32, cy2: f32, x: f32, y: f32 },
    Close,
}

/// Text layout parameters.
pub struct TextParams<'a> {
    pub text_size: f32,
    pub align_h: TextAlignH,
    pub align_v: TextAlignV,
    pub leading: Option<f32>,
    pub max_w: Option<f32>,
    pub max_h: Option<f32>,
    pub wrap: TextWrapMode,
    pub font_family: Option<&'a str>,
    pub text_style: TextStyle,
    pub text_weight: Option<f32>,
    pub text_variations: &'a [([u8; 4], f32)],
    pub text_features: &'a [([u8; 4], u16)],
    pub glyph_colors: Option<&'a [Color]>,
}

/// Owned [`TextParams`]: a `RenderState` snapshot that outlives the borrow.
pub struct OwnedTextParams {
    pub text_size: f32,
    pub align_h: TextAlignH,
    pub align_v: TextAlignV,
    pub leading: Option<f32>,
    pub max_w: Option<f32>,
    pub max_h: Option<f32>,
    pub wrap: TextWrapMode,
    pub font_family: Option<String>,
    pub text_style: TextStyle,
    pub text_weight: Option<f32>,
    pub text_variations: Vec<([u8; 4], f32)>,
    pub text_features: Vec<([u8; 4], u16)>,
    pub glyph_colors: Option<Vec<Color>>,
}

impl OwnedTextParams {
    /// Snapshot a `RenderState`'s text state. `glyph_colors` is left unset; the
    /// draw path fills it in, measurement queries don't need it.
    pub fn from_render_state(state: &RenderState, max_w: Option<f32>, max_h: Option<f32>) -> Self {
        Self {
            text_size: state.text_size,
            align_h: state.text_align_h,
            align_v: state.text_align_v,
            leading: state.text_leading,
            max_w,
            max_h,
            wrap: state.text_wrap,
            font_family: state.text_font_family.clone(),
            text_style: state.text_style,
            text_weight: state.text_weight,
            text_variations: state.text_variations.clone(),
            text_features: state.text_features.clone(),
            glyph_colors: None,
        }
    }

    /// Borrow as a [`TextParams`].
    pub fn as_params(&self) -> TextParams<'_> {
        TextParams {
            text_size: self.text_size,
            align_h: self.align_h,
            align_v: self.align_v,
            leading: self.leading,
            max_w: self.max_w,
            max_h: self.max_h,
            wrap: self.wrap,
            font_family: self.font_family.as_deref(),
            text_style: self.text_style,
            text_weight: self.text_weight,
            text_variations: &self.text_variations,
            text_features: &self.text_features,
            glyph_colors: self.glyph_colors.as_deref(),
        }
    }
}

/// Tessellate text into a mesh (fill).
pub fn text(mesh: &mut Mesh, content: &str, x: f32, y: f32, color: Color, params: &TextParams, text_cx: &TextContext) {
    if content.is_empty() {
        return;
    }

    text_cx.with(|font_cx, layout_cx| {
        let layout = build_layout(font_cx, layout_cx, content, color, params);
        let (base_x, base_y) = compute_text_origin(&layout, x, y, params.align_v);
        tessellate_layout(mesh, &layout, base_x, base_y, params.max_h, params.glyph_colors);
    });
}

/// Tessellate text outlines as strokes into a mesh.
pub fn text_stroke(mesh: &mut Mesh, content: &str, x: f32, y: f32, color: Color, stroke_weight: f32, params: &TextParams, text_cx: &TextContext) {
    if content.is_empty() {
        return;
    }

    text_cx.with(|font_cx, layout_cx| {
        let layout = build_layout(font_cx, layout_cx, content, color, params);
        let (base_x, base_y) = compute_text_origin(&layout, x, y, params.align_v);
        stroke_layout(mesh, &layout, base_x, base_y, color, stroke_weight, params.max_h);
    });
}

/// Measure the width of text.
pub fn text_width(content: &str, params: &TextParams, text_cx: &TextContext) -> f32 {
    if content.is_empty() {
        return 0.0;
    }

    text_cx.with(|font_cx, layout_cx| {
        let layout = build_layout(font_cx, layout_cx, content, Color::BLACK, params);

        let mut max_width: f32 = 0.0;
        for line_idx in 0..layout.len() {
            if let Some(line) = layout.get(line_idx) {
                max_width = max_width.max(line.metrics().advance);
            }
        }
        max_width
    })
}

/// Font ascent for the current text size.
pub fn text_ascent(params: &TextParams, text_cx: &TextContext) -> f32 {
    text_cx.with(|font_cx, layout_cx| {
        let layout = build_layout(font_cx, layout_cx, "X", Color::BLACK, params);
        layout
            .get(0)
            .map(|line| line.metrics().ascent)
            .unwrap_or(0.0)
    })
}

/// Font descent for the current text size.
pub fn text_descent(params: &TextParams, text_cx: &TextContext) -> f32 {
    text_cx.with(|font_cx, layout_cx| {
        let layout = build_layout(font_cx, layout_cx, "X", Color::BLACK, params);
        layout
            .get(0)
            .map(|line| line.metrics().descent)
            .unwrap_or(0.0)
    })
}

/// Bounding box of text as `[x, y, width, height]`.
pub fn text_bounds(content: &str, x: f32, y: f32, params: &TextParams, text_cx: &TextContext) -> [f32; 4] {
    if content.is_empty() {
        return [x, y, 0.0, 0.0];
    }

    text_cx.with(|font_cx, layout_cx| {
        let layout = build_layout(font_cx, layout_cx, content, Color::BLACK, params);

        // the text origin is the layout's top-left corner
        let (box_x, box_y) = compute_text_origin(&layout, x, y, params.align_v);
        let width = layout.width();
        let height = match params.max_h {
            Some(h) => layout.height().min(h),
            None => layout.height(),
        };

        [box_x, box_y, width, height]
    })
}

/// A single laid-out line.
#[derive(Debug, Clone)]
pub struct TextLineInfo {
    pub text: String,
    /// `[x, y, width, height]`.
    pub rect: [f32; 4],
}

/// A single laid-out glyph.
#[derive(Debug, Clone)]
pub struct TextGlyphInfo {
    /// `[x, y, width, height]`.
    pub rect: [f32; 4],
}

/// Number of lines after layout.
pub fn text_line_count(content: &str, params: &TextParams, text_cx: &TextContext) -> usize {
    if content.is_empty() {
        return 0;
    }
    text_cx.with(|font_cx, layout_cx| {
        let layout = build_layout(font_cx, layout_cx, content, Color::BLACK, params);
        layout.len()
    })
}

/// Per-line text and bounding rects.
pub fn text_lines(
    content: &str,
    x: f32,
    y: f32,
    params: &TextParams,
    text_cx: &TextContext,
) -> Vec<TextLineInfo> {
    if content.is_empty() {
        return Vec::new();
    }
    text_cx.with(|font_cx, layout_cx| {
        let layout = build_layout(font_cx, layout_cx, content, Color::BLACK, params);
        let (base_x, base_y) = compute_text_origin(&layout, x, y, params.align_v);
        let mut result = Vec::new();

        for line_idx in 0..layout.len() {
            let Some(line) = layout.get(line_idx) else {
                continue;
            };
            let metrics = line.metrics();

            if let Some(h) = params.max_h {
                if metrics.baseline + metrics.descent > h {
                    break;
                }
            }

            let line_y = base_y + metrics.baseline - metrics.ascent;
            let line_text = &content[line.text_range()];

            result.push(TextLineInfo {
                text: line_text.to_string(),
                rect: [base_x, line_y, metrics.advance, metrics.ascent + metrics.descent],
            });
        }
        result
    })
}

/// Per-glyph bounding rects.
pub fn text_glyph_rects(
    content: &str,
    x: f32,
    y: f32,
    params: &TextParams,
    text_cx: &TextContext,
) -> Vec<TextGlyphInfo> {
    if content.is_empty() {
        return Vec::new();
    }
    text_cx.with(|font_cx, layout_cx| {
        let layout = build_layout(font_cx, layout_cx, content, Color::BLACK, params);
        let (base_x, base_y) = compute_text_origin(&layout, x, y, params.align_v);
        let mut result = Vec::new();

        for line_idx in 0..layout.len() {
            let Some(line) = layout.get(line_idx) else {
                continue;
            };
            let metrics = line.metrics();

            if let Some(h) = params.max_h {
                if metrics.baseline + metrics.descent > h {
                    break;
                }
            }

            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                let run = glyph_run.run();
                let font_size = run.font_size();
                for glyph in glyph_run.positioned_glyphs() {
                    let gx = base_x + glyph.x;
                    let gy = base_y + glyph.y - metrics.ascent;
                    result.push(TextGlyphInfo {
                        rect: [gx, gy, glyph.advance, font_size],
                    });
                }
            }
        }
        result
    })
}

/// Extract glyph outlines as path commands (one vec per glyph).
pub fn text_to_paths(
    content: &str,
    x: f32,
    y: f32,
    params: &TextParams,
    text_cx: &TextContext,
) -> Vec<Vec<PathCommand>> {
    if content.is_empty() {
        return Vec::new();
    }

    text_cx.with(|font_cx, layout_cx| {
        let layout = build_layout(font_cx, layout_cx, content, Color::BLACK, params);
        let (base_x, base_y) = compute_text_origin(&layout, x, y, params.align_v);
        extract_glyph_path_commands(&layout, base_x, base_y, params.max_h)
    })
}

/// Default `sample_factor` for [`text_to_points`].
pub const DEFAULT_SAMPLE_FACTOR: f32 = 0.1;

/// Sample points along text outlines. Higher `sample_factor` = more points;
/// see [`DEFAULT_SAMPLE_FACTOR`].
pub fn text_to_points(
    content: &str,
    x: f32,
    y: f32,
    sample_factor: f32,
    params: &TextParams,
    text_cx: &TextContext,
) -> Vec<[f32; 2]> {
    if content.is_empty() {
        return Vec::new();
    }

    text_cx.with(|font_cx, layout_cx| {
        let layout = build_layout(font_cx, layout_cx, content, Color::BLACK, params);
        let (base_x, base_y) = compute_text_origin(&layout, x, y, params.align_v);
        let glyph_paths = extract_glyph_lyon_paths(&layout, base_x, base_y, params.max_h);

        let step = sample_factor.max(0.001);
        let mut points = Vec::new();

        for path in &glyph_paths {
            for event in path.iter() {
                use lyon::path::Event;
                match event {
                    Event::Begin { at } => {
                        points.push([at.x, at.y]);
                    }
                    Event::Line { from, to } => {
                        let dx = to.x - from.x;
                        let dy = to.y - from.y;
                        let len = (dx * dx + dy * dy).sqrt();
                        let steps = (len * step).max(1.0) as usize;
                        for i in 1..=steps {
                            let t = i as f32 / steps as f32;
                            points.push([from.x + dx * t, from.y + dy * t]);
                        }
                    }
                    Event::Quadratic { from, ctrl, to } => {
                        let steps = (20.0 * step).max(2.0) as usize;
                        for i in 1..=steps {
                            let t = i as f32 / steps as f32;
                            let inv = 1.0 - t;
                            let px = inv * inv * from.x + 2.0 * inv * t * ctrl.x + t * t * to.x;
                            let py = inv * inv * from.y + 2.0 * inv * t * ctrl.y + t * t * to.y;
                            points.push([px, py]);
                        }
                    }
                    Event::Cubic { from, ctrl1, ctrl2, to } => {
                        let steps = (30.0 * step).max(2.0) as usize;
                        for i in 1..=steps {
                            let t = i as f32 / steps as f32;
                            let inv = 1.0 - t;
                            let px = inv * inv * inv * from.x
                                + 3.0 * inv * inv * t * ctrl1.x
                                + 3.0 * inv * t * t * ctrl2.x
                                + t * t * t * to.x;
                            let py = inv * inv * inv * from.y
                                + 3.0 * inv * inv * t * ctrl1.y
                                + 3.0 * inv * t * t * ctrl2.y
                                + t * t * t * to.y;
                            points.push([px, py]);
                        }
                    }
                    Event::End { .. } => {}
                }
            }
        }

        points
    })
}

/// 3D extruded mesh from text outlines: front and back faces plus side walls,
/// in Bevy's Y-up convention.
pub fn text_to_model(
    content: &str,
    x: f32,
    y: f32,
    depth: f32,
    params: &TextParams,
    text_cx: &TextContext,
) -> Mesh {
    let mut mesh = empty_mesh();

    if content.is_empty() || depth <= 0.0 {
        return mesh;
    }

    text_cx.with(|font_cx, layout_cx| {
        let layout = build_layout(font_cx, layout_cx, content, Color::WHITE, params);
        let (base_x, base_y) = compute_text_origin(&layout, x, y, params.align_v);
        let glyph_paths = extract_glyph_lyon_paths_yup(&layout, base_x, base_y, params.max_h);

        let half_depth = depth / 2.0;
        let mut fill_tess = FillTessellator::new();

        for path in &glyph_paths {
            // front face: z = +half_depth, normal +Z
            {
                let mut builder = Extrusion3DBuilder::new(&mut mesh, half_depth, [0.0, 0.0, 1.0]);
                let _ = fill_tess.tessellate_path(path, &FillOptions::default(), &mut builder);
            }

            // back face: z = -half_depth, normal -Z, with winding reversed below
            let back_indices_start = mesh
                .indices()
                .map(|i| match i {
                    Indices::U32(v) => v.len(),
                    _ => 0,
                })
                .unwrap_or(0);
            {
                let mut builder =
                    Extrusion3DBuilder::new(&mut mesh, -half_depth, [0.0, 0.0, -1.0]);
                let _ = fill_tess.tessellate_path(path, &FillOptions::default(), &mut builder);
            }

            if let Some(Indices::U32(indices)) = mesh.indices_mut() {
                let mut i = back_indices_start;
                while i + 2 < indices.len() {
                    indices.swap(i + 1, i + 2);
                    i += 3;
                }
            }

            // side walls: connect each contour's front vertices to its back ones
            let mut contour_points: Vec<Point<f32>> = Vec::new();

            for event in path.iter() {
                use lyon::path::Event;
                match event {
                    Event::Begin { at } => {
                        contour_points.clear();
                        contour_points.push(at);
                    }
                    Event::Line { from: _, to } => {
                        contour_points.push(to);
                    }
                    Event::Quadratic { from, ctrl, to } => {
                        let steps = 8;
                        for s in 1..=steps {
                            let t = s as f32 / steps as f32;
                            let inv = 1.0 - t;
                            let px = inv * inv * from.x + 2.0 * inv * t * ctrl.x + t * t * to.x;
                            let py = inv * inv * from.y + 2.0 * inv * t * ctrl.y + t * t * to.y;
                            contour_points.push(Point::new(px, py));
                        }
                    }
                    Event::Cubic { from, ctrl1, ctrl2, to } => {
                        let steps = 12;
                        for s in 1..=steps {
                            let t = s as f32 / steps as f32;
                            let inv = 1.0 - t;
                            let px = inv * inv * inv * from.x
                                + 3.0 * inv * inv * t * ctrl1.x
                                + 3.0 * inv * t * t * ctrl2.x
                                + t * t * t * to.x;
                            let py = inv * inv * inv * from.y
                                + 3.0 * inv * inv * t * ctrl1.y
                                + 3.0 * inv * t * t * ctrl2.y
                                + t * t * t * to.y;
                            contour_points.push(Point::new(px, py));
                        }
                    }
                    Event::End { close, .. } => {
                        if close && contour_points.len() >= 2 {
                            for i in 0..contour_points.len() {
                                let j = (i + 1) % contour_points.len();
                                let p0 = contour_points[i];
                                let p1 = contour_points[j];

                                // outward normal of this edge
                                let dx = p1.x - p0.x;
                                let dy = p1.y - p0.y;
                                let len = (dx * dx + dy * dy).sqrt().max(1e-6);
                                let nx = -dy / len;
                                let ny = dx / len;
                                let normal = [nx, ny, 0.0];

                                // quad vertices: front-p0, front-p1, back-p1, back-p0
                                let base = vertex_count(&mesh) as u32;
                                push_vertex_3d(&mut mesh, [p0.x, p0.y, half_depth], normal);
                                push_vertex_3d(&mut mesh, [p1.x, p1.y, half_depth], normal);
                                push_vertex_3d(&mut mesh, [p1.x, p1.y, -half_depth], normal);
                                push_vertex_3d(&mut mesh, [p0.x, p0.y, -half_depth], normal);

                                if let Some(Indices::U32(indices)) = mesh.indices_mut() {
                                    indices.extend_from_slice(&[
                                        base,
                                        base + 1,
                                        base + 2,
                                        base,
                                        base + 2,
                                        base + 3,
                                    ]);
                                }
                            }
                        }
                        contour_points.clear();
                    }
                }
            }
        }
    });

    mesh
}

fn empty_mesh() -> Mesh {
    use bevy::render::mesh::PrimitiveTopology;
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new());
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new());
    mesh.insert_indices(Indices::U32(Vec::new()));
    mesh
}

fn vertex_count(mesh: &Mesh) -> usize {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .map(|a| match a {
            VertexAttributeValues::Float32x3(v) => v.len(),
            _ => 0,
        })
        .unwrap_or(0)
}

fn push_vertex_3d(mesh: &mut Mesh, position: [f32; 3], normal: [f32; 3]) {
    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        positions.push(position);
    }
    if let Some(VertexAttributeValues::Float32x3(normals)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
    {
        normals.push(normal);
    }
    if let Some(VertexAttributeValues::Float32x4(colors)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
    {
        colors.push([1.0, 1.0, 1.0, 1.0]);
    }
    if let Some(VertexAttributeValues::Float32x2(uvs)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
    {
        uvs.push([0.0, 0.0]);
    }
}

/// Places lyon fill output at a fixed Z depth and normal.
struct Extrusion3DBuilder<'a> {
    mesh: &'a mut Mesh,
    z: f32,
    normal: [f32; 3],
    begin_vertex_count: u32,
}

impl<'a> Extrusion3DBuilder<'a> {
    fn new(mesh: &'a mut Mesh, z: f32, normal: [f32; 3]) -> Self {
        Self {
            mesh,
            z,
            normal,
            begin_vertex_count: 0,
        }
    }
}

impl<'a> GeometryBuilder for Extrusion3DBuilder<'a> {
    fn begin_geometry(&mut self) {
        self.begin_vertex_count = vertex_count(self.mesh) as u32;
    }
    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        if let Some(Indices::U32(indices)) = self.mesh.indices_mut() {
            indices.push(a.to_usize() as u32);
            indices.push(b.to_usize() as u32);
            indices.push(c.to_usize() as u32);
        }
    }
    fn abort_geometry(&mut self) {}
}

impl<'a> FillGeometryBuilder for Extrusion3DBuilder<'a> {
    fn add_fill_vertex(&mut self, vertex: FillVertex) -> Result<VertexId, GeometryBuilderError> {
        let pos = vertex.position();
        let count = vertex_count(self.mesh);
        push_vertex_3d(self.mesh, [pos.x, pos.y, self.z], self.normal);
        Ok(VertexId::from_usize(count))
    }
}

/// Absolute position of the layout's top-left corner — parley's glyph
/// positions are measured against it. `y` is the first line's baseline for
/// `Baseline` align, otherwise the top/center/bottom of the text block.
fn compute_text_origin(layout: &Layout<Color>, x: f32, y: f32, align_v: TextAlignV) -> (f32, f32) {
    let total_height = layout.height();
    let first_baseline = layout
        .get(0)
        .map(|line| line.metrics().baseline)
        .unwrap_or(0.0);

    let y_offset = match align_v {
        TextAlignV::Baseline => y - first_baseline,
        TextAlignV::Top => y,
        TextAlignV::Center => y - total_height / 2.0,
        TextAlignV::Bottom => y - total_height,
    };

    (x, y_offset)
}

/// Extract text outlines, one `PathCommand` vec per contour.
pub fn text_to_contours(
    content: &str,
    x: f32,
    y: f32,
    params: &TextParams,
    text_cx: &TextContext,
) -> Vec<Vec<PathCommand>> {
    if content.is_empty() {
        return Vec::new();
    }

    text_cx.with(|font_cx, layout_cx| {
        let layout = build_layout(font_cx, layout_cx, content, Color::BLACK, params);
        let (base_x, base_y) = compute_text_origin(&layout, x, y, params.align_v);
        let lyon_paths = extract_glyph_lyon_paths(&layout, base_x, base_y, params.max_h);

        let mut contours = Vec::new();
        for path in &lyon_paths {
            let mut current_contour = Vec::new();
            for event in path.iter() {
                use lyon::path::Event;
                match event {
                    Event::Begin { at } => {
                        current_contour.push(PathCommand::MoveTo(at.x, at.y));
                    }
                    Event::Line { from: _, to } => {
                        current_contour.push(PathCommand::LineTo(to.x, to.y));
                    }
                    Event::Quadratic { from: _, ctrl, to } => {
                        current_contour.push(PathCommand::QuadTo {
                            cx: ctrl.x, cy: ctrl.y, x: to.x, y: to.y,
                        });
                    }
                    Event::Cubic { from: _, ctrl1, ctrl2, to } => {
                        current_contour.push(PathCommand::CubicTo {
                            cx1: ctrl1.x, cy1: ctrl1.y,
                            cx2: ctrl2.x, cy2: ctrl2.y,
                            x: to.x, y: to.y,
                        });
                    }
                    Event::End { close, .. } => {
                        if close {
                            current_contour.push(PathCommand::Close);
                        }
                        if !current_contour.is_empty() {
                            contours.push(std::mem::take(&mut current_contour));
                        }
                    }
                }
            }
            if !current_contour.is_empty() {
                contours.push(current_contour);
            }
        }
        contours
    })
}

/// Extract glyph outlines as PathCommand vecs.
fn extract_glyph_path_commands(
    layout: &Layout<Color>,
    base_x: f32,
    base_y: f32,
    max_h: Option<f32>,
) -> Vec<Vec<PathCommand>> {
    let lyon_paths = extract_glyph_lyon_paths(layout, base_x, base_y, max_h);
    lyon_paths
        .into_iter()
        .map(|path| {
            let mut cmds = Vec::new();
            for event in path.iter() {
                use lyon::path::Event;
                match event {
                    Event::Begin { at } => cmds.push(PathCommand::MoveTo(at.x, at.y)),
                    Event::Line { from: _, to } => cmds.push(PathCommand::LineTo(to.x, to.y)),
                    Event::Quadratic { from: _, ctrl, to } => {
                        cmds.push(PathCommand::QuadTo {
                            cx: ctrl.x, cy: ctrl.y, x: to.x, y: to.y,
                        });
                    }
                    Event::Cubic { from: _, ctrl1, ctrl2, to } => {
                        cmds.push(PathCommand::CubicTo {
                            cx1: ctrl1.x, cy1: ctrl1.y,
                            cx2: ctrl2.x, cy2: ctrl2.y,
                            x: to.x, y: to.y,
                        });
                    }
                    Event::End { close, .. } => {
                        if close { cmds.push(PathCommand::Close); }
                    }
                }
            }
            cmds
        })
        .collect()
}

/// Extract glyph outlines as lyon `Path`s, one per glyph.
fn extract_glyph_lyon_paths(
    layout: &Layout<Color>,
    base_x: f32,
    base_y: f32,
    max_h: Option<f32>,
) -> Vec<Path> {
    let mut paths = Vec::new();

    for line_idx in 0..layout.len() {
        let Some(line) = layout.get(line_idx) else {
            continue;
        };

        if let Some(h) = max_h {
            let metrics = line.metrics();
            if metrics.baseline + metrics.descent > h {
                break;
            }
        }

        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };

            let run = glyph_run.run();
            let font_data = run.font();
            let font_size = run.font_size();
            let normalized_coords = run.normalized_coords();

            let Ok(font_ref) = FontRef::from_index(font_data.data.as_ref(), font_data.index)
            else {
                continue;
            };

            let outlines = font_ref.outline_glyphs();
            let skrifa_size = Size::new(font_size);
            let coords: Vec<NormalizedCoord> = normalized_coords
                .iter()
                .map(|&c| NormalizedCoord::from_bits(c))
                .collect();
            let location = LocationRef::new(&coords);

            for glyph in glyph_run.positioned_glyphs() {
                let glyph_id = skrifa::GlyphId::new(glyph.id);

                if let Some(outline_glyph) = outlines.get(glyph_id) {
                    let mut pen = LyonOutlinePen::new();
                    let settings = DrawSettings::unhinted(skrifa_size, location);
                    let _ = outline_glyph.draw(settings, &mut pen);

                    if let Some(path) = pen.build() {
                        let tx = base_x + glyph.x;
                        let ty = base_y + glyph.y;
                        paths.push(translate_path_flip_y(&path, tx, ty));
                    }
                }
            }
        }
    }

    paths
}

fn build_layout(
    font_cx: &mut FontContext,
    layout_cx: &mut LayoutContext<Color>,
    content: &str,
    color: Color,
    params: &TextParams,
) -> Layout<Color> {
    let mut builder = layout_cx.ranged_builder(font_cx, content, 1.0, false);

    let family = params.font_family.unwrap_or(DEFAULT_FONT_FAMILY);
    builder.push_default(StyleProperty::FontSize(params.text_size));
    builder.push_default(StyleProperty::FontStack(FontStack::Single(
        FontFamily::Named(Cow::Owned(family.to_string())),
    )));
    builder.push_default(StyleProperty::Brush(color));

    if let Some(line_height) = params.leading {
        builder.push_default(StyleProperty::LineHeight(LineHeight::Absolute(line_height)));
    }

    if matches!(params.wrap, TextWrapMode::Char) {
        builder.push_default(StyleProperty::WordBreak(WordBreakStrength::BreakAll));
    }

    // text_weight overrides the bold implied by text_style
    if let Some(weight) = params.text_weight {
        builder.push_default(StyleProperty::FontWeight(ParleyFontWeight::new(weight)));
        if matches!(params.text_style, TextStyle::Italic | TextStyle::BoldItalic) {
            builder.push_default(StyleProperty::FontStyle(ParleyFontStyle::Italic));
        }
    } else {
        match params.text_style {
            TextStyle::Normal => {}
            TextStyle::Italic => {
                builder.push_default(StyleProperty::FontStyle(ParleyFontStyle::Italic));
            }
            TextStyle::Bold => {
                builder.push_default(StyleProperty::FontWeight(ParleyFontWeight::BOLD));
            }
            TextStyle::BoldItalic => {
                builder.push_default(StyleProperty::FontStyle(ParleyFontStyle::Italic));
                builder.push_default(StyleProperty::FontWeight(ParleyFontWeight::BOLD));
            }
        }
    }

    if !params.text_variations.is_empty() {
        let vars: Vec<FontVariation> = params
            .text_variations
            .iter()
            .map(|&(tag, value)| FontVariation {
                tag: u32::from_be_bytes(tag),
                value,
            })
            .collect();
        builder.push_default(StyleProperty::FontVariations(FontSettings::List(
            Cow::Owned(vars),
        )));
    }

    if !params.text_features.is_empty() {
        let feats: Vec<FontFeature> = params
            .text_features
            .iter()
            .map(|&(tag, value)| FontFeature {
                tag: u32::from_be_bytes(tag),
                value,
            })
            .collect();
        builder.push_default(StyleProperty::FontFeatures(FontSettings::List(
            Cow::Owned(feats),
        )));
    }

    let mut layout = builder.build(content);

    let max_advance = params.max_w.unwrap_or(f32::MAX);
    layout.break_all_lines(Some(max_advance));

    let alignment = match params.align_h {
        TextAlignH::Left => Alignment::Start,
        TextAlignH::Center => Alignment::Center,
        TextAlignH::Right => Alignment::End,
    };
    layout.align(params.max_w, alignment, AlignmentOptions::default());

    layout
}

fn tessellate_layout(
    mesh: &mut Mesh,
    layout: &Layout<Color>,
    base_x: f32,
    base_y: f32,
    max_h: Option<f32>,
    glyph_colors: Option<&[Color]>,
) {
    let mut fill_tess = FillTessellator::new();
    let mut glyph_index: usize = 0;

    for line_idx in 0..layout.len() {
        let Some(line) = layout.get(line_idx) else {
            continue;
        };

        // stop once a line falls past max_h
        if let Some(h) = max_h {
            let metrics = line.metrics();
            if metrics.baseline + metrics.descent > h {
                break;
            }
        }

        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };

            let run = glyph_run.run();
            let font_data = run.font();
            let font_size = run.font_size();
            let normalized_coords = run.normalized_coords();
            let color = glyph_run.style().brush.clone();

            let Ok(font_ref) = FontRef::from_index(font_data.data.as_ref(), font_data.index) else {
                continue;
            };

            let outlines = font_ref.outline_glyphs();
            let skrifa_size = Size::new(font_size);

            // parley's i16 normalized coords -> skrifa's F2Dot14 NormalizedCoord
            let coords: Vec<NormalizedCoord> = normalized_coords
                .iter()
                .map(|&c| NormalizedCoord::from_bits(c))
                .collect();
            let location = LocationRef::new(&coords);

            for glyph in glyph_run.positioned_glyphs() {
                let glyph_color = glyph_colors
                    .filter(|colors| !colors.is_empty())
                    .map(|colors| colors[glyph_index % colors.len()])
                    .unwrap_or(color.clone());
                glyph_index += 1;

                let glyph_id = skrifa::GlyphId::new(glyph.id);

                if let Some(outline_glyph) = outlines.get(glyph_id) {
                    let mut pen = LyonOutlinePen::new();
                    let settings = DrawSettings::unhinted(skrifa_size, location);
                    let _ = outline_glyph.draw(settings, &mut pen);

                    if let Some(path) = pen.build() {
                        // font outlines are Y-up; translate_path_flip_y flips to Y-down
                        let tx = base_x + glyph.x;
                        let ty = base_y + glyph.y;

                        let translated = translate_path_flip_y(&path, tx, ty);

                        let mut builder = MeshBuilder::new(mesh, glyph_color.clone());
                        let _ = fill_tess.tessellate_path(
                            &translated,
                            &FillOptions::default(),
                            &mut builder,
                        );
                    }
                }
            }
        }
    }
}

fn stroke_layout(
    mesh: &mut Mesh,
    layout: &Layout<Color>,
    base_x: f32,
    base_y: f32,
    color: Color,
    stroke_weight: f32,
    max_h: Option<f32>,
) {
    let mut stroke_tess = StrokeTessellator::new();
    let stroke_opts = StrokeOptions::default().with_line_width(stroke_weight);

    let glyph_paths = extract_glyph_lyon_paths(layout, base_x, base_y, max_h);
    for path in &glyph_paths {
        let mut builder = MeshBuilder::new(mesh, color.clone());
        let _ = stroke_tess.tessellate_path(path, &stroke_opts, &mut builder);
    }
}

/// Translate a lyon path keeping Y-up convention (for 3D geometry).
/// Font outline Y is up; layout ty is Y-down, so we compute: (x + tx, y - ty).
fn translate_path_yup(path: &Path, tx: f32, ty: f32) -> Path {
    let mut builder = Path::builder();
    for event in path.iter() {
        use lyon::path::Event;
        match event {
            Event::Begin { at } => {
                builder.begin(Point::new(at.x + tx, at.y - ty));
            }
            Event::Line { from: _, to } => {
                builder.line_to(Point::new(to.x + tx, to.y - ty));
            }
            Event::Quadratic { from: _, ctrl, to } => {
                builder.quadratic_bezier_to(
                    Point::new(ctrl.x + tx, ctrl.y - ty),
                    Point::new(to.x + tx, to.y - ty),
                );
            }
            Event::Cubic {
                from: _,
                ctrl1,
                ctrl2,
                to,
            } => {
                builder.cubic_bezier_to(
                    Point::new(ctrl1.x + tx, ctrl1.y - ty),
                    Point::new(ctrl2.x + tx, ctrl2.y - ty),
                    Point::new(to.x + tx, to.y - ty),
                );
            }
            Event::End {
                last: _,
                first: _,
                close,
            } => {
                builder.end(close);
            }
        }
    }
    builder.build()
}

/// Extract glyph outlines as lyon Path objects in Y-up convention (for 3D).
fn extract_glyph_lyon_paths_yup(
    layout: &Layout<Color>,
    base_x: f32,
    base_y: f32,
    max_h: Option<f32>,
) -> Vec<Path> {
    let mut paths = Vec::new();

    for line_idx in 0..layout.len() {
        let Some(line) = layout.get(line_idx) else {
            continue;
        };

        if let Some(h) = max_h {
            let metrics = line.metrics();
            if metrics.baseline + metrics.descent > h {
                break;
            }
        }

        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };

            let run = glyph_run.run();
            let font_data = run.font();
            let font_size = run.font_size();
            let normalized_coords = run.normalized_coords();

            let Ok(font_ref) = FontRef::from_index(font_data.data.as_ref(), font_data.index)
            else {
                continue;
            };

            let outlines = font_ref.outline_glyphs();
            let skrifa_size = Size::new(font_size);
            let coords: Vec<NormalizedCoord> = normalized_coords
                .iter()
                .map(|&c| NormalizedCoord::from_bits(c))
                .collect();
            let location = LocationRef::new(&coords);

            for glyph in glyph_run.positioned_glyphs() {
                let glyph_id = skrifa::GlyphId::new(glyph.id);

                if let Some(outline_glyph) = outlines.get(glyph_id) {
                    let mut pen = LyonOutlinePen::new();
                    let settings = DrawSettings::unhinted(skrifa_size, location);
                    let _ = outline_glyph.draw(settings, &mut pen);

                    if let Some(path) = pen.build() {
                        let tx = base_x + glyph.x;
                        let ty = base_y + glyph.y;
                        paths.push(translate_path_yup(&path, tx, ty));
                    }
                }
            }
        }
    }

    paths
}

/// Translate a lyon path by (tx, ty) and flip Y coordinates (font Y-up to screen Y-down).
fn translate_path_flip_y(path: &Path, tx: f32, ty: f32) -> Path {
    let mut builder = Path::builder();
    for event in path.iter() {
        use lyon::path::Event;
        match event {
            Event::Begin { at } => {
                builder.begin(Point::new(at.x + tx, -at.y + ty));
            }
            Event::Line { from: _, to } => {
                builder.line_to(Point::new(to.x + tx, -to.y + ty));
            }
            Event::Quadratic { from: _, ctrl, to } => {
                builder.quadratic_bezier_to(
                    Point::new(ctrl.x + tx, -ctrl.y + ty),
                    Point::new(to.x + tx, -to.y + ty),
                );
            }
            Event::Cubic {
                from: _,
                ctrl1,
                ctrl2,
                to,
            } => {
                builder.cubic_bezier_to(
                    Point::new(ctrl1.x + tx, -ctrl1.y + ty),
                    Point::new(ctrl2.x + tx, -ctrl2.y + ty),
                    Point::new(to.x + tx, -to.y + ty),
                );
            }
            Event::End {
                last: _,
                first: _,
                close,
            } => {
                builder.end(close);
            }
        }
    }
    builder.build()
}

/// An `OutlinePen` that builds a lyon `Path` from a skrifa glyph outline.
struct LyonOutlinePen {
    builder: lyon::path::path::Builder,
    has_content: bool,
}

impl LyonOutlinePen {
    fn new() -> Self {
        Self {
            builder: Path::builder(),
            has_content: false,
        }
    }

    fn build(self) -> Option<Path> {
        if self.has_content {
            Some(self.builder.build())
        } else {
            None
        }
    }
}

impl OutlinePen for LyonOutlinePen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.builder.begin(Point::new(x, y));
        self.has_content = true;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(Point::new(x, y));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.builder
            .quadratic_bezier_to(Point::new(cx0, cy0), Point::new(x, y));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.builder.cubic_bezier_to(
            Point::new(cx0, cy0),
            Point::new(cx1, cy1),
            Point::new(x, y),
        );
    }

    fn close(&mut self) {
        self.builder.end(true);
    }
}
