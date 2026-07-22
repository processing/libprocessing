#![cfg(target_arch = "wasm32")]

use bevy::color::Color as BevyColor;
use bevy::color::LinearRgba;
use bevy::math::{Vec2, Vec3, Vec4};
use bevy::prelude::Entity;
use bevy::render::render_resource::{Extent3d, TextureFormat};
use processing::prelude::color::{ColorMode, ColorSpace};
use processing::prelude::error::ProcessingError;
use processing::prelude::*;
use wasm_bindgen::prelude::*;

fn check<T, E: std::fmt::Display>(result: Result<T, E>) -> Result<T, JsValue> {
    result.map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen(start)]
fn wasm_start() {
    console_error_panic_hook::set_once();
}

#[derive(Debug, Clone, Copy)]
struct Color {
    pub c1: f32,
    pub c2: f32,
    pub c3: f32,
    pub a: f32,
    pub space: u8,
}

impl Color {
    pub fn resolve(self, mode: &ColorMode) -> bevy::color::Color {
        let c1 = mode.scale(self.c1, 0);
        let c2 = mode.scale(self.c2, 1);
        let c3 = mode.scale(self.c3, 2);
        let ca = mode.scale(self.a, 3);
        mode.space.color(c1, c2, c3, ca)
    }

    pub fn from_linear(lin: LinearRgba) -> Self {
        Color {
            c1: lin.red,
            c2: lin.green,
            c3: lin.blue,
            a: lin.alpha,
            space: ColorSpace::Linear as u8,
        }
    }
}

#[wasm_bindgen(js_name = "init")]
pub async fn js_init() -> Result<(), JsValue> {
    check(init(Config::new()).await)
}

#[wasm_bindgen(js_name = "surfaceCreateFromCanvas")]
pub fn js_surface_create_from_canvas(
    canvas_id: &str,
    width: u32,
    height: u32,
) -> Result<u64, JsValue> {
    check(surface_create_from_canvas(canvas_id, width, height).map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "graphicsCreate")]
pub fn js_graphics_create(surface_id: u64, width: u32, height: u32) -> Result<u64, JsValue> {
    let surface_entity = Entity::from_bits(surface_id);
    check(graphics_create(surface_entity, width, height, TextureFormat::Rgba16Float))
        .map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "graphicsDestroy")]
pub fn js_graphics_destroy(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_destroy(graphics_entity))
}

#[wasm_bindgen(js_name = "surfaceDestroy")]
pub fn js_surface_destroy(window_id: u64) -> Result<(), JsValue> {
    let window_entity = Entity::from_bits(window_id);
    check(surface_destroy(window_entity))
}

#[wasm_bindgen(js_name = "surfaceResize")]
pub fn js_surface_resize(window_id: u64, width: u32, height: u32) -> Result<(), JsValue> {
    let window_entity = Entity::from_bits(window_id);
    check(surface_resize(window_entity, width, height))
}

#[wasm_bindgen(js_name = "backgroundColor")]
pub fn js_background_color(
    graphics_id: u64,
    c1: f32,
    c2: f32,
    c3: f32,
    a: f32,
    space: u8,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let color = Color { c1, c2, c3, a, space };
    check((|| {
        let mode = graphics_get_color_mode(graphics_entity)?;
        let color = color.resolve(&mode);
        graphics_record_command(graphics_entity, DrawCommand::BackgroundColor(color))
    })())
}

#[wasm_bindgen(js_name = "backgroundImage")]
pub fn js_background_image(graphics_id: u64, image_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let image_entity = Entity::from_bits(image_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::BackgroundImage(image_entity),
    ))
}

#[wasm_bindgen(js_name = "beginDraw")]
pub fn js_begin_draw(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_begin_draw(graphics_entity))
}

#[wasm_bindgen(js_name = "flush")]
pub fn js_flush(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_flush(graphics_entity))
}

#[wasm_bindgen(js_name = "endDraw")]
pub fn js_end_draw(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_end_draw(graphics_entity))
}

#[wasm_bindgen(js_name = "exit")]
pub fn js_exit(exit_code: u8) -> Result<(), JsValue> {
    check(exit(exit_code))
}

#[wasm_bindgen(js_name = "colorMode")]
pub fn js_color_mode(
    graphics_id: u64,
    space: u8,
    max1: f32,
    max2: f32,
    max3: f32,
    max_alpha: f32,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check((|| {
        let space = processing::prelude::color::ColorSpace::from_u8(space).ok_or_else(|| {
            processing::prelude::error::ProcessingError::InvalidArgument(format!(
                "unknown color space: {space}"
            ))
        })?;
        let mode = processing::prelude::color::ColorMode::new(space, max1, max2, max3, max_alpha);
        graphics_set_color_mode(graphics_entity, mode)
    })())
}

#[wasm_bindgen(js_name = "setFill")]
pub fn js_set_fill(
    graphics_id: u64,
    c1: f32,
    c2: f32,
    c3: f32,
    a: f32,
    space: u8,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let color = Color { c1, c2, c3, a, space };
    check((|| {
        let mode = graphics_get_color_mode(graphics_entity)?;
        graphics_record_command(graphics_entity, DrawCommand::Fill(color.resolve(&mode)))
    })())
}

#[wasm_bindgen(js_name = "setStrokeColor")]
pub fn js_set_stroke_color(
    graphics_id: u64,
    c1: f32,
    c2: f32,
    c3: f32,
    a: f32,
    space: u8,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let color = Color { c1, c2, c3, a, space };
    check((|| {
        let mode = graphics_get_color_mode(graphics_entity)?;
        graphics_record_command(
            graphics_entity,
            DrawCommand::StrokeColor(color.resolve(&mode)),
        )
    })())
}

#[wasm_bindgen(js_name = "setStrokeWeight")]
pub fn js_set_stroke_weight(graphics_id: u64, weight: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::StrokeWeight(weight)))
}

#[wasm_bindgen(js_name = "setStrokeCap")]
pub fn js_set_stroke_cap(graphics_id: u64, cap: u8) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::StrokeCap(processing::prelude::StrokeCapMode::from(cap)),
    ))
}

#[wasm_bindgen(js_name = "setStrokeJoin")]
pub fn js_set_stroke_join(graphics_id: u64, join: u8) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::StrokeJoin(processing::prelude::StrokeJoinMode::from(join)),
    ))
}

#[wasm_bindgen(js_name = "rectMode")]
pub fn js_rect_mode(graphics_id: u64, mode: u8) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::RectMode(processing::prelude::ShapeMode::from(mode)),
    ))
}

#[wasm_bindgen(js_name = "ellipseMode")]
pub fn js_ellipse_mode(graphics_id: u64, mode: u8) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::EllipseMode(processing::prelude::ShapeMode::from(mode)),
    ))
}

#[wasm_bindgen(js_name = "noFill")]
pub fn js_no_fill(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::NoFill))
}

#[wasm_bindgen(js_name = "noStroke")]
pub fn js_no_stroke(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::NoStroke))
}

#[wasm_bindgen(js_name = "pushMatrix")]
pub fn js_push_matrix(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::PushMatrix))
}

#[wasm_bindgen(js_name = "popMatrix")]
pub fn js_pop_matrix(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::PopMatrix))
}

#[wasm_bindgen(js_name = "resetMatrix")]
pub fn js_reset_matrix(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::ResetMatrix))
}

#[wasm_bindgen(js_name = "translate")]
pub fn js_translate(graphics_id: u64, x: f32, y: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Translate(Vec2::new(x, y)),
    ))
}

#[wasm_bindgen(js_name = "rotate")]
pub fn js_rotate(graphics_id: u64, angle: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::Rotate { angle }))
}

#[wasm_bindgen(js_name = "scale")]
pub fn js_scale(graphics_id: u64, x: f32, y: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::Scale(Vec2::new(x, y))))
}

#[wasm_bindgen(js_name = "shearX")]
pub fn js_shear_x(graphics_id: u64, angle: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::ShearX { angle }))
}

#[wasm_bindgen(js_name = "shearY")]
pub fn js_shear_y(graphics_id: u64, angle: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::ShearY { angle }))
}

#[wasm_bindgen(js_name = "setBlendMode")]
pub fn js_set_blend_mode(graphics_id: u64, mode: u8) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check((|| {
        let blend_state = processing::prelude::BlendMode::try_from(mode)?.to_blend_state();
        graphics_record_command(graphics_entity, DrawCommand::BlendMode(blend_state))
    })())
}

#[wasm_bindgen(js_name = "setCustomBlendMode")]
pub fn js_set_custom_blend_mode(
    graphics_id: u64,
    color_src: u8,
    color_dst: u8,
    color_op: u8,
    alpha_src: u8,
    alpha_dst: u8,
    alpha_op: u8,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check((|| {
        let blend_state = custom_blend_state(
            color_src, color_dst, color_op, alpha_src, alpha_dst, alpha_op,
        )?;
        graphics_record_command(graphics_entity, DrawCommand::BlendMode(Some(blend_state)))
    })())
}

#[wasm_bindgen(js_name = "rect")]
pub fn js_rect(
    graphics_id: u64,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    tl: f32,
    tr: f32,
    br: f32,
    bl: f32,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Rect {
            x,
            y,
            w,
            h,
            radii: [tl, tr, br, bl],
        },
    ))
}

#[wasm_bindgen(js_name = "ellipse")]
pub fn js_ellipse(graphics_id: u64, cx: f32, cy: f32, w: f32, h: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::Ellipse { cx, cy, w, h }))
}

#[wasm_bindgen(js_name = "circle")]
pub fn js_circle(graphics_id: u64, cx: f32, cy: f32, d: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Ellipse { cx, cy, w: d, h: d },
    ))
}

#[wasm_bindgen(js_name = "line")]
pub fn js_line(graphics_id: u64, x1: f32, y1: f32, x2: f32, y2: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::Line { x1, y1, x2, y2 }))
}

#[wasm_bindgen(js_name = "triangle")]
pub fn js_triangle(
    graphics_id: u64,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Triangle {
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
        },
    ))
}

#[wasm_bindgen(js_name = "quad")]
pub fn js_quad(
    graphics_id: u64,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    x4: f32,
    y4: f32,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
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
    ))
}

#[wasm_bindgen(js_name = "point")]
pub fn js_point(graphics_id: u64, x: f32, y: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::Point { x, y }))
}

#[wasm_bindgen(js_name = "square")]
pub fn js_square(graphics_id: u64, x: f32, y: f32, s: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Rect {
            x,
            y,
            w: s,
            h: s,
            radii: [0.0; 4],
        },
    ))
}

#[wasm_bindgen(js_name = "arc")]
pub fn js_arc(
    graphics_id: u64,
    cx: f32,
    cy: f32,
    w: f32,
    h: f32,
    start: f32,
    stop: f32,
    mode: u8,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Arc {
            cx,
            cy,
            w,
            h,
            start,
            stop,
            mode: processing::prelude::ArcMode::from(mode),
        },
    ))
}

#[wasm_bindgen(js_name = "bezier")]
pub fn js_bezier(
    graphics_id: u64,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    x4: f32,
    y4: f32,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
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
    ))
}

#[wasm_bindgen(js_name = "curve")]
pub fn js_curve(
    graphics_id: u64,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    x4: f32,
    y4: f32,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
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
    ))
}

#[wasm_bindgen(js_name = "cylinder")]
pub fn js_cylinder(graphics_id: u64, radius: f32, height: f32, detail: u32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Cylinder {
            radius,
            height,
            detail,
        },
    ))
}

#[wasm_bindgen(js_name = "cone")]
pub fn js_cone(graphics_id: u64, radius: f32, height: f32, detail: u32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Cone {
            radius,
            height,
            detail,
        },
    ))
}

#[wasm_bindgen(js_name = "torus")]
pub fn js_torus(
    graphics_id: u64,
    radius: f32,
    tube_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Torus {
            radius,
            tube_radius,
            major_segments,
            minor_segments,
        },
    ))
}

#[wasm_bindgen(js_name = "plane")]
pub fn js_plane(graphics_id: u64, width: f32, height: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::Plane { width, height }))
}

#[wasm_bindgen(js_name = "capsule")]
pub fn js_capsule(graphics_id: u64, radius: f32, length: f32, detail: u32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Capsule {
            radius,
            length,
            detail,
        },
    ))
}

#[wasm_bindgen(js_name = "conicalFrustum")]
pub fn js_conical_frustum(
    graphics_id: u64,
    radius_top: f32,
    radius_bottom: f32,
    height: f32,
    detail: u32,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::ConicalFrustum {
            radius_top,
            radius_bottom,
            height,
            detail,
        },
    ))
}

#[wasm_bindgen(js_name = "tetrahedron")]
pub fn js_tetrahedron(graphics_id: u64, radius: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::Tetrahedron { radius }))
}

#[wasm_bindgen(js_name = "beginShape")]
pub fn js_begin_shape(graphics_id: u64, kind: u8) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::BeginShape {
            kind: processing::prelude::ShapeKind::from(kind),
        },
    ))
}

#[wasm_bindgen(js_name = "endShape")]
pub fn js_end_shape(graphics_id: u64, close: bool) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::EndShape { close }))
}

#[wasm_bindgen(js_name = "vertex")]
pub fn js_vertex(graphics_id: u64, x: f32, y: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::ShapeVertex { x, y }))
}

#[wasm_bindgen(js_name = "bezierVertex")]
pub fn js_bezier_vertex(
    graphics_id: u64,
    cx1: f32,
    cy1: f32,
    cx2: f32,
    cy2: f32,
    x: f32,
    y: f32,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::ShapeBezierVertex {
            cx1,
            cy1,
            cx2,
            cy2,
            x,
            y,
        },
    ))
}

#[wasm_bindgen(js_name = "quadraticVertex")]
pub fn js_quadratic_vertex(
    graphics_id: u64,
    cx: f32,
    cy: f32,
    x: f32,
    y: f32,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::ShapeQuadraticVertex { cx, cy, x, y },
    ))
}

#[wasm_bindgen(js_name = "curveVertex")]
pub fn js_curve_vertex(graphics_id: u64, x: f32, y: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::ShapeCurveVertex { x, y }))
}

#[wasm_bindgen(js_name = "beginContour")]
pub fn js_begin_contour(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::BeginContour))
}

#[wasm_bindgen(js_name = "endContour")]
pub fn js_end_contour(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::EndContour))
}

#[wasm_bindgen(js_name = "loadFont")]
pub fn js_load_font(path: &str) -> Result<u64, JsValue> {
    check(font_load(path).map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "createFont")]
pub fn js_create_font(name: &str) -> Result<u64, JsValue> {
    check(font_create(name).map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "fontVariationCount")]
pub fn js_font_variation_count(font_id: u64) -> Result<u32, JsValue> {
    let font_entity = Entity::from_bits(font_id);
    check(font_variations(font_entity).map(|v| v.len() as u32))
}

#[wasm_bindgen(js_name = "fontVariation")]
pub fn js_font_variation(font_id: u64, index: u32) -> Result<Vec<f32>, JsValue> {
    let font_entity = Entity::from_bits(font_id);
    let axes = check(font_variations(font_entity))?;
    match axes.get(index as usize) {
        Some(axis) => {
            let tag_bytes = axis.tag.as_bytes();
            let mut out = Vec::with_capacity(7);
            for i in 0..4 {
                out.push(*tag_bytes.get(i).unwrap_or(&b' ') as f32);
            }
            out.push(axis.min);
            out.push(axis.max);
            out.push(axis.default);
            Ok(out)
        }
        None => Ok(Vec::new()),
    }
}

#[wasm_bindgen(js_name = "textFont")]
pub fn js_text_font(graphics_id: u64, font_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let font_entity = if font_id == 0 {
        None
    } else {
        Some(Entity::from_bits(font_id))
    };
    check(graphics_text_font(graphics_entity, font_entity))
}

#[wasm_bindgen(js_name = "text")]
pub fn js_text(graphics_id: u64, content: &str, x: f32, y: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let content = content.to_string();
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Text {
            content,
            x,
            y,
            z: 0.0,
            max_w: None,
            max_h: None,
        },
    ))
}

#[wasm_bindgen(js_name = "text3D")]
pub fn js_text_3d(graphics_id: u64, content: &str, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let content = content.to_string();
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Text {
            content,
            x,
            y,
            z,
            max_w: None,
            max_h: None,
        },
    ))
}

#[wasm_bindgen(js_name = "textInt")]
pub fn js_text_int(graphics_id: u64, value: i32, x: f32, y: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let content = value.to_string();
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Text {
            content,
            x,
            y,
            z: 0.0,
            max_w: None,
            max_h: None,
        },
    ))
}

#[wasm_bindgen(js_name = "textFloat")]
pub fn js_text_float(graphics_id: u64, value: f32, x: f32, y: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let content = format!("{:.3}", value);
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Text {
            content,
            x,
            y,
            z: 0.0,
            max_w: None,
            max_h: None,
        },
    ))
}

#[wasm_bindgen(js_name = "textBox")]
pub fn js_text_box(
    graphics_id: u64,
    content: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let content = content.to_string();
    check(graphics_record_command(
        graphics_entity,
        DrawCommand::Text {
            content,
            x,
            y,
            z: 0.0,
            max_w: Some(w),
            max_h: Some(h),
        },
    ))
}

#[wasm_bindgen(js_name = "textStyle")]
pub fn js_text_style(graphics_id: u64, style: u8) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_text_style(graphics_entity, style))
}

#[wasm_bindgen(js_name = "textBounds")]
pub fn js_text_bounds(
    graphics_id: u64,
    content: &str,
    x: f32,
    y: f32,
) -> Result<Vec<f32>, JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let bounds = check(graphics_text_bounds(graphics_entity, content, x, y, None, None))?;
    Ok(bounds.to_vec())
}

#[wasm_bindgen(js_name = "textVariation")]
pub fn js_text_variation(graphics_id: u64, tag: &str, value: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_text_variation(graphics_entity, tag, value))
}

#[wasm_bindgen(js_name = "clearTextVariations")]
pub fn js_clear_text_variations(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_clear_text_variations(graphics_entity))
}

#[wasm_bindgen(js_name = "textFeature")]
pub fn js_text_feature(graphics_id: u64, tag: &str, value: u16) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_text_feature(graphics_entity, tag, value))
}

#[wasm_bindgen(js_name = "noTextFeature")]
pub fn js_no_text_feature(graphics_id: u64, tag: &str) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_no_text_feature(graphics_entity, tag))
}

#[wasm_bindgen(js_name = "clearTextFeatures")]
pub fn js_clear_text_features(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_clear_text_features(graphics_entity))
}

#[wasm_bindgen(js_name = "textGlyphColors")]
pub fn js_text_glyph_colors(graphics_id: u64, colors: &[f32]) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let count = colors.len() / 4;
    let colors: Vec<BevyColor> = (0..count)
        .map(|i| {
            let base = i * 4;
            BevyColor::srgba(colors[base], colors[base + 1], colors[base + 2], colors[base + 3])
        })
        .collect();
    check(graphics_text_glyph_colors(graphics_entity, colors))
}

#[wasm_bindgen(js_name = "textWeight")]
pub fn js_text_weight(graphics_id: u64, weight: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_text_weight(graphics_entity, weight))
}

#[wasm_bindgen(js_name = "textSize")]
pub fn js_text_size(graphics_id: u64, size: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::TextSize(size)))
}

#[wasm_bindgen(js_name = "textAlign")]
pub fn js_text_align(graphics_id: u64, h: u8, v: u8) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_text_align(graphics_entity, h, v))
}

#[wasm_bindgen(js_name = "textLeading")]
pub fn js_text_leading(graphics_id: u64, leading: f32) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_record_command(graphics_entity, DrawCommand::TextLeading(leading)))
}

#[wasm_bindgen(js_name = "textWrap")]
pub fn js_text_wrap(graphics_id: u64, mode: u8) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_text_wrap(graphics_entity, mode))
}

#[wasm_bindgen(js_name = "textWidth")]
pub fn js_text_width(graphics_id: u64, content: &str) -> Result<f32, JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_text_width(graphics_entity, content))
}

#[wasm_bindgen(js_name = "textAscent")]
pub fn js_text_ascent(graphics_id: u64) -> Result<f32, JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_text_ascent(graphics_entity))
}

#[wasm_bindgen(js_name = "textDescent")]
pub fn js_text_descent(graphics_id: u64) -> Result<f32, JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_text_descent(graphics_entity))
}

#[wasm_bindgen(js_name = "imageCreate")]
pub fn js_image_create(width: u32, height: u32, data: &[u8]) -> Result<u64, JsValue> {
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    check(image_create(size, data.to_vec(), TextureFormat::Rgba8UnormSrgb).map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "imageLoad")]
pub async fn js_image_load(path: &str) -> Result<u64, JsValue> {
    check(image_load(path).await.map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "imageResize")]
pub fn js_image_resize(image_id: u64, new_width: u32, new_height: u32) -> Result<(), JsValue> {
    let image_entity = Entity::from_bits(image_id);
    let new_size = Extent3d {
        width: new_width,
        height: new_height,
        depth_or_array_layers: 1,
    };
    check(image_resize(image_entity, new_size))
}

#[wasm_bindgen(js_name = "imageReadback")]
pub fn js_image_readback(image_id: u64) -> Result<Vec<f32>, JsValue> {
    let image_entity = Entity::from_bits(image_id);
    let colors = check(image_readback(image_entity))?;
    let mut out = Vec::with_capacity(colors.len() * 4);
    for color in &colors {
        let c = Color::from_linear(*color);
        out.push(c.c1);
        out.push(c.c2);
        out.push(c.c3);
        out.push(c.a);
    }
    Ok(out)
}

#[wasm_bindgen(js_name = "mode3D")]
pub fn js_mode_3d(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_mode_3d(graphics_entity))
}

#[wasm_bindgen(js_name = "mode2D")]
pub fn js_mode_2d(graphics_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_mode_2d(graphics_entity))
}

#[wasm_bindgen(js_name = "perspective")]
pub fn js_perspective(
    graphics_id: u64,
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_perspective(
        graphics_entity,
        fov,
        aspect,
        near,
        far,
        bevy::math::Vec4::new(0.0, 0.0, -1.0, -near),
    ))
}

#[wasm_bindgen(js_name = "ortho")]
pub fn js_ortho(
    graphics_id: u64,
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    check(graphics_ortho(graphics_entity, left, right, bottom, top, near, far))
}

#[wasm_bindgen(js_name = "transformSetPosition")]
pub fn js_transform_set_position(entity_id: u64, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(entity_id);
    check(transform_set_position(entity, Vec3::new(x, y, z)))
}

#[wasm_bindgen(js_name = "transformTranslate")]
pub fn js_transform_translate(entity_id: u64, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(entity_id);
    check(transform_translate(entity, Vec3::new(x, y, z)))
}

#[wasm_bindgen(js_name = "transformSetRotation")]
pub fn js_transform_set_rotation(entity_id: u64, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(entity_id);
    check(transform_set_rotation(entity, Vec3::new(x, y, z)))
}

#[wasm_bindgen(js_name = "transformRotateX")]
pub fn js_transform_rotate_x(entity_id: u64, angle: f32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(entity_id);
    check(transform_rotate_x(entity, angle))
}

#[wasm_bindgen(js_name = "transformRotateY")]
pub fn js_transform_rotate_y(entity_id: u64, angle: f32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(entity_id);
    check(transform_rotate_y(entity, angle))
}

#[wasm_bindgen(js_name = "transformRotateZ")]
pub fn js_transform_rotate_z(entity_id: u64, angle: f32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(entity_id);
    check(transform_rotate_z(entity, angle))
}

#[wasm_bindgen(js_name = "transformRotateAxis")]
pub fn js_transform_rotate_axis(
    entity_id: u64,
    angle: f32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
) -> Result<(), JsValue> {
    let entity = Entity::from_bits(entity_id);
    check(transform_rotate_axis(entity, angle, Vec3::new(axis_x, axis_y, axis_z)))
}

#[wasm_bindgen(js_name = "transformSetScale")]
pub fn js_transform_set_scale(entity_id: u64, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(entity_id);
    check(transform_set_scale(entity, Vec3::new(x, y, z)))
}

#[wasm_bindgen(js_name = "transformScale")]
pub fn js_transform_scale(entity_id: u64, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(entity_id);
    check(transform_scale(entity, Vec3::new(x, y, z)))
}

#[wasm_bindgen(js_name = "transformLookAt")]
pub fn js_transform_look_at(
    entity_id: u64,
    target_x: f32,
    target_y: f32,
    target_z: f32,
) -> Result<(), JsValue> {
    let entity = Entity::from_bits(entity_id);
    check(transform_look_at(entity, Vec3::new(target_x, target_y, target_z)))
}

#[wasm_bindgen(js_name = "transformReset")]
pub fn js_transform_reset(entity_id: u64) -> Result<(), JsValue> {
    let entity = Entity::from_bits(entity_id);
    check(transform_reset(entity))
}

pub const PROCESSING_ATTR_FORMAT_FLOAT: u8 = 1;
pub const PROCESSING_ATTR_FORMAT_FLOAT2: u8 = 2;
pub const PROCESSING_ATTR_FORMAT_FLOAT3: u8 = 3;
pub const PROCESSING_ATTR_FORMAT_FLOAT4: u8 = 4;

pub const PROCESSING_TOPOLOGY_POINT_LIST: u8 = 0;
pub const PROCESSING_TOPOLOGY_LINE_LIST: u8 = 1;
pub const PROCESSING_TOPOLOGY_LINE_STRIP: u8 = 2;
pub const PROCESSING_TOPOLOGY_TRIANGLE_LIST: u8 = 3;
pub const PROCESSING_TOPOLOGY_TRIANGLE_STRIP: u8 = 4;

pub const PROCESSING_STROKE_CAP_ROUND: u8 = 0;
pub const PROCESSING_STROKE_CAP_SQUARE: u8 = 1;
pub const PROCESSING_STROKE_CAP_PROJECT: u8 = 2;

pub const PROCESSING_STROKE_JOIN_ROUND: u8 = 0;
pub const PROCESSING_STROKE_JOIN_MITER: u8 = 1;
pub const PROCESSING_STROKE_JOIN_BEVEL: u8 = 2;

pub const PROCESSING_BLEND_MODE_BLEND: u8 = 0;
pub const PROCESSING_BLEND_MODE_ADD: u8 = 1;
pub const PROCESSING_BLEND_MODE_SUBTRACT: u8 = 2;
pub const PROCESSING_BLEND_MODE_DARKEST: u8 = 3;
pub const PROCESSING_BLEND_MODE_LIGHTEST: u8 = 4;
pub const PROCESSING_BLEND_MODE_DIFFERENCE: u8 = 5;
pub const PROCESSING_BLEND_MODE_EXCLUSION: u8 = 6;
pub const PROCESSING_BLEND_MODE_MULTIPLY: u8 = 7;
pub const PROCESSING_BLEND_MODE_SCREEN: u8 = 8;
pub const PROCESSING_BLEND_MODE_REPLACE: u8 = 9;

pub const PROCESSING_BLEND_FACTOR_ZERO: u8 = 0;
pub const PROCESSING_BLEND_FACTOR_ONE: u8 = 1;
pub const PROCESSING_BLEND_FACTOR_SRC: u8 = 2;
pub const PROCESSING_BLEND_FACTOR_ONE_MINUS_SRC: u8 = 3;
pub const PROCESSING_BLEND_FACTOR_SRC_ALPHA: u8 = 4;
pub const PROCESSING_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA: u8 = 5;
pub const PROCESSING_BLEND_FACTOR_DST: u8 = 6;
pub const PROCESSING_BLEND_FACTOR_ONE_MINUS_DST: u8 = 7;
pub const PROCESSING_BLEND_FACTOR_DST_ALPHA: u8 = 8;
pub const PROCESSING_BLEND_FACTOR_ONE_MINUS_DST_ALPHA: u8 = 9;
pub const PROCESSING_BLEND_FACTOR_SRC_ALPHA_SATURATED: u8 = 10;

pub const PROCESSING_BLEND_OP_ADD: u8 = 0;
pub const PROCESSING_BLEND_OP_SUBTRACT: u8 = 1;
pub const PROCESSING_BLEND_OP_REVERSE_SUBTRACT: u8 = 2;
pub const PROCESSING_BLEND_OP_MIN: u8 = 3;
pub const PROCESSING_BLEND_OP_MAX: u8 = 4;

#[wasm_bindgen(js_name = "geometryLayoutCreate")]
pub fn js_geometry_layout_create() -> Result<u64, JsValue> {
    check(geometry_layout_create()).map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "geometryLayoutAddPosition")]
pub fn js_geometry_layout_add_position(layout_id: u64) -> Result<(), JsValue> {
    let entity = Entity::from_bits(layout_id);
    check(geometry_layout_add_position(entity))
}

#[wasm_bindgen(js_name = "geometryLayoutAddNormal")]
pub fn js_geometry_layout_add_normal(layout_id: u64) -> Result<(), JsValue> {
    let entity = Entity::from_bits(layout_id);
    check(geometry_layout_add_normal(entity))
}

#[wasm_bindgen(js_name = "geometryLayoutAddColor")]
pub fn js_geometry_layout_add_color(layout_id: u64) -> Result<(), JsValue> {
    let entity = Entity::from_bits(layout_id);
    check(geometry_layout_add_color(entity))
}

#[wasm_bindgen(js_name = "geometryLayoutAddUv")]
pub fn js_geometry_layout_add_uv(layout_id: u64) -> Result<(), JsValue> {
    let entity = Entity::from_bits(layout_id);
    check(geometry_layout_add_uv(entity))
}

#[wasm_bindgen(js_name = "geometryLayoutAddAttribute")]
pub fn js_geometry_layout_add_attribute(layout_id: u64, attr_id: u64) -> Result<(), JsValue> {
    let layout_entity = Entity::from_bits(layout_id);
    let attr_entity = Entity::from_bits(attr_id);
    check(geometry_layout_add_attribute(layout_entity, attr_entity))
}

#[wasm_bindgen(js_name = "geometryLayoutDestroy")]
pub fn js_geometry_layout_destroy(layout_id: u64) -> Result<(), JsValue> {
    let entity = Entity::from_bits(layout_id);
    check(geometry_layout_destroy(entity))
}

#[wasm_bindgen(js_name = "geometryCreateWithLayout")]
pub fn js_geometry_create_with_layout(layout_id: u64, topology: u8) -> Result<u64, JsValue> {
    let Some(topo) = geometry::Topology::from_u8(topology) else {
        return Err(JsValue::from_str("Invalid topology"));
    };
    let entity = Entity::from_bits(layout_id);
    check(geometry_create_with_layout(entity, topo)).map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "geometryCreate")]
pub fn js_geometry_create(topology: u8) -> Result<u64, JsValue> {
    let Some(topo) = geometry::Topology::from_u8(topology) else {
        return Err(JsValue::from_str("Invalid topology"));
    };
    check(geometry_create(topo)).map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "geometryNormal")]
pub fn js_geometry_normal(geo_id: u64, nx: f32, ny: f32, nz: f32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(geo_id);
    check(geometry_normal(entity, Vec3::new(nx, ny, nz)))
}

#[wasm_bindgen(js_name = "geometryColor")]
pub fn js_geometry_color(geo_id: u64, r: f32, g: f32, b: f32, a: f32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(geo_id);
    check(geometry_color(entity, Vec4::new(r, g, b, a)))
}

#[wasm_bindgen(js_name = "geometryUv")]
pub fn js_geometry_uv(geo_id: u64, u: f32, v: f32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(geo_id);
    check(geometry_uv(entity, u, v))
}

#[wasm_bindgen(js_name = "geometryAttributeCreate")]
pub fn js_geometry_attribute_create(name: &str, format: u8) -> Result<u64, JsValue> {
    let attr_format = match geometry::AttributeFormat::from_u8(format) {
        Some(f) => f,
        None => return Err(JsValue::from_str("Invalid attribute format")),
    };
    check(geometry_attribute_create(name, attr_format)).map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "geometryAttributeDestroy")]
pub fn js_geometry_attribute_destroy(attr_id: u64) -> Result<(), JsValue> {
    let entity = Entity::from_bits(attr_id);
    check(geometry_attribute_destroy(entity))
}

#[wasm_bindgen(js_name = "geometryAttributePosition")]
pub fn js_geometry_attribute_position() -> u64 {
    geometry_attribute_position().to_bits()
}

#[wasm_bindgen(js_name = "geometryAttributeNormal")]
pub fn js_geometry_attribute_normal() -> u64 {
    geometry_attribute_normal().to_bits()
}

#[wasm_bindgen(js_name = "geometryAttributeColor")]
pub fn js_geometry_attribute_color() -> u64 {
    geometry_attribute_color().to_bits()
}

#[wasm_bindgen(js_name = "geometryAttributeUv")]
pub fn js_geometry_attribute_uv() -> u64 {
    geometry_attribute_uv().to_bits()
}

#[wasm_bindgen(js_name = "geometryAttributeFloat")]
pub fn js_geometry_attribute_float(geo_id: u64, attr_id: u64, v: f32) -> Result<(), JsValue> {
    let geo_entity = Entity::from_bits(geo_id);
    let attr_entity = Entity::from_bits(attr_id);
    check(geometry_attribute_float(geo_entity, attr_entity, v))
}

#[wasm_bindgen(js_name = "geometryAttributeFloat2")]
pub fn js_geometry_attribute_float2(
    geo_id: u64,
    attr_id: u64,
    x: f32,
    y: f32,
) -> Result<(), JsValue> {
    let geo_entity = Entity::from_bits(geo_id);
    let attr_entity = Entity::from_bits(attr_id);
    check(geometry_attribute_float2(geo_entity, attr_entity, x, y))
}

#[wasm_bindgen(js_name = "geometryAttributeFloat3")]
pub fn js_geometry_attribute_float3(
    geo_id: u64,
    attr_id: u64,
    x: f32,
    y: f32,
    z: f32,
) -> Result<(), JsValue> {
    let geo_entity = Entity::from_bits(geo_id);
    let attr_entity = Entity::from_bits(attr_id);
    check(geometry_attribute_float3(geo_entity, attr_entity, x, y, z))
}

#[wasm_bindgen(js_name = "geometryAttributeFloat4")]
pub fn js_geometry_attribute_float4(
    geo_id: u64,
    attr_id: u64,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
) -> Result<(), JsValue> {
    let geo_entity = Entity::from_bits(geo_id);
    let attr_entity = Entity::from_bits(attr_id);
    check(geometry_attribute_float4(geo_entity, attr_entity, x, y, z, w))
}

#[wasm_bindgen(js_name = "geometryVertex")]
pub fn js_geometry_vertex(geo_id: u64, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(geo_id);
    check(geometry_vertex(entity, Vec3::new(x, y, z)))
}

#[wasm_bindgen(js_name = "geometryIndex")]
pub fn js_geometry_index(geo_id: u64, i: u32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(geo_id);
    check(geometry_index(entity, i))
}

#[wasm_bindgen(js_name = "geometryVertexCount")]
pub fn js_geometry_vertex_count(geo_id: u64) -> Result<u32, JsValue> {
    let entity = Entity::from_bits(geo_id);
    check(geometry_vertex_count(entity))
}

#[wasm_bindgen(js_name = "geometryIndexCount")]
pub fn js_geometry_index_count(geo_id: u64) -> Result<u32, JsValue> {
    let entity = Entity::from_bits(geo_id);
    check(geometry_index_count(entity))
}

#[wasm_bindgen(js_name = "geometryGetPositions")]
pub fn js_geometry_get_positions(geo_id: u64, start: u32, end: u32) -> Result<Vec<f32>, JsValue> {
    let entity = Entity::from_bits(geo_id);
    let positions = check(geometry_get_positions(entity, start as usize, end as usize))?;
    Ok(positions.into_iter().flatten().collect())
}

#[wasm_bindgen(js_name = "geometryGetNormals")]
pub fn js_geometry_get_normals(geo_id: u64, start: u32, end: u32) -> Result<Vec<f32>, JsValue> {
    let entity = Entity::from_bits(geo_id);
    let normals = check(geometry_get_normals(entity, start as usize, end as usize))?;
    Ok(normals.into_iter().flatten().collect())
}

#[wasm_bindgen(js_name = "geometryGetColors")]
pub fn js_geometry_get_colors(geo_id: u64, start: u32, end: u32) -> Result<Vec<f32>, JsValue> {
    let entity = Entity::from_bits(geo_id);
    let colors = check(geometry_get_colors(entity, start as usize, end as usize))?;
    Ok(colors.into_iter().flatten().collect())
}

#[wasm_bindgen(js_name = "geometryGetUvs")]
pub fn js_geometry_get_uvs(geo_id: u64, start: u32, end: u32) -> Result<Vec<f32>, JsValue> {
    let entity = Entity::from_bits(geo_id);
    let uvs = check(geometry_get_uvs(entity, start as usize, end as usize))?;
    Ok(uvs.into_iter().flatten().collect())
}

#[wasm_bindgen(js_name = "geometryGetIndices")]
pub fn js_geometry_get_indices(geo_id: u64, start: u32, end: u32) -> Result<Vec<u32>, JsValue> {
    let entity = Entity::from_bits(geo_id);
    check(geometry_get_indices(entity, start as usize, end as usize))
}

#[wasm_bindgen(js_name = "geometrySetVertex")]
pub fn js_geometry_set_vertex(
    geo_id: u64,
    index: u32,
    x: f32,
    y: f32,
    z: f32,
) -> Result<(), JsValue> {
    let entity = Entity::from_bits(geo_id);
    check(geometry_set_vertex(entity, index, Vec3::new(x, y, z)))
}

#[wasm_bindgen(js_name = "geometrySetNormal")]
pub fn js_geometry_set_normal(
    geo_id: u64,
    index: u32,
    nx: f32,
    ny: f32,
    nz: f32,
) -> Result<(), JsValue> {
    let entity = Entity::from_bits(geo_id);
    check(geometry_set_normal(entity, index, Vec3::new(nx, ny, nz)))
}

#[wasm_bindgen(js_name = "geometrySetColor")]
pub fn js_geometry_set_color(
    geo_id: u64,
    index: u32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> Result<(), JsValue> {
    let entity = Entity::from_bits(geo_id);
    check(geometry_set_color(entity, index, Vec4::new(r, g, b, a)))
}

#[wasm_bindgen(js_name = "geometrySetUv")]
pub fn js_geometry_set_uv(geo_id: u64, index: u32, u: f32, v: f32) -> Result<(), JsValue> {
    let entity = Entity::from_bits(geo_id);
    check(geometry_set_uv(entity, index, Vec2::new(u, v)))
}

#[wasm_bindgen(js_name = "geometryDestroy")]
pub fn js_geometry_destroy(geo_id: u64) -> Result<(), JsValue> {
    let entity = Entity::from_bits(geo_id);
    check(geometry_destroy(entity))
}

#[wasm_bindgen(js_name = "model")]
pub fn js_model(graphics_id: u64, geo_id: u64) -> Result<(), JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let geo_entity = Entity::from_bits(geo_id);
    check(graphics_record_command(graphics_entity, DrawCommand::Geometry(geo_entity)))
}

#[wasm_bindgen(js_name = "geometryBox")]
pub fn js_geometry_box(width: f32, height: f32, depth: f32) -> Result<u64, JsValue> {
    check(geometry_box(width, height, depth)).map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "geometrySphere")]
pub fn js_geometry_sphere(radius: f32, sectors: u32, stacks: u32) -> Result<u64, JsValue> {
    check(geometry_sphere(radius, sectors, stacks)).map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "lightCreateDirectional")]
pub fn js_light_create_directional(
    graphics_id: u64,
    c1: f32,
    c2: f32,
    c3: f32,
    a: f32,
    space: u8,
    illuminance: f32,
) -> Result<u64, JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let color = Color { c1, c2, c3, a, space };
    check((|| {
        let mode = graphics_get_color_mode(graphics_entity)?;
        light_create_directional(graphics_entity, color.resolve(&mode), illuminance)
    })())
    .map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "lightCreatePoint")]
pub fn js_light_create_point(
    graphics_id: u64,
    c1: f32,
    c2: f32,
    c3: f32,
    a: f32,
    space: u8,
    intensity: f32,
    range: f32,
    radius: f32,
) -> Result<u64, JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let color = Color { c1, c2, c3, a, space };
    check((|| {
        let mode = graphics_get_color_mode(graphics_entity)?;
        light_create_point(
            graphics_entity,
            color.resolve(&mode),
            intensity,
            range,
            radius,
        )
    })())
    .map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "lightCreateSpot")]
pub fn js_light_create_spot(
    graphics_id: u64,
    c1: f32,
    c2: f32,
    c3: f32,
    a: f32,
    space: u8,
    intensity: f32,
    range: f32,
    radius: f32,
    inner_angle: f32,
    outer_angle: f32,
) -> Result<u64, JsValue> {
    let graphics_entity = Entity::from_bits(graphics_id);
    let color = Color { c1, c2, c3, a, space };
    check((|| {
        let mode = graphics_get_color_mode(graphics_entity)?;
        light_create_spot(
            graphics_entity,
            color.resolve(&mode),
            intensity,
            range,
            radius,
            inner_angle,
            outer_angle,
        )
    })())
    .map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "materialCreatePbr")]
pub fn js_material_create_pbr() -> Result<u64, JsValue> {
    check(material_create_pbr()).map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "materialSetFloat")]
pub fn js_material_set_float(mat_id: u64, name: &str, value: f32) -> Result<(), JsValue> {
    check(material_set(
        Entity::from_bits(mat_id),
        name,
        shader_value::ShaderValue::Float(value),
    ))
}

#[wasm_bindgen(js_name = "materialSetFloat4")]
pub fn js_material_set_float4(
    mat_id: u64,
    name: &str,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> Result<(), JsValue> {
    check(material_set(
        Entity::from_bits(mat_id),
        name,
        shader_value::ShaderValue::Float4([r, g, b, a]),
    ))
}

#[wasm_bindgen(js_name = "materialDestroy")]
pub fn js_material_destroy(mat_id: u64) -> Result<(), JsValue> {
    check(material_destroy(Entity::from_bits(mat_id)))
}

#[wasm_bindgen(js_name = "material")]
pub fn js_material(window_id: u64, mat_id: u64) -> Result<(), JsValue> {
    let window_entity = Entity::from_bits(window_id);
    let mat_entity = Entity::from_bits(mat_id);
    check(graphics_record_command(window_entity, DrawCommand::Material(mat_entity)))
}

#[wasm_bindgen(js_name = "shaderCreate")]
pub fn js_shader_create(source: &str) -> Result<u64, JsValue> {
    check(shader_create(source)).map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "shaderLoad")]
pub fn js_shader_load(path: &str) -> Result<u64, JsValue> {
    check(shader_load(path)).map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "shaderDestroy")]
pub fn js_shader_destroy(shader_id: u64) -> Result<(), JsValue> {
    check(shader_destroy(Entity::from_bits(shader_id)))
}

#[wasm_bindgen(js_name = "bufferCreate")]
pub fn js_buffer_create(size: u64) -> Result<u64, JsValue> {
    check(buffer_create(size)).map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "bufferCreateWithData")]
pub fn js_buffer_create_with_data(data: &[u8]) -> Result<u64, JsValue> {
    check(buffer_create_with_data(data.to_vec())).map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "bufferWrite")]
pub fn js_buffer_write(buf_id: u64, data: &[u8]) -> Result<(), JsValue> {
    check(buffer_write(Entity::from_bits(buf_id), data.to_vec()))
}

#[wasm_bindgen(js_name = "bufferSize")]
pub fn js_buffer_size(buf_id: u64) -> Result<u64, JsValue> {
    check(buffer_size(Entity::from_bits(buf_id)))
}

#[wasm_bindgen(js_name = "bufferRead")]
pub fn js_buffer_read(buf_id: u64) -> Result<Vec<u8>, JsValue> {
    check(buffer_read(Entity::from_bits(buf_id)))
}

#[wasm_bindgen(js_name = "bufferDestroy")]
pub fn js_buffer_destroy(buf_id: u64) -> Result<(), JsValue> {
    check(buffer_destroy(Entity::from_bits(buf_id)))
}

#[wasm_bindgen(js_name = "computeCreate")]
pub fn js_compute_create(shader_id: u64) -> Result<u64, JsValue> {
    check(compute_create(Entity::from_bits(shader_id))).map(|e| e.to_bits())
}

#[wasm_bindgen(js_name = "computeSetFloat")]
pub fn js_compute_set_float(compute_id: u64, name: &str, value: f32) -> Result<(), JsValue> {
    check(compute_set(
        Entity::from_bits(compute_id),
        name,
        shader_value::ShaderValue::Float(value),
    ))
}

#[wasm_bindgen(js_name = "computeSetBuffer")]
pub fn js_compute_set_buffer(compute_id: u64, name: &str, buf_id: u64) -> Result<(), JsValue> {
    check(compute_set(
        Entity::from_bits(compute_id),
        name,
        shader_value::ShaderValue::Buffer(Entity::from_bits(buf_id)),
    ))
}

#[wasm_bindgen(js_name = "computeDispatch")]
pub fn js_compute_dispatch(compute_id: u64, x: u32, y: u32, z: u32) -> Result<(), JsValue> {
    check(compute_dispatch(Entity::from_bits(compute_id), x, y, z))
}

#[wasm_bindgen(js_name = "computeDestroy")]
pub fn js_compute_destroy(compute_id: u64) -> Result<(), JsValue> {
    check(compute_destroy(Entity::from_bits(compute_id)))
}

pub const PROCESSING_MOUSE_LEFT: u8 = 0;
pub const PROCESSING_MOUSE_MIDDLE: u8 = 1;
pub const PROCESSING_MOUSE_RIGHT: u8 = 2;

pub const PROCESSING_KEY_SPACE: u32 = 32;
pub const PROCESSING_KEY_QUOTE: u32 = 39;
pub const PROCESSING_KEY_COMMA: u32 = 44;
pub const PROCESSING_KEY_MINUS: u32 = 45;
pub const PROCESSING_KEY_PERIOD: u32 = 46;
pub const PROCESSING_KEY_SLASH: u32 = 47;
pub const PROCESSING_KEY_0: u32 = 48;
pub const PROCESSING_KEY_1: u32 = 49;
pub const PROCESSING_KEY_2: u32 = 50;
pub const PROCESSING_KEY_3: u32 = 51;
pub const PROCESSING_KEY_4: u32 = 52;
pub const PROCESSING_KEY_5: u32 = 53;
pub const PROCESSING_KEY_6: u32 = 54;
pub const PROCESSING_KEY_7: u32 = 55;
pub const PROCESSING_KEY_8: u32 = 56;
pub const PROCESSING_KEY_9: u32 = 57;
pub const PROCESSING_KEY_SEMICOLON: u32 = 59;
pub const PROCESSING_KEY_EQUAL: u32 = 61;
pub const PROCESSING_KEY_A: u32 = 65;
pub const PROCESSING_KEY_B: u32 = 66;
pub const PROCESSING_KEY_C: u32 = 67;
pub const PROCESSING_KEY_D: u32 = 68;
pub const PROCESSING_KEY_E: u32 = 69;
pub const PROCESSING_KEY_F: u32 = 70;
pub const PROCESSING_KEY_G: u32 = 71;
pub const PROCESSING_KEY_H: u32 = 72;
pub const PROCESSING_KEY_I: u32 = 73;
pub const PROCESSING_KEY_J: u32 = 74;
pub const PROCESSING_KEY_K: u32 = 75;
pub const PROCESSING_KEY_L: u32 = 76;
pub const PROCESSING_KEY_M: u32 = 77;
pub const PROCESSING_KEY_N: u32 = 78;
pub const PROCESSING_KEY_O: u32 = 79;
pub const PROCESSING_KEY_P: u32 = 80;
pub const PROCESSING_KEY_Q: u32 = 81;
pub const PROCESSING_KEY_R: u32 = 82;
pub const PROCESSING_KEY_S: u32 = 83;
pub const PROCESSING_KEY_T: u32 = 84;
pub const PROCESSING_KEY_U: u32 = 85;
pub const PROCESSING_KEY_V: u32 = 86;
pub const PROCESSING_KEY_W: u32 = 87;
pub const PROCESSING_KEY_X: u32 = 88;
pub const PROCESSING_KEY_Y: u32 = 89;
pub const PROCESSING_KEY_Z: u32 = 90;
pub const PROCESSING_KEY_BRACKET_LEFT: u32 = 91;
pub const PROCESSING_KEY_BACKSLASH: u32 = 92;
pub const PROCESSING_KEY_BRACKET_RIGHT: u32 = 93;
pub const PROCESSING_KEY_BACKQUOTE: u32 = 96;
pub const PROCESSING_KEY_ESCAPE: u32 = 256;
pub const PROCESSING_KEY_ENTER: u32 = 257;
pub const PROCESSING_KEY_TAB: u32 = 258;
pub const PROCESSING_KEY_BACKSPACE: u32 = 259;
pub const PROCESSING_KEY_INSERT: u32 = 260;
pub const PROCESSING_KEY_DELETE: u32 = 261;
pub const PROCESSING_KEY_RIGHT: u32 = 262;
pub const PROCESSING_KEY_LEFT: u32 = 263;
pub const PROCESSING_KEY_DOWN: u32 = 264;
pub const PROCESSING_KEY_UP: u32 = 265;
pub const PROCESSING_KEY_PAGE_UP: u32 = 266;
pub const PROCESSING_KEY_PAGE_DOWN: u32 = 267;
pub const PROCESSING_KEY_HOME: u32 = 268;
pub const PROCESSING_KEY_END: u32 = 269;
pub const PROCESSING_KEY_CAPS_LOCK: u32 = 280;
pub const PROCESSING_KEY_SCROLL_LOCK: u32 = 281;
pub const PROCESSING_KEY_NUM_LOCK: u32 = 282;
pub const PROCESSING_KEY_PRINT_SCREEN: u32 = 283;
pub const PROCESSING_KEY_PAUSE: u32 = 284;
pub const PROCESSING_KEY_F1: u32 = 290;
pub const PROCESSING_KEY_F2: u32 = 291;
pub const PROCESSING_KEY_F3: u32 = 292;
pub const PROCESSING_KEY_F4: u32 = 293;
pub const PROCESSING_KEY_F5: u32 = 294;
pub const PROCESSING_KEY_F6: u32 = 295;
pub const PROCESSING_KEY_F7: u32 = 296;
pub const PROCESSING_KEY_F8: u32 = 297;
pub const PROCESSING_KEY_F9: u32 = 298;
pub const PROCESSING_KEY_F10: u32 = 299;
pub const PROCESSING_KEY_F11: u32 = 300;
pub const PROCESSING_KEY_F12: u32 = 301;
pub const PROCESSING_KEY_NUMPAD_0: u32 = 320;
pub const PROCESSING_KEY_NUMPAD_1: u32 = 321;
pub const PROCESSING_KEY_NUMPAD_2: u32 = 322;
pub const PROCESSING_KEY_NUMPAD_3: u32 = 323;
pub const PROCESSING_KEY_NUMPAD_4: u32 = 324;
pub const PROCESSING_KEY_NUMPAD_5: u32 = 325;
pub const PROCESSING_KEY_NUMPAD_6: u32 = 326;
pub const PROCESSING_KEY_NUMPAD_7: u32 = 327;
pub const PROCESSING_KEY_NUMPAD_8: u32 = 328;
pub const PROCESSING_KEY_NUMPAD_9: u32 = 329;
pub const PROCESSING_KEY_NUMPAD_DECIMAL: u32 = 330;
pub const PROCESSING_KEY_NUMPAD_DIVIDE: u32 = 331;
pub const PROCESSING_KEY_NUMPAD_MULTIPLY: u32 = 332;
pub const PROCESSING_KEY_NUMPAD_SUBTRACT: u32 = 333;
pub const PROCESSING_KEY_NUMPAD_ADD: u32 = 334;
pub const PROCESSING_KEY_NUMPAD_ENTER: u32 = 335;
pub const PROCESSING_KEY_NUMPAD_EQUAL: u32 = 336;
pub const PROCESSING_KEY_SHIFT_LEFT: u32 = 340;
pub const PROCESSING_KEY_CONTROL_LEFT: u32 = 341;
pub const PROCESSING_KEY_ALT_LEFT: u32 = 342;
pub const PROCESSING_KEY_SUPER_LEFT: u32 = 343;
pub const PROCESSING_KEY_SHIFT_RIGHT: u32 = 344;
pub const PROCESSING_KEY_CONTROL_RIGHT: u32 = 345;
pub const PROCESSING_KEY_ALT_RIGHT: u32 = 346;
pub const PROCESSING_KEY_SUPER_RIGHT: u32 = 347;
pub const PROCESSING_KEY_CONTEXT_MENU: u32 = 348;

#[wasm_bindgen(js_name = "inputMouseMove")]
pub fn js_input_mouse_move(surface_id: u64, x: f32, y: f32) -> Result<(), JsValue> {
    check(input_set_mouse_move(Entity::from_bits(surface_id), x, y))
}

#[wasm_bindgen(js_name = "inputMouseButton")]
pub fn js_input_mouse_button(surface_id: u64, button: u8, pressed: bool) -> Result<(), JsValue> {
    check((|| {
        let btn = match button {
            PROCESSING_MOUSE_LEFT => MouseButton::Left,
            PROCESSING_MOUSE_MIDDLE => MouseButton::Middle,
            PROCESSING_MOUSE_RIGHT => MouseButton::Right,
            _ => {
                return Err(ProcessingError::InvalidArgument(format!(
                    "invalid mouse button: {button}"
                )));
            }
        };
        input_set_mouse_button(Entity::from_bits(surface_id), btn, pressed)
    })())
}

#[wasm_bindgen(js_name = "inputScroll")]
pub fn js_input_scroll(surface_id: u64, x: f32, y: f32) -> Result<(), JsValue> {
    check(input_set_scroll(Entity::from_bits(surface_id), x, y))
}

#[wasm_bindgen(js_name = "inputKey")]
pub fn js_input_key(surface_id: u64, key_code: u32, pressed: bool) -> Result<(), JsValue> {
    check((|| {
        let kc = key_code_from_u32(key_code)?;
        input_set_key(Entity::from_bits(surface_id), kc, pressed)
    })())
}

#[wasm_bindgen(js_name = "inputChar")]
pub fn js_input_char(surface_id: u64, key_code: u32, codepoint: u32) -> Result<(), JsValue> {
    check((|| {
        let kc = key_code_from_u32(key_code)?;
        let ch = char::from_u32(codepoint).ok_or_else(|| {
            ProcessingError::InvalidArgument(format!("invalid codepoint: {codepoint}"))
        })?;
        input_set_char(Entity::from_bits(surface_id), kc, ch)
    })())
}

#[wasm_bindgen(js_name = "inputCursorEnter")]
pub fn js_input_cursor_enter(surface_id: u64) -> Result<(), JsValue> {
    check(input_set_cursor_enter(Entity::from_bits(surface_id)))
}

#[wasm_bindgen(js_name = "inputCursorLeave")]
pub fn js_input_cursor_leave(surface_id: u64) -> Result<(), JsValue> {
    check(input_set_cursor_leave(Entity::from_bits(surface_id)))
}

#[wasm_bindgen(js_name = "inputFocus")]
pub fn js_input_focus(surface_id: u64, focused: bool) -> Result<(), JsValue> {
    check(input_set_focus(Entity::from_bits(surface_id), focused))
}

#[wasm_bindgen(js_name = "inputFlush")]
pub fn js_input_flush() -> Result<(), JsValue> {
    check(input_flush())
}

#[wasm_bindgen(js_name = "mouseX")]
pub fn js_mouse_x(surface_id: u64) -> Result<f32, JsValue> {
    check(input_mouse_x(Entity::from_bits(surface_id)))
}

#[wasm_bindgen(js_name = "mouseY")]
pub fn js_mouse_y(surface_id: u64) -> Result<f32, JsValue> {
    check(input_mouse_y(Entity::from_bits(surface_id)))
}

#[wasm_bindgen(js_name = "pmouseX")]
pub fn js_pmouse_x(surface_id: u64) -> Result<f32, JsValue> {
    check(input_pmouse_x(Entity::from_bits(surface_id)))
}

#[wasm_bindgen(js_name = "pmouseY")]
pub fn js_pmouse_y(surface_id: u64) -> Result<f32, JsValue> {
    check(input_pmouse_y(Entity::from_bits(surface_id)))
}

#[wasm_bindgen(js_name = "mouseIsPressed")]
pub fn js_mouse_is_pressed() -> Result<bool, JsValue> {
    check(input_mouse_is_pressed())
}

#[wasm_bindgen(js_name = "mouseButton")]
pub fn js_mouse_button() -> Result<i8, JsValue> {
    check(input_mouse_button().map(|opt| match opt {
        Some(MouseButton::Left) => PROCESSING_MOUSE_LEFT as i8,
        Some(MouseButton::Middle) => PROCESSING_MOUSE_MIDDLE as i8,
        Some(MouseButton::Right) => PROCESSING_MOUSE_RIGHT as i8,
        _ => -1,
    }))
}

#[wasm_bindgen(js_name = "keyIsPressed")]
pub fn js_key_is_pressed() -> Result<bool, JsValue> {
    check(input_key_is_pressed())
}

#[wasm_bindgen(js_name = "keyIsDown")]
pub fn js_key_is_down(key_code: u32) -> Result<bool, JsValue> {
    check((|| {
        let kc = key_code_from_u32(key_code)?;
        input_key_is_down(kc)
    })())
}

#[wasm_bindgen(js_name = "keyJustPressed")]
pub fn js_key_just_pressed(key_code: u32) -> Result<bool, JsValue> {
    check((|| {
        let kc = key_code_from_u32(key_code)?;
        input_key_just_pressed(kc)
    })())
}

#[wasm_bindgen(js_name = "key")]
pub fn js_key() -> Result<u32, JsValue> {
    check(input_key().map(|opt| opt.map(|c| c as u32).unwrap_or(0)))
}

#[wasm_bindgen(js_name = "keyCode")]
pub fn js_key_code() -> Result<u32, JsValue> {
    check(input_key_code().map(|opt| opt.map(key_code_to_u32).unwrap_or(0)))
}

#[wasm_bindgen(js_name = "movedX")]
pub fn js_moved_x() -> Result<f32, JsValue> {
    check(input_moved_x())
}

#[wasm_bindgen(js_name = "movedY")]
pub fn js_moved_y() -> Result<f32, JsValue> {
    check(input_moved_y())
}

#[wasm_bindgen(js_name = "mouseWheel")]
pub fn js_mouse_wheel() -> Result<f32, JsValue> {
    check(input_mouse_wheel())
}

fn key_code_from_u32(val: u32) -> processing::prelude::error::Result<KeyCode> {
    match val {
        PROCESSING_KEY_SPACE => Ok(KeyCode::Space),
        PROCESSING_KEY_QUOTE => Ok(KeyCode::Quote),
        PROCESSING_KEY_COMMA => Ok(KeyCode::Comma),
        PROCESSING_KEY_MINUS => Ok(KeyCode::Minus),
        PROCESSING_KEY_PERIOD => Ok(KeyCode::Period),
        PROCESSING_KEY_SLASH => Ok(KeyCode::Slash),
        PROCESSING_KEY_0 => Ok(KeyCode::Digit0),
        PROCESSING_KEY_1 => Ok(KeyCode::Digit1),
        PROCESSING_KEY_2 => Ok(KeyCode::Digit2),
        PROCESSING_KEY_3 => Ok(KeyCode::Digit3),
        PROCESSING_KEY_4 => Ok(KeyCode::Digit4),
        PROCESSING_KEY_5 => Ok(KeyCode::Digit5),
        PROCESSING_KEY_6 => Ok(KeyCode::Digit6),
        PROCESSING_KEY_7 => Ok(KeyCode::Digit7),
        PROCESSING_KEY_8 => Ok(KeyCode::Digit8),
        PROCESSING_KEY_9 => Ok(KeyCode::Digit9),
        PROCESSING_KEY_SEMICOLON => Ok(KeyCode::Semicolon),
        PROCESSING_KEY_EQUAL => Ok(KeyCode::Equal),
        PROCESSING_KEY_A => Ok(KeyCode::KeyA),
        PROCESSING_KEY_B => Ok(KeyCode::KeyB),
        PROCESSING_KEY_C => Ok(KeyCode::KeyC),
        PROCESSING_KEY_D => Ok(KeyCode::KeyD),
        PROCESSING_KEY_E => Ok(KeyCode::KeyE),
        PROCESSING_KEY_F => Ok(KeyCode::KeyF),
        PROCESSING_KEY_G => Ok(KeyCode::KeyG),
        PROCESSING_KEY_H => Ok(KeyCode::KeyH),
        PROCESSING_KEY_I => Ok(KeyCode::KeyI),
        PROCESSING_KEY_J => Ok(KeyCode::KeyJ),
        PROCESSING_KEY_K => Ok(KeyCode::KeyK),
        PROCESSING_KEY_L => Ok(KeyCode::KeyL),
        PROCESSING_KEY_M => Ok(KeyCode::KeyM),
        PROCESSING_KEY_N => Ok(KeyCode::KeyN),
        PROCESSING_KEY_O => Ok(KeyCode::KeyO),
        PROCESSING_KEY_P => Ok(KeyCode::KeyP),
        PROCESSING_KEY_Q => Ok(KeyCode::KeyQ),
        PROCESSING_KEY_R => Ok(KeyCode::KeyR),
        PROCESSING_KEY_S => Ok(KeyCode::KeyS),
        PROCESSING_KEY_T => Ok(KeyCode::KeyT),
        PROCESSING_KEY_U => Ok(KeyCode::KeyU),
        PROCESSING_KEY_V => Ok(KeyCode::KeyV),
        PROCESSING_KEY_W => Ok(KeyCode::KeyW),
        PROCESSING_KEY_X => Ok(KeyCode::KeyX),
        PROCESSING_KEY_Y => Ok(KeyCode::KeyY),
        PROCESSING_KEY_Z => Ok(KeyCode::KeyZ),
        PROCESSING_KEY_BRACKET_LEFT => Ok(KeyCode::BracketLeft),
        PROCESSING_KEY_BACKSLASH => Ok(KeyCode::Backslash),
        PROCESSING_KEY_BRACKET_RIGHT => Ok(KeyCode::BracketRight),
        PROCESSING_KEY_BACKQUOTE => Ok(KeyCode::Backquote),
        PROCESSING_KEY_ESCAPE => Ok(KeyCode::Escape),
        PROCESSING_KEY_ENTER => Ok(KeyCode::Enter),
        PROCESSING_KEY_TAB => Ok(KeyCode::Tab),
        PROCESSING_KEY_BACKSPACE => Ok(KeyCode::Backspace),
        PROCESSING_KEY_INSERT => Ok(KeyCode::Insert),
        PROCESSING_KEY_DELETE => Ok(KeyCode::Delete),
        PROCESSING_KEY_RIGHT => Ok(KeyCode::ArrowRight),
        PROCESSING_KEY_LEFT => Ok(KeyCode::ArrowLeft),
        PROCESSING_KEY_DOWN => Ok(KeyCode::ArrowDown),
        PROCESSING_KEY_UP => Ok(KeyCode::ArrowUp),
        PROCESSING_KEY_PAGE_UP => Ok(KeyCode::PageUp),
        PROCESSING_KEY_PAGE_DOWN => Ok(KeyCode::PageDown),
        PROCESSING_KEY_HOME => Ok(KeyCode::Home),
        PROCESSING_KEY_END => Ok(KeyCode::End),
        PROCESSING_KEY_CAPS_LOCK => Ok(KeyCode::CapsLock),
        PROCESSING_KEY_SCROLL_LOCK => Ok(KeyCode::ScrollLock),
        PROCESSING_KEY_NUM_LOCK => Ok(KeyCode::NumLock),
        PROCESSING_KEY_PRINT_SCREEN => Ok(KeyCode::PrintScreen),
        PROCESSING_KEY_PAUSE => Ok(KeyCode::Pause),
        PROCESSING_KEY_F1 => Ok(KeyCode::F1),
        PROCESSING_KEY_F2 => Ok(KeyCode::F2),
        PROCESSING_KEY_F3 => Ok(KeyCode::F3),
        PROCESSING_KEY_F4 => Ok(KeyCode::F4),
        PROCESSING_KEY_F5 => Ok(KeyCode::F5),
        PROCESSING_KEY_F6 => Ok(KeyCode::F6),
        PROCESSING_KEY_F7 => Ok(KeyCode::F7),
        PROCESSING_KEY_F8 => Ok(KeyCode::F8),
        PROCESSING_KEY_F9 => Ok(KeyCode::F9),
        PROCESSING_KEY_F10 => Ok(KeyCode::F10),
        PROCESSING_KEY_F11 => Ok(KeyCode::F11),
        PROCESSING_KEY_F12 => Ok(KeyCode::F12),
        PROCESSING_KEY_NUMPAD_0 => Ok(KeyCode::Numpad0),
        PROCESSING_KEY_NUMPAD_1 => Ok(KeyCode::Numpad1),
        PROCESSING_KEY_NUMPAD_2 => Ok(KeyCode::Numpad2),
        PROCESSING_KEY_NUMPAD_3 => Ok(KeyCode::Numpad3),
        PROCESSING_KEY_NUMPAD_4 => Ok(KeyCode::Numpad4),
        PROCESSING_KEY_NUMPAD_5 => Ok(KeyCode::Numpad5),
        PROCESSING_KEY_NUMPAD_6 => Ok(KeyCode::Numpad6),
        PROCESSING_KEY_NUMPAD_7 => Ok(KeyCode::Numpad7),
        PROCESSING_KEY_NUMPAD_8 => Ok(KeyCode::Numpad8),
        PROCESSING_KEY_NUMPAD_9 => Ok(KeyCode::Numpad9),
        PROCESSING_KEY_NUMPAD_DECIMAL => Ok(KeyCode::NumpadDecimal),
        PROCESSING_KEY_NUMPAD_DIVIDE => Ok(KeyCode::NumpadDivide),
        PROCESSING_KEY_NUMPAD_MULTIPLY => Ok(KeyCode::NumpadMultiply),
        PROCESSING_KEY_NUMPAD_SUBTRACT => Ok(KeyCode::NumpadSubtract),
        PROCESSING_KEY_NUMPAD_ADD => Ok(KeyCode::NumpadAdd),
        PROCESSING_KEY_NUMPAD_ENTER => Ok(KeyCode::NumpadEnter),
        PROCESSING_KEY_NUMPAD_EQUAL => Ok(KeyCode::NumpadEqual),
        PROCESSING_KEY_SHIFT_LEFT => Ok(KeyCode::ShiftLeft),
        PROCESSING_KEY_CONTROL_LEFT => Ok(KeyCode::ControlLeft),
        PROCESSING_KEY_ALT_LEFT => Ok(KeyCode::AltLeft),
        PROCESSING_KEY_SUPER_LEFT => Ok(KeyCode::SuperLeft),
        PROCESSING_KEY_SHIFT_RIGHT => Ok(KeyCode::ShiftRight),
        PROCESSING_KEY_CONTROL_RIGHT => Ok(KeyCode::ControlRight),
        PROCESSING_KEY_ALT_RIGHT => Ok(KeyCode::AltRight),
        PROCESSING_KEY_SUPER_RIGHT => Ok(KeyCode::SuperRight),
        PROCESSING_KEY_CONTEXT_MENU => Ok(KeyCode::ContextMenu),
        _ => Err(ProcessingError::InvalidArgument(format!(
            "unknown key code: {val}"
        ))),
    }
}

fn key_code_to_u32(kc: KeyCode) -> u32 {
    match kc {
        KeyCode::Space => PROCESSING_KEY_SPACE,
        KeyCode::Quote => PROCESSING_KEY_QUOTE,
        KeyCode::Comma => PROCESSING_KEY_COMMA,
        KeyCode::Minus => PROCESSING_KEY_MINUS,
        KeyCode::Period => PROCESSING_KEY_PERIOD,
        KeyCode::Slash => PROCESSING_KEY_SLASH,
        KeyCode::Digit0 => PROCESSING_KEY_0,
        KeyCode::Digit1 => PROCESSING_KEY_1,
        KeyCode::Digit2 => PROCESSING_KEY_2,
        KeyCode::Digit3 => PROCESSING_KEY_3,
        KeyCode::Digit4 => PROCESSING_KEY_4,
        KeyCode::Digit5 => PROCESSING_KEY_5,
        KeyCode::Digit6 => PROCESSING_KEY_6,
        KeyCode::Digit7 => PROCESSING_KEY_7,
        KeyCode::Digit8 => PROCESSING_KEY_8,
        KeyCode::Digit9 => PROCESSING_KEY_9,
        KeyCode::Semicolon => PROCESSING_KEY_SEMICOLON,
        KeyCode::Equal => PROCESSING_KEY_EQUAL,
        KeyCode::KeyA => PROCESSING_KEY_A,
        KeyCode::KeyB => PROCESSING_KEY_B,
        KeyCode::KeyC => PROCESSING_KEY_C,
        KeyCode::KeyD => PROCESSING_KEY_D,
        KeyCode::KeyE => PROCESSING_KEY_E,
        KeyCode::KeyF => PROCESSING_KEY_F,
        KeyCode::KeyG => PROCESSING_KEY_G,
        KeyCode::KeyH => PROCESSING_KEY_H,
        KeyCode::KeyI => PROCESSING_KEY_I,
        KeyCode::KeyJ => PROCESSING_KEY_J,
        KeyCode::KeyK => PROCESSING_KEY_K,
        KeyCode::KeyL => PROCESSING_KEY_L,
        KeyCode::KeyM => PROCESSING_KEY_M,
        KeyCode::KeyN => PROCESSING_KEY_N,
        KeyCode::KeyO => PROCESSING_KEY_O,
        KeyCode::KeyP => PROCESSING_KEY_P,
        KeyCode::KeyQ => PROCESSING_KEY_Q,
        KeyCode::KeyR => PROCESSING_KEY_R,
        KeyCode::KeyS => PROCESSING_KEY_S,
        KeyCode::KeyT => PROCESSING_KEY_T,
        KeyCode::KeyU => PROCESSING_KEY_U,
        KeyCode::KeyV => PROCESSING_KEY_V,
        KeyCode::KeyW => PROCESSING_KEY_W,
        KeyCode::KeyX => PROCESSING_KEY_X,
        KeyCode::KeyY => PROCESSING_KEY_Y,
        KeyCode::KeyZ => PROCESSING_KEY_Z,
        KeyCode::BracketLeft => PROCESSING_KEY_BRACKET_LEFT,
        KeyCode::Backslash => PROCESSING_KEY_BACKSLASH,
        KeyCode::BracketRight => PROCESSING_KEY_BRACKET_RIGHT,
        KeyCode::Backquote => PROCESSING_KEY_BACKQUOTE,
        KeyCode::Escape => PROCESSING_KEY_ESCAPE,
        KeyCode::Enter => PROCESSING_KEY_ENTER,
        KeyCode::Tab => PROCESSING_KEY_TAB,
        KeyCode::Backspace => PROCESSING_KEY_BACKSPACE,
        KeyCode::Insert => PROCESSING_KEY_INSERT,
        KeyCode::Delete => PROCESSING_KEY_DELETE,
        KeyCode::ArrowRight => PROCESSING_KEY_RIGHT,
        KeyCode::ArrowLeft => PROCESSING_KEY_LEFT,
        KeyCode::ArrowDown => PROCESSING_KEY_DOWN,
        KeyCode::ArrowUp => PROCESSING_KEY_UP,
        KeyCode::PageUp => PROCESSING_KEY_PAGE_UP,
        KeyCode::PageDown => PROCESSING_KEY_PAGE_DOWN,
        KeyCode::Home => PROCESSING_KEY_HOME,
        KeyCode::End => PROCESSING_KEY_END,
        KeyCode::CapsLock => PROCESSING_KEY_CAPS_LOCK,
        KeyCode::ScrollLock => PROCESSING_KEY_SCROLL_LOCK,
        KeyCode::NumLock => PROCESSING_KEY_NUM_LOCK,
        KeyCode::PrintScreen => PROCESSING_KEY_PRINT_SCREEN,
        KeyCode::Pause => PROCESSING_KEY_PAUSE,
        KeyCode::F1 => PROCESSING_KEY_F1,
        KeyCode::F2 => PROCESSING_KEY_F2,
        KeyCode::F3 => PROCESSING_KEY_F3,
        KeyCode::F4 => PROCESSING_KEY_F4,
        KeyCode::F5 => PROCESSING_KEY_F5,
        KeyCode::F6 => PROCESSING_KEY_F6,
        KeyCode::F7 => PROCESSING_KEY_F7,
        KeyCode::F8 => PROCESSING_KEY_F8,
        KeyCode::F9 => PROCESSING_KEY_F9,
        KeyCode::F10 => PROCESSING_KEY_F10,
        KeyCode::F11 => PROCESSING_KEY_F11,
        KeyCode::F12 => PROCESSING_KEY_F12,
        KeyCode::Numpad0 => PROCESSING_KEY_NUMPAD_0,
        KeyCode::Numpad1 => PROCESSING_KEY_NUMPAD_1,
        KeyCode::Numpad2 => PROCESSING_KEY_NUMPAD_2,
        KeyCode::Numpad3 => PROCESSING_KEY_NUMPAD_3,
        KeyCode::Numpad4 => PROCESSING_KEY_NUMPAD_4,
        KeyCode::Numpad5 => PROCESSING_KEY_NUMPAD_5,
        KeyCode::Numpad6 => PROCESSING_KEY_NUMPAD_6,
        KeyCode::Numpad7 => PROCESSING_KEY_NUMPAD_7,
        KeyCode::Numpad8 => PROCESSING_KEY_NUMPAD_8,
        KeyCode::Numpad9 => PROCESSING_KEY_NUMPAD_9,
        KeyCode::NumpadDecimal => PROCESSING_KEY_NUMPAD_DECIMAL,
        KeyCode::NumpadDivide => PROCESSING_KEY_NUMPAD_DIVIDE,
        KeyCode::NumpadMultiply => PROCESSING_KEY_NUMPAD_MULTIPLY,
        KeyCode::NumpadSubtract => PROCESSING_KEY_NUMPAD_SUBTRACT,
        KeyCode::NumpadAdd => PROCESSING_KEY_NUMPAD_ADD,
        KeyCode::NumpadEnter => PROCESSING_KEY_NUMPAD_ENTER,
        KeyCode::NumpadEqual => PROCESSING_KEY_NUMPAD_EQUAL,
        KeyCode::ShiftLeft => PROCESSING_KEY_SHIFT_LEFT,
        KeyCode::ControlLeft => PROCESSING_KEY_CONTROL_LEFT,
        KeyCode::AltLeft => PROCESSING_KEY_ALT_LEFT,
        KeyCode::SuperLeft => PROCESSING_KEY_SUPER_LEFT,
        KeyCode::ShiftRight => PROCESSING_KEY_SHIFT_RIGHT,
        KeyCode::ControlRight => PROCESSING_KEY_CONTROL_RIGHT,
        KeyCode::AltRight => PROCESSING_KEY_ALT_RIGHT,
        KeyCode::SuperRight => PROCESSING_KEY_SUPER_RIGHT,
        KeyCode::ContextMenu => PROCESSING_KEY_CONTEXT_MENU,
        _ => 0,
    }
}

#[wasm_bindgen(js_name = "geometryAttributeRotation")]
pub fn js_geometry_attribute_rotation() -> u64 {
    geometry_attribute_rotation().to_bits()
}

#[wasm_bindgen(js_name = "geometryAttributeScale")]
pub fn js_geometry_attribute_scale() -> u64 {
    geometry_attribute_scale().to_bits()
}

#[wasm_bindgen(js_name = "geometryAttributeDead")]
pub fn js_geometry_attribute_dead() -> u64 {
    geometry_attribute_dead().to_bits()
}

#[wasm_bindgen(js_name = "particlesCreate")]
pub fn js_particles_create(capacity: u32, attribute_entities: Vec<u64>) -> Result<u64, JsValue> {
    let attrs: Vec<Entity> = attribute_entities.into_iter().map(Entity::from_bits).collect();
    check(particles_create(capacity, attrs).map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "particlesCreateFromGeometry")]
pub fn js_particles_create_from_geometry(
    geometry: u64,
    attribute_entities: Vec<u64>,
) -> Result<u64, JsValue> {
    let attrs: Vec<Entity> = attribute_entities.into_iter().map(Entity::from_bits).collect();
    check(
        particles_create_from_geometry(Entity::from_bits(geometry), attrs).map(|e| e.to_bits()),
    )
}

#[wasm_bindgen(js_name = "particlesDestroy")]
pub fn js_particles_destroy(entity: u64) -> Result<(), JsValue> {
    check(particles_destroy(Entity::from_bits(entity)))
}

#[wasm_bindgen(js_name = "particlesCapacity")]
pub fn js_particles_capacity(entity: u64) -> Result<u32, JsValue> {
    check(particles_capacity(Entity::from_bits(entity)))
}

#[wasm_bindgen(js_name = "particlesBuffer")]
pub fn js_particles_buffer(entity: u64, attribute: u64) -> Result<Option<u64>, JsValue> {
    check(particles_buffer(
        Entity::from_bits(entity),
        Entity::from_bits(attribute),
    ))
    .map(|opt| opt.map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "particlesEmit")]
pub fn js_particles_emit(entity: u64, count: u32) -> Result<(), JsValue> {
    check(particles_emit(Entity::from_bits(entity), count, vec![]))
}

#[wasm_bindgen(js_name = "particlesEmitGpu")]
pub fn js_particles_emit_gpu(entity: u64, count: u32, compute: u64) -> Result<(), JsValue> {
    check(particles_emit_gpu(
        Entity::from_bits(entity),
        count,
        Entity::from_bits(compute),
    ))
}

#[wasm_bindgen(js_name = "particlesApply")]
pub fn js_particles_apply(particles: u64, compute: u64) -> Result<(), JsValue> {
    check(particles_apply(
        Entity::from_bits(particles),
        Entity::from_bits(compute),
    ))
}

#[wasm_bindgen(js_name = "particlesKernelNoise")]
pub fn js_particles_kernel_noise() -> Result<u64, JsValue> {
    check(particles_kernel_noise().map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "particlesKernelTransform")]
pub fn js_particles_kernel_transform() -> Result<u64, JsValue> {
    check(particles_kernel_transform().map(|e| e.to_bits()))
}

#[wasm_bindgen(js_name = "particles")]
pub fn js_particles(graphics_id: u64, particles: u64, geometry: u64) -> Result<(), JsValue> {
    check(graphics_record_command(
        Entity::from_bits(graphics_id),
        DrawCommand::Particles {
            particles: Entity::from_bits(particles),
            geometry: Entity::from_bits(geometry),
        },
    ))
}

#[wasm_bindgen(js_name = "materialSetAlbedoColor")]
pub fn js_material_set_albedo_color(
    entity: u64,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> Result<(), JsValue> {
    check(material_set_albedo_color(
        Entity::from_bits(entity),
        [r, g, b, a],
    ))
}

#[wasm_bindgen(js_name = "materialSetAlbedoBuffer")]
pub fn js_material_set_albedo_buffer(entity: u64, buffer: u64) -> Result<(), JsValue> {
    check(material_set_albedo_buffer(
        Entity::from_bits(entity),
        Entity::from_bits(buffer),
    ))
}
