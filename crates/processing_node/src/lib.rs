use std::cell::RefCell;
use std::sync::Once;

use bevy::{
    color::LinearRgba,
    math::{Affine3A, Mat4, Vec2, Vec3, Vec4},
    prelude::Entity,
    render::render_resource::{Extent3d, TextureFormat},
};
use napi::bindgen_prelude::{
    BigInt, Float32Array, FnArgs, Function, JsObjectValue, Object, Uint8Array, Uint32Array, Unknown,
};
use napi::{Env, Error, Result};
use napi_derive::napi;
use processing::prelude::color::{ColorMode, ColorSpace};
use processing::prelude::error::ProcessingError;
use processing::prelude::{shader_value::ShaderValue, *};
use processing_glfw::GlfwContext;

fn check<T, E: std::fmt::Display>(result: std::result::Result<T, E>) -> Result<T> {
    result.map_err(|e| Error::from_reason(e.to_string()))
}

// Entity ids cross the boundary as BigInt, matching processing_wasm's u64.
fn ent(id: BigInt) -> Entity {
    Entity::from_bits(id.get_u64().1)
}

fn id(entity: Entity) -> BigInt {
    BigInt::from(entity.to_bits())
}

#[derive(Debug, Clone, Copy)]
struct Color {
    c1: f32,
    c2: f32,
    c3: f32,
    a: f32,
    space: u8,
}

impl Color {
    fn new(c1: f64, c2: f64, c3: f64, a: f64, space: u8) -> Self {
        Color {
            c1: c1 as f32,
            c2: c2 as f32,
            c3: c3 as f32,
            a: a as f32,
            space,
        }
    }

    fn resolve(self, mode: &ColorMode) -> bevy::color::Color {
        let c1 = mode.scale(self.c1, 0);
        let c2 = mode.scale(self.c2, 1);
        let c3 = mode.scale(self.c3, 2);
        let ca = mode.scale(self.a, 3);
        mode.space.color(c1, c2, c3, ca)
    }

    fn to_linear(self) -> LinearRgba {
        ColorSpace::from_u8(self.space)
            .unwrap_or(ColorSpace::Linear)
            .color(self.c1, self.c2, self.c3, self.a)
            .to_linear()
    }
}

fn linear_to_floats(colors: &[LinearRgba]) -> Float32Array {
    let mut out = Vec::with_capacity(colors.len() * 4);
    for c in colors {
        out.extend_from_slice(&[c.red, c.green, c.blue, c.alpha]);
    }
    out.into()
}

fn floats_to_linear(pixels: &[f32], space: u8) -> Vec<LinearRgba> {
    pixels
        .as_chunks::<4>().0.iter()
        .map(|p| {
            Color {
                c1: p[0],
                c2: p[1],
                c3: p[2],
                a: p[3],
                space,
            }
            .to_linear()
        })
        .collect()
}

fn record(graphics_id: BigInt, cmd: DrawCommand) -> Result<()> {
    check(graphics_record_command(ent(graphics_id), cmd))
}

thread_local! {
    // GLFW's event pump is global; one context drives every window on this thread.
    static GLFW: RefCell<Option<GlfwContext>> = const { RefCell::new(None) };
}

#[napi(object)]
pub struct InitConfig {
    pub asset_root_path: Option<String>,
    pub sketch_root_path: Option<String>,
    pub sketch_file_name: Option<String>,
    pub log_level: Option<String>,
}

// Tear down while the thread-locals are still alive; the eager TLS destructor
// aborts inside a Bevy resource drop.
fn teardown() {
    let _ = exit(0);
    GLFW.with(|cell| cell.borrow_mut().take());
}

fn install_teardown_hooks(env: &Env) -> Result<()> {
    env.add_env_cleanup_hook((), |_| teardown())?;
    // process.exit() skips env cleanup hooks.
    let process: Object = env.get_global()?.get_named_property("process")?;
    let once: Function<FnArgs<(&str, Function<(), ()>)>, Unknown> =
        process.get_named_property("once")?;
    let hook = env.create_function_from_closure("processingTeardown", |_| {
        teardown();
        Ok(())
    })?;
    once.apply(process, ("exit", hook).into())?;
    Ok(())
}

/// Must be called from the main thread.
#[napi(js_name = "init")]
pub fn js_init(env: Env, config: Option<InitConfig>) -> Result<()> {
    static TEARDOWN_HOOKS: Once = Once::new();
    let mut hooked = Ok(());
    TEARDOWN_HOOKS.call_once(|| hooked = install_teardown_hooks(&env));
    hooked?;
    let mut cfg = Config::new();
    if let Some(c) = config {
        for (key, value) in [
            (ConfigKey::AssetRootPath, c.asset_root_path),
            (ConfigKey::SketchRootPath, c.sketch_root_path),
            (ConfigKey::SketchFileName, c.sketch_file_name),
            (ConfigKey::LogLevel, c.log_level),
        ] {
            if let Some(value) = value {
                cfg.set(key, value);
            }
        }
    }
    check(init(cfg))
}

/// Opens a GLFW window and returns its surface. The first call creates the main
/// window; later calls add secondary windows on the same GLFW instance.
#[napi(js_name = "surfaceCreateWindow")]
pub fn js_surface_create_window(
    width: u32,
    height: u32,
    transparent: Option<bool>,
    title: Option<String>,
) -> Result<BigInt> {
    let transparent = transparent.unwrap_or(false);
    let title = title.unwrap_or_else(|| "Processing".to_string());
    GLFW.with(|cell| {
        let mut ctx = cell.borrow_mut();
        let surface = match ctx.as_mut() {
            Some(glfw) => check(glfw.add_window(width, height, transparent, &title))?,
            None => {
                let mut glfw = check(GlfwContext::new(width, height, transparent))?;
                let surface = check(glfw.create_surface(width, height, transparent))?;
                *ctx = Some(glfw);
                surface
            }
        };
        check(surface_set_title(surface, title))?;
        Ok(id(surface))
    })
}

#[napi(js_name = "surfaceCreateOffscreen")]
pub fn js_surface_create_offscreen(
    width: u32,
    height: u32,
    scale_factor: f64,
    hdr: Option<bool>,
) -> Result<BigInt> {
    let format = if hdr.unwrap_or(false) {
        TextureFormat::Rgba16Float
    } else {
        TextureFormat::Rgba8UnormSrgb
    };
    check(surface_create_offscreen(
        width,
        height,
        scale_factor as f32,
        format,
    ))
    .map(id)
}

/// Pumps GLFW events and flushes input. Returns false once the main window closes.
#[napi(js_name = "pollEvents")]
pub fn js_poll_events() -> Result<bool> {
    GLFW.with(|cell| match cell.borrow_mut().as_mut() {
        Some(glfw) => Ok(glfw.poll_events()),
        None => Err(Error::from_reason(
            "no window; call surfaceCreateWindow first",
        )),
    })
}

#[napi(js_name = "graphicsCreate")]
pub fn js_graphics_create(surface_id: BigInt, width: u32, height: u32) -> Result<BigInt> {
    check(graphics_create(
        ent(surface_id),
        width,
        height,
        TextureFormat::Rgba16Float,
    ))
    .map(id)
}

#[napi(js_name = "graphicsDestroy")]
pub fn js_graphics_destroy(graphics_id: BigInt) -> Result<()> {
    check(graphics_destroy(ent(graphics_id)))
}

/// Destroying the main window's surface also tears down the GLFW context.
#[napi(js_name = "surfaceDestroy")]
pub fn js_surface_destroy(surface_id: BigInt) -> Result<()> {
    let surface = ent(surface_id);
    check(surface_destroy(surface))?;
    GLFW.with(|cell| {
        let mut ctx = cell.borrow_mut();
        if ctx.as_ref().and_then(|g| g.main_surface()) == Some(surface) {
            *ctx = None;
        }
    });
    Ok(())
}

#[napi(js_name = "surfaceResize")]
pub fn js_surface_resize(surface_id: BigInt, width: u32, height: u32) -> Result<()> {
    check(surface_resize(ent(surface_id), width, height))
}

#[napi(js_name = "backgroundColor")]
pub fn js_background_color(
    graphics_id: BigInt,
    c1: f64,
    c2: f64,
    c3: f64,
    a: f64,
    space: u8,
) -> Result<()> {
    let graphics = ent(graphics_id);
    let color = Color::new(c1, c2, c3, a, space);
    check((|| {
        let mode = graphics_get_color_mode(graphics)?;
        graphics_record_command(graphics, DrawCommand::BackgroundColor(color.resolve(&mode)))
    })())
}

#[napi(js_name = "backgroundImage")]
pub fn js_background_image(graphics_id: BigInt, image_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::BackgroundImage(ent(image_id)))
}

#[napi(js_name = "clear")]
pub fn js_clear(graphics_id: BigInt) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::BackgroundColor(bevy::prelude::Color::NONE),
    )
}

#[napi(js_name = "beginDraw")]
pub fn js_begin_draw(graphics_id: BigInt) -> Result<()> {
    check(graphics_begin_draw(ent(graphics_id)))
}

#[napi(js_name = "flush")]
pub fn js_flush(graphics_id: BigInt) -> Result<()> {
    check(graphics_flush(ent(graphics_id)))
}

#[napi(js_name = "endDraw")]
pub fn js_end_draw(graphics_id: BigInt) -> Result<()> {
    check(graphics_end_draw(ent(graphics_id)))
}

/// Shuts down internal resources but does not terminate the process.
#[napi(js_name = "exit")]
pub fn js_exit(exit_code: u8) -> Result<()> {
    check(exit(exit_code))
}

#[napi(js_name = "colorMode")]
pub fn js_color_mode(
    graphics_id: BigInt,
    space: u8,
    max1: f64,
    max2: f64,
    max3: f64,
    max_alpha: f64,
) -> Result<()> {
    let graphics = ent(graphics_id);
    check((|| {
        let space = ColorSpace::from_u8(space).ok_or_else(|| {
            ProcessingError::InvalidArgument(format!("unknown color space: {space}"))
        })?;
        let mode = ColorMode::new(
            space,
            max1 as f32,
            max2 as f32,
            max3 as f32,
            max_alpha as f32,
        );
        graphics_set_color_mode(graphics, mode)
    })())
}

#[napi(js_name = "setFill")]
pub fn js_set_fill(
    graphics_id: BigInt,
    c1: f64,
    c2: f64,
    c3: f64,
    a: f64,
    space: u8,
) -> Result<()> {
    let graphics = ent(graphics_id);
    let color = Color::new(c1, c2, c3, a, space);
    check((|| {
        let mode = graphics_get_color_mode(graphics)?;
        graphics_record_command(graphics, DrawCommand::Fill(color.resolve(&mode)))
    })())
}

#[napi(js_name = "setStrokeColor")]
pub fn js_set_stroke_color(
    graphics_id: BigInt,
    c1: f64,
    c2: f64,
    c3: f64,
    a: f64,
    space: u8,
) -> Result<()> {
    let graphics = ent(graphics_id);
    let color = Color::new(c1, c2, c3, a, space);
    check((|| {
        let mode = graphics_get_color_mode(graphics)?;
        graphics_record_command(graphics, DrawCommand::StrokeColor(color.resolve(&mode)))
    })())
}

#[napi(js_name = "setStrokeWeight")]
pub fn js_set_stroke_weight(graphics_id: BigInt, weight: f64) -> Result<()> {
    record(graphics_id, DrawCommand::StrokeWeight(weight as f32))
}

#[napi(js_name = "setStrokeCap")]
pub fn js_set_stroke_cap(graphics_id: BigInt, cap: u8) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::StrokeCap(StrokeCapMode::from(cap)),
    )
}

#[napi(js_name = "setStrokeJoin")]
pub fn js_set_stroke_join(graphics_id: BigInt, join: u8) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::StrokeJoin(StrokeJoinMode::from(join)),
    )
}

#[napi(js_name = "rectMode")]
pub fn js_rect_mode(graphics_id: BigInt, mode: u8) -> Result<()> {
    record(graphics_id, DrawCommand::RectMode(ShapeMode::from(mode)))
}

#[napi(js_name = "ellipseMode")]
pub fn js_ellipse_mode(graphics_id: BigInt, mode: u8) -> Result<()> {
    record(graphics_id, DrawCommand::EllipseMode(ShapeMode::from(mode)))
}

#[napi(js_name = "noFill")]
pub fn js_no_fill(graphics_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::NoFill)
}

#[napi(js_name = "noStroke")]
pub fn js_no_stroke(graphics_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::NoStroke)
}

#[napi(js_name = "pushMatrix")]
pub fn js_push_matrix(graphics_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::PushMatrix)
}

#[napi(js_name = "popMatrix")]
pub fn js_pop_matrix(graphics_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::PopMatrix)
}

#[napi(js_name = "pushStyle")]
pub fn js_push_style(graphics_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::PushStyle)
}

#[napi(js_name = "popStyle")]
pub fn js_pop_style(graphics_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::PopStyle)
}

#[napi(js_name = "push")]
pub fn js_push(graphics_id: BigInt) -> Result<()> {
    let graphics = ent(graphics_id);
    check(graphics_record_command(graphics, DrawCommand::PushStyle))?;
    check(graphics_record_command(graphics, DrawCommand::PushMatrix))
}

#[napi(js_name = "pop")]
pub fn js_pop(graphics_id: BigInt) -> Result<()> {
    let graphics = ent(graphics_id);
    check(graphics_record_command(graphics, DrawCommand::PopStyle))?;
    check(graphics_record_command(graphics, DrawCommand::PopMatrix))
}

#[napi(js_name = "resetMatrix")]
pub fn js_reset_matrix(graphics_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::ResetMatrix)
}

#[napi(js_name = "translate")]
pub fn js_translate(graphics_id: BigInt, x: f64, y: f64, z: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Translate(Vec3::new(x as f32, y as f32, z as f32)),
    )
}

/// Rotates by `angle` about the axis (x, y, z).
#[napi(js_name = "rotate")]
pub fn js_rotate(graphics_id: BigInt, angle: f64, x: f64, y: f64, z: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Rotate {
            angle: angle as f32,
            axis: Vec3::new(x as f32, y as f32, z as f32),
        },
    )
}

#[napi(js_name = "scale")]
pub fn js_scale(graphics_id: BigInt, x: f64, y: f64, z: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Scale(Vec3::new(x as f32, y as f32, z as f32)),
    )
}

#[napi(js_name = "shearX")]
pub fn js_shear_x(graphics_id: BigInt, angle: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::ShearX {
            angle: angle as f32,
        },
    )
}

#[napi(js_name = "shearY")]
pub fn js_shear_y(graphics_id: BigInt, angle: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::ShearY {
            angle: angle as f32,
        },
    )
}

#[napi(js_name = "setBlendMode")]
pub fn js_set_blend_mode(graphics_id: BigInt, mode: u8) -> Result<()> {
    let graphics = ent(graphics_id);
    check((|| {
        let blend_state = BlendMode::try_from(mode)?.to_blend_state();
        graphics_record_command(graphics, DrawCommand::BlendMode(blend_state))
    })())
}

#[napi(js_name = "setCustomBlendMode")]
pub fn js_set_custom_blend_mode(
    graphics_id: BigInt,
    color_src: u8,
    color_dst: u8,
    color_op: u8,
    alpha_src: u8,
    alpha_dst: u8,
    alpha_op: u8,
) -> Result<()> {
    let graphics = ent(graphics_id);
    check((|| {
        let blend_state = custom_blend_state(
            color_src, color_dst, color_op, alpha_src, alpha_dst, alpha_op,
        )?;
        graphics_record_command(graphics, DrawCommand::BlendMode(Some(blend_state)))
    })())
}

#[napi(js_name = "rect")]
pub fn js_rect(
    graphics_id: BigInt,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    tl: f64,
    tr: f64,
    br: f64,
    bl: f64,
) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Rect {
            x: x as f32,
            y: y as f32,
            w: w as f32,
            h: h as f32,
            radii: [tl as f32, tr as f32, br as f32, bl as f32],
        },
    )
}

#[napi(js_name = "ellipse")]
pub fn js_ellipse(graphics_id: BigInt, cx: f64, cy: f64, w: f64, h: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Ellipse {
            cx: cx as f32,
            cy: cy as f32,
            w: w as f32,
            h: h as f32,
        },
    )
}

#[napi(js_name = "circle")]
pub fn js_circle(graphics_id: BigInt, cx: f64, cy: f64, d: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Ellipse {
            cx: cx as f32,
            cy: cy as f32,
            w: d as f32,
            h: d as f32,
        },
    )
}

#[napi(js_name = "line")]
pub fn js_line(graphics_id: BigInt, x1: f64, y1: f64, x2: f64, y2: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Line {
            x1: x1 as f32,
            y1: y1 as f32,
            x2: x2 as f32,
            y2: y2 as f32,
        },
    )
}

#[napi(js_name = "triangle")]
pub fn js_triangle(
    graphics_id: BigInt,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    x3: f64,
    y3: f64,
) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Triangle {
            x1: x1 as f32,
            y1: y1 as f32,
            x2: x2 as f32,
            y2: y2 as f32,
            x3: x3 as f32,
            y3: y3 as f32,
        },
    )
}

#[napi(js_name = "quad")]
pub fn js_quad(
    graphics_id: BigInt,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    x3: f64,
    y3: f64,
    x4: f64,
    y4: f64,
) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Quad {
            x1: x1 as f32,
            y1: y1 as f32,
            x2: x2 as f32,
            y2: y2 as f32,
            x3: x3 as f32,
            y3: y3 as f32,
            x4: x4 as f32,
            y4: y4 as f32,
        },
    )
}

#[napi(js_name = "point")]
pub fn js_point(graphics_id: BigInt, x: f64, y: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Point {
            x: x as f32,
            y: y as f32,
        },
    )
}

#[napi(js_name = "square")]
pub fn js_square(graphics_id: BigInt, x: f64, y: f64, s: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Rect {
            x: x as f32,
            y: y as f32,
            w: s as f32,
            h: s as f32,
            radii: [0.0; 4],
        },
    )
}

#[napi(js_name = "arc")]
pub fn js_arc(
    graphics_id: BigInt,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    start: f64,
    stop: f64,
    mode: u8,
) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Arc {
            cx: cx as f32,
            cy: cy as f32,
            w: w as f32,
            h: h as f32,
            start: start as f32,
            stop: stop as f32,
            mode: ArcMode::from(mode),
        },
    )
}

#[napi(js_name = "bezier")]
pub fn js_bezier(
    graphics_id: BigInt,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    x3: f64,
    y3: f64,
    x4: f64,
    y4: f64,
) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Bezier {
            x1: x1 as f32,
            y1: y1 as f32,
            x2: x2 as f32,
            y2: y2 as f32,
            x3: x3 as f32,
            y3: y3 as f32,
            x4: x4 as f32,
            y4: y4 as f32,
        },
    )
}

#[napi(js_name = "curve")]
pub fn js_curve(
    graphics_id: BigInt,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    x3: f64,
    y3: f64,
    x4: f64,
    y4: f64,
) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Curve {
            x1: x1 as f32,
            y1: y1 as f32,
            x2: x2 as f32,
            y2: y2 as f32,
            x3: x3 as f32,
            y3: y3 as f32,
            x4: x4 as f32,
            y4: y4 as f32,
        },
    )
}

#[napi(js_name = "box")]
pub fn js_box(graphics_id: BigInt, width: f64, height: f64, depth: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Box {
            width: width as f32,
            height: height as f32,
            depth: depth as f32,
        },
    )
}

#[napi(js_name = "sphere")]
pub fn js_sphere(graphics_id: BigInt, radius: f64, sectors: u32, stacks: u32) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Sphere {
            radius: radius as f32,
            sectors,
            stacks,
        },
    )
}

#[napi(js_name = "cylinder")]
pub fn js_cylinder(graphics_id: BigInt, radius: f64, height: f64, detail: u32) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Cylinder {
            radius: radius as f32,
            height: height as f32,
            detail,
        },
    )
}

#[napi(js_name = "cone")]
pub fn js_cone(graphics_id: BigInt, radius: f64, height: f64, detail: u32) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Cone {
            radius: radius as f32,
            height: height as f32,
            detail,
        },
    )
}

#[napi(js_name = "torus")]
pub fn js_torus(
    graphics_id: BigInt,
    radius: f64,
    tube_radius: f64,
    major_segments: u32,
    minor_segments: u32,
) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Torus {
            radius: radius as f32,
            tube_radius: tube_radius as f32,
            major_segments,
            minor_segments,
        },
    )
}

#[napi(js_name = "plane")]
pub fn js_plane(graphics_id: BigInt, width: f64, height: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Plane {
            width: width as f32,
            height: height as f32,
        },
    )
}

#[napi(js_name = "capsule")]
pub fn js_capsule(graphics_id: BigInt, radius: f64, length: f64, detail: u32) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Capsule {
            radius: radius as f32,
            length: length as f32,
            detail,
        },
    )
}

#[napi(js_name = "conicalFrustum")]
pub fn js_conical_frustum(
    graphics_id: BigInt,
    radius_top: f64,
    radius_bottom: f64,
    height: f64,
    detail: u32,
) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::ConicalFrustum {
            radius_top: radius_top as f32,
            radius_bottom: radius_bottom as f32,
            height: height as f32,
            detail,
        },
    )
}

#[napi(js_name = "tetrahedron")]
pub fn js_tetrahedron(graphics_id: BigInt, radius: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Tetrahedron {
            radius: radius as f32,
        },
    )
}

#[napi(js_name = "beginShape")]
pub fn js_begin_shape(graphics_id: BigInt, kind: u8) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::BeginShape {
            kind: ShapeKind::from(kind),
        },
    )
}

#[napi(js_name = "endShape")]
pub fn js_end_shape(graphics_id: BigInt, close: bool) -> Result<()> {
    record(graphics_id, DrawCommand::EndShape { close })
}

#[napi(js_name = "vertex")]
pub fn js_vertex(graphics_id: BigInt, x: f64, y: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::ShapeVertex {
            x: x as f32,
            y: y as f32,
        },
    )
}

#[napi(js_name = "bezierVertex")]
pub fn js_bezier_vertex(
    graphics_id: BigInt,
    cx1: f64,
    cy1: f64,
    cx2: f64,
    cy2: f64,
    x: f64,
    y: f64,
) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::ShapeBezierVertex {
            cx1: cx1 as f32,
            cy1: cy1 as f32,
            cx2: cx2 as f32,
            cy2: cy2 as f32,
            x: x as f32,
            y: y as f32,
        },
    )
}

#[napi(js_name = "quadraticVertex")]
pub fn js_quadratic_vertex(graphics_id: BigInt, cx: f64, cy: f64, x: f64, y: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::ShapeQuadraticVertex {
            cx: cx as f32,
            cy: cy as f32,
            x: x as f32,
            y: y as f32,
        },
    )
}

#[napi(js_name = "curveVertex")]
pub fn js_curve_vertex(graphics_id: BigInt, x: f64, y: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::ShapeCurveVertex {
            x: x as f32,
            y: y as f32,
        },
    )
}

#[napi(js_name = "beginContour")]
pub fn js_begin_contour(graphics_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::BeginContour)
}

#[napi(js_name = "endContour")]
pub fn js_end_contour(graphics_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::EndContour)
}

#[napi(js_name = "loadFont")]
pub fn js_load_font(path: String) -> Result<BigInt> {
    check(font_load(&path)).map(id)
}

#[napi(js_name = "createFont")]
pub fn js_create_font(name: String) -> Result<BigInt> {
    check(font_create(&name)).map(id)
}

#[napi(js_name = "fontVariationCount")]
pub fn js_font_variation_count(font_id: BigInt) -> Result<u32> {
    check(font_variations(ent(font_id))).map(|v| v.len() as u32)
}

#[napi(object)]
pub struct FontVariation {
    pub tag: String,
    pub min: f64,
    pub max: f64,
    pub default: f64,
}

#[napi(js_name = "fontVariation")]
pub fn js_font_variation(font_id: BigInt, index: u32) -> Result<Option<FontVariation>> {
    let axes = check(font_variations(ent(font_id)))?;
    Ok(axes.get(index as usize).map(|axis| FontVariation {
        tag: axis.tag.to_string(),
        min: axis.min as f64,
        max: axis.max as f64,
        default: axis.default as f64,
    }))
}

/// Pass null to reset to the default font.
#[napi(js_name = "textFont")]
pub fn js_text_font(graphics_id: BigInt, font_id: Option<BigInt>) -> Result<()> {
    check(graphics_text_font(ent(graphics_id), font_id.map(ent)))
}

fn text_command(content: String, x: f64, y: f64, z: f64, max: Option<(f64, f64)>) -> DrawCommand {
    DrawCommand::Text {
        content,
        x: x as f32,
        y: y as f32,
        z: z as f32,
        max_w: max.map(|(w, _)| w as f32),
        max_h: max.map(|(_, h)| h as f32),
    }
}

#[napi(js_name = "text")]
pub fn js_text(graphics_id: BigInt, content: String, x: f64, y: f64) -> Result<()> {
    record(graphics_id, text_command(content, x, y, 0.0, None))
}

#[napi(js_name = "text3D")]
pub fn js_text_3d(graphics_id: BigInt, content: String, x: f64, y: f64, z: f64) -> Result<()> {
    record(graphics_id, text_command(content, x, y, z, None))
}

#[napi(js_name = "textInt")]
pub fn js_text_int(graphics_id: BigInt, value: i32, x: f64, y: f64) -> Result<()> {
    record(
        graphics_id,
        text_command(value.to_string(), x, y, 0.0, None),
    )
}

#[napi(js_name = "textFloat")]
pub fn js_text_float(graphics_id: BigInt, value: f64, x: f64, y: f64) -> Result<()> {
    let content = format!("{:.3}", value as f32);
    record(graphics_id, text_command(content, x, y, 0.0, None))
}

#[napi(js_name = "textBox")]
pub fn js_text_box(
    graphics_id: BigInt,
    content: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) -> Result<()> {
    record(graphics_id, text_command(content, x, y, 0.0, Some((w, h))))
}

#[napi(js_name = "textStyle")]
pub fn js_text_style(graphics_id: BigInt, style: u8) -> Result<()> {
    check(graphics_text_style(ent(graphics_id), style))
}

/// Returns [x, y, w, h].
#[napi(js_name = "textBounds")]
pub fn js_text_bounds(
    graphics_id: BigInt,
    content: String,
    x: f64,
    y: f64,
) -> Result<Float32Array> {
    let bounds = check(graphics_text_bounds(
        ent(graphics_id),
        &content,
        x as f32,
        y as f32,
        None,
        None,
    ))?;
    Ok(bounds.to_vec().into())
}

#[napi(js_name = "textVariation")]
pub fn js_text_variation(graphics_id: BigInt, tag: String, value: f64) -> Result<()> {
    check(graphics_text_variation(
        ent(graphics_id),
        &tag,
        value as f32,
    ))
}

#[napi(js_name = "clearTextVariations")]
pub fn js_clear_text_variations(graphics_id: BigInt) -> Result<()> {
    check(graphics_clear_text_variations(ent(graphics_id)))
}

#[napi(js_name = "textFeature")]
pub fn js_text_feature(graphics_id: BigInt, tag: String, value: u16) -> Result<()> {
    check(graphics_text_feature(ent(graphics_id), &tag, value))
}

#[napi(js_name = "noTextFeature")]
pub fn js_no_text_feature(graphics_id: BigInt, tag: String) -> Result<()> {
    check(graphics_no_text_feature(ent(graphics_id), &tag))
}

#[napi(js_name = "clearTextFeatures")]
pub fn js_clear_text_features(graphics_id: BigInt) -> Result<()> {
    check(graphics_clear_text_features(ent(graphics_id)))
}

/// `colors` is packed sRGB (r, g, b, a) per glyph.
#[napi(js_name = "textGlyphColors")]
pub fn js_text_glyph_colors(graphics_id: BigInt, colors: Float32Array) -> Result<()> {
    let colors = colors
        .as_chunks::<4>().0.iter()
        .map(|c| bevy::color::Color::srgba(c[0], c[1], c[2], c[3]))
        .collect();
    check(graphics_text_glyph_colors(ent(graphics_id), colors))
}

#[napi(js_name = "textWeight")]
pub fn js_text_weight(graphics_id: BigInt, weight: f64) -> Result<()> {
    check(graphics_text_weight(ent(graphics_id), weight as f32))
}

#[napi(js_name = "textSize")]
pub fn js_text_size(graphics_id: BigInt, size: f64) -> Result<()> {
    record(graphics_id, DrawCommand::TextSize(size as f32))
}

#[napi(js_name = "textAlign")]
pub fn js_text_align(graphics_id: BigInt, h: u8, v: u8) -> Result<()> {
    check(graphics_text_align(ent(graphics_id), h, v))
}

#[napi(js_name = "textLeading")]
pub fn js_text_leading(graphics_id: BigInt, leading: f64) -> Result<()> {
    record(graphics_id, DrawCommand::TextLeading(leading as f32))
}

#[napi(js_name = "textWrap")]
pub fn js_text_wrap(graphics_id: BigInt, mode: u8) -> Result<()> {
    check(graphics_text_wrap(ent(graphics_id), mode))
}

#[napi(js_name = "textWidth")]
pub fn js_text_width(graphics_id: BigInt, content: String) -> Result<f64> {
    check(graphics_text_width(ent(graphics_id), &content)).map(f64::from)
}

#[napi(js_name = "textAscent")]
pub fn js_text_ascent(graphics_id: BigInt) -> Result<f64> {
    check(graphics_text_ascent(ent(graphics_id))).map(f64::from)
}

#[napi(js_name = "textDescent")]
pub fn js_text_descent(graphics_id: BigInt) -> Result<f64> {
    check(graphics_text_descent(ent(graphics_id))).map(f64::from)
}

fn extent(width: u32, height: u32) -> Extent3d {
    Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    }
}

/// `data` is RGBA8 sRGB.
#[napi(js_name = "imageCreate")]
pub fn js_image_create(width: u32, height: u32, data: Uint8Array) -> Result<BigInt> {
    check(image_create(
        extent(width, height),
        data.to_vec(),
        TextureFormat::Rgba8UnormSrgb,
    ))
    .map(id)
}

/// `floats` is linear RGBA; stored as Rgba16Float.
#[napi(js_name = "imageCreateHdr")]
pub fn js_image_create_hdr(width: u32, height: u32, floats: Float32Array) -> Result<BigInt> {
    let mut packed = Vec::with_capacity(floats.len() * 2);
    for &f in floats.iter() {
        packed.extend_from_slice(&half::f16::from_f32(f).to_le_bytes());
    }
    check(image_create(
        extent(width, height),
        packed,
        TextureFormat::Rgba16Float,
    ))
    .map(id)
}

#[napi(js_name = "imageLoad")]
pub fn js_image_load(path: String) -> Result<BigInt> {
    check(image_load(&path)).map(id)
}

#[napi(js_name = "imageResize")]
pub fn js_image_resize(image_id: BigInt, new_width: u32, new_height: u32) -> Result<()> {
    check(image_resize(ent(image_id), extent(new_width, new_height)))
}

/// Returns packed linear RGBA.
#[napi(js_name = "imageReadback")]
pub fn js_image_readback(image_id: BigInt) -> Result<Float32Array> {
    check(image_readback(ent(image_id))).map(|c| linear_to_floats(&c))
}

/// Returns packed linear RGBA.
#[napi(js_name = "graphicsReadback")]
pub fn js_graphics_readback(graphics_id: BigInt) -> Result<Float32Array> {
    check(graphics_readback(ent(graphics_id))).map(|c| linear_to_floats(&c))
}

/// `pixels` is packed RGBA in `space`.
#[napi(js_name = "graphicsUpdate")]
pub fn js_graphics_update(graphics_id: BigInt, pixels: Float32Array, space: u8) -> Result<()> {
    check(graphics_update(
        ent(graphics_id),
        &floats_to_linear(&pixels, space),
    ))
}

/// `pixels` is packed RGBA in `space`.
#[napi(js_name = "graphicsUpdateRegion")]
pub fn js_graphics_update_region(
    graphics_id: BigInt,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    pixels: Float32Array,
    space: u8,
) -> Result<()> {
    check(graphics_update_region(
        ent(graphics_id),
        x,
        y,
        width,
        height,
        &floats_to_linear(&pixels, space),
    ))
}

#[napi(js_name = "graphicsSet")]
pub fn js_graphics_set(
    graphics_id: BigInt,
    x: u32,
    y: u32,
    c1: f64,
    c2: f64,
    c3: f64,
    a: f64,
    space: u8,
) -> Result<()> {
    let color = Color::new(c1, c2, c3, a, space).to_linear();
    check(graphics_update_region(
        ent(graphics_id),
        x,
        y,
        1,
        1,
        &[color],
    ))
}

#[napi(js_name = "tint")]
pub fn js_tint(graphics_id: BigInt, c1: f64, c2: f64, c3: f64, a: f64, space: u8) -> Result<()> {
    let graphics = ent(graphics_id);
    let color = Color::new(c1, c2, c3, a, space);
    check((|| {
        let mode = graphics_get_color_mode(graphics)?;
        graphics_record_command(graphics, DrawCommand::Tint(color.resolve(&mode)))
    })())
}

#[napi(js_name = "noTint")]
pub fn js_no_tint(graphics_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::NoTint)
}

#[napi(js_name = "imageMode")]
pub fn js_image_mode(graphics_id: BigInt, mode: u8) -> Result<()> {
    record(graphics_id, DrawCommand::ImageMode(ShapeMode::from(mode)))
}

#[napi(js_name = "image")]
pub fn js_image(graphics_id: BigInt, image_id: BigInt, dx: f64, dy: f64) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Image {
            entity: ent(image_id),
            dx: dx as f32,
            dy: dy as f32,
            d_width: None,
            d_height: None,
            sx: None,
            sy: None,
            s_width: None,
            s_height: None,
        },
    )
}

#[napi(js_name = "imageScaled")]
pub fn js_image_scaled(
    graphics_id: BigInt,
    image_id: BigInt,
    dx: f64,
    dy: f64,
    d_width: f64,
    d_height: f64,
) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Image {
            entity: ent(image_id),
            dx: dx as f32,
            dy: dy as f32,
            d_width: Some(d_width as f32),
            d_height: Some(d_height as f32),
            sx: None,
            sy: None,
            s_width: None,
            s_height: None,
        },
    )
}

#[napi(js_name = "imageRegion")]
pub fn js_image_region(
    graphics_id: BigInt,
    image_id: BigInt,
    dx: f64,
    dy: f64,
    d_width: f64,
    d_height: f64,
    sx: f64,
    sy: f64,
    s_width: f64,
    s_height: f64,
) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Image {
            entity: ent(image_id),
            dx: dx as f32,
            dy: dy as f32,
            d_width: Some(d_width as f32),
            d_height: Some(d_height as f32),
            sx: Some(sx as f32),
            sy: Some(sy as f32),
            s_width: Some(s_width as f32),
            s_height: Some(s_height as f32),
        },
    )
}

#[napi(js_name = "mode3D")]
pub fn js_mode_3d(graphics_id: BigInt) -> Result<()> {
    check(graphics_mode_3d(ent(graphics_id)))
}

#[napi(js_name = "mode2D")]
pub fn js_mode_2d(graphics_id: BigInt) -> Result<()> {
    check(graphics_mode_2d(ent(graphics_id)))
}

#[napi(js_name = "perspective")]
pub fn js_perspective(
    graphics_id: BigInt,
    fov: f64,
    aspect: f64,
    near: f64,
    far: f64,
) -> Result<()> {
    check(graphics_perspective(
        ent(graphics_id),
        fov as f32,
        aspect as f32,
        near as f32,
        far as f32,
        Vec4::new(0.0, 0.0, -1.0, -near as f32),
    ))
}

#[napi(js_name = "ortho")]
pub fn js_ortho(
    graphics_id: BigInt,
    left: f64,
    right: f64,
    bottom: f64,
    top: f64,
    near: f64,
    far: f64,
) -> Result<()> {
    check(graphics_ortho(
        ent(graphics_id),
        left as f32,
        right as f32,
        bottom as f32,
        top as f32,
        near as f32,
        far as f32,
    ))
}

fn vec3(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3::new(x as f32, y as f32, z as f32)
}

#[napi(js_name = "transformSetPosition")]
pub fn js_transform_set_position(entity_id: BigInt, x: f64, y: f64, z: f64) -> Result<()> {
    check(transform_set_position(ent(entity_id), vec3(x, y, z)))
}

#[napi(js_name = "transformTranslate")]
pub fn js_transform_translate(entity_id: BigInt, x: f64, y: f64, z: f64) -> Result<()> {
    check(transform_translate(ent(entity_id), vec3(x, y, z)))
}

/// Euler angles in radians.
#[napi(js_name = "transformSetRotation")]
pub fn js_transform_set_rotation(entity_id: BigInt, x: f64, y: f64, z: f64) -> Result<()> {
    check(transform_set_rotation(ent(entity_id), vec3(x, y, z)))
}

#[napi(js_name = "transformRotateX")]
pub fn js_transform_rotate_x(entity_id: BigInt, angle: f64) -> Result<()> {
    check(transform_rotate_x(ent(entity_id), angle as f32))
}

#[napi(js_name = "transformRotateY")]
pub fn js_transform_rotate_y(entity_id: BigInt, angle: f64) -> Result<()> {
    check(transform_rotate_y(ent(entity_id), angle as f32))
}

#[napi(js_name = "transformRotateZ")]
pub fn js_transform_rotate_z(entity_id: BigInt, angle: f64) -> Result<()> {
    check(transform_rotate_z(ent(entity_id), angle as f32))
}

#[napi(js_name = "transformRotateAxis")]
pub fn js_transform_rotate_axis(
    entity_id: BigInt,
    angle: f64,
    axis_x: f64,
    axis_y: f64,
    axis_z: f64,
) -> Result<()> {
    check(transform_rotate_axis(
        ent(entity_id),
        angle as f32,
        vec3(axis_x, axis_y, axis_z),
    ))
}

#[napi(js_name = "transformSetScale")]
pub fn js_transform_set_scale(entity_id: BigInt, x: f64, y: f64, z: f64) -> Result<()> {
    check(transform_set_scale(ent(entity_id), vec3(x, y, z)))
}

#[napi(js_name = "transformScale")]
pub fn js_transform_scale(entity_id: BigInt, x: f64, y: f64, z: f64) -> Result<()> {
    check(transform_scale(ent(entity_id), vec3(x, y, z)))
}

#[napi(js_name = "transformLookAt")]
pub fn js_transform_look_at(
    entity_id: BigInt,
    target_x: f64,
    target_y: f64,
    target_z: f64,
) -> Result<()> {
    check(transform_look_at(
        ent(entity_id),
        vec3(target_x, target_y, target_z),
    ))
}

#[napi(js_name = "transformReset")]
pub fn js_transform_reset(entity_id: BigInt) -> Result<()> {
    check(transform_reset(ent(entity_id)))
}

#[napi(js_name = "orbitCamera")]
pub fn js_orbit_camera(graphics_id: BigInt) -> Result<()> {
    check(graphics_orbit_camera(ent(graphics_id)))
}

#[napi(js_name = "freeCamera")]
pub fn js_free_camera(graphics_id: BigInt) -> Result<()> {
    check(graphics_free_camera(ent(graphics_id)))
}

#[napi(js_name = "panCamera")]
pub fn js_pan_camera(graphics_id: BigInt) -> Result<()> {
    check(graphics_pan_camera(ent(graphics_id)))
}

#[napi(js_name = "disableCameraController")]
pub fn js_disable_camera_controller(graphics_id: BigInt) -> Result<()> {
    check(graphics_disable_camera_controller(ent(graphics_id)))
}

#[napi(js_name = "cameraSetDistance")]
pub fn js_camera_set_distance(graphics_id: BigInt, distance: f64) -> Result<()> {
    check(camera_set_distance(ent(graphics_id), distance as f32))
}

#[napi(js_name = "cameraSetCenter")]
pub fn js_camera_set_center(graphics_id: BigInt, x: f64, y: f64, z: f64) -> Result<()> {
    check(camera_set_center(ent(graphics_id), vec3(x, y, z)))
}

#[napi(js_name = "cameraSetMinDistance")]
pub fn js_camera_set_min_distance(graphics_id: BigInt, min: f64) -> Result<()> {
    check(camera_set_min_distance(ent(graphics_id), min as f32))
}

#[napi(js_name = "cameraSetMaxDistance")]
pub fn js_camera_set_max_distance(graphics_id: BigInt, max: f64) -> Result<()> {
    check(camera_set_max_distance(ent(graphics_id), max as f32))
}

#[napi(js_name = "cameraSetSpeed")]
pub fn js_camera_set_speed(graphics_id: BigInt, speed: f64) -> Result<()> {
    check(camera_set_speed(ent(graphics_id), speed as f32))
}

#[napi(js_name = "cameraReset")]
pub fn js_camera_reset(graphics_id: BigInt) -> Result<()> {
    check(camera_reset(ent(graphics_id)))
}

/// An active camera controller overrides this each frame.
#[napi(js_name = "camera")]
pub fn js_camera(
    graphics_id: BigInt,
    eye_x: f64,
    eye_y: f64,
    eye_z: f64,
    center_x: f64,
    center_y: f64,
    center_z: f64,
    up_x: f64,
    up_y: f64,
    up_z: f64,
) -> Result<()> {
    check(graphics_camera(
        ent(graphics_id),
        vec3(eye_x, eye_y, eye_z),
        vec3(center_x, center_y, center_z),
        vec3(up_x, up_y, up_z),
    ))
}

/// `matrix` is a column-major 4x4.
#[napi(js_name = "applyMatrix")]
pub fn js_apply_matrix(graphics_id: BigInt, matrix: Float32Array) -> Result<()> {
    let cols: [f32; 16] = matrix
        .as_ref()
        .try_into()
        .map_err(|_| Error::from_reason("applyMatrix expects 16 floats"))?;
    let affine = Affine3A::from_mat4(Mat4::from_cols_array(&cols));
    record(graphics_id, DrawCommand::ApplyMatrix(affine))
}

/// Column-major 4x4. Flushes pending draws.
#[napi(js_name = "getMatrix")]
pub fn js_get_matrix(graphics_id: BigInt) -> Result<Float32Array> {
    check(graphics_get_matrix(ent(graphics_id))).map(|m| m.to_cols_array().to_vec().into())
}

#[napi(js_name = "modelX")]
pub fn js_model_x(graphics_id: BigInt, x: f64, y: f64, z: f64) -> Result<f64> {
    check(graphics_model_point(ent(graphics_id), vec3(x, y, z))).map(|p| p.x as f64)
}

#[napi(js_name = "modelY")]
pub fn js_model_y(graphics_id: BigInt, x: f64, y: f64, z: f64) -> Result<f64> {
    check(graphics_model_point(ent(graphics_id), vec3(x, y, z))).map(|p| p.y as f64)
}

#[napi(js_name = "modelZ")]
pub fn js_model_z(graphics_id: BigInt, x: f64, y: f64, z: f64) -> Result<f64> {
    check(graphics_model_point(ent(graphics_id), vec3(x, y, z))).map(|p| p.z as f64)
}

#[napi(js_name = "screenX")]
pub fn js_screen_x(graphics_id: BigInt, x: f64, y: f64, z: f64) -> Result<f64> {
    check(graphics_screen_point(ent(graphics_id), vec3(x, y, z))).map(|p| p.x as f64)
}

#[napi(js_name = "screenY")]
pub fn js_screen_y(graphics_id: BigInt, x: f64, y: f64, z: f64) -> Result<f64> {
    check(graphics_screen_point(ent(graphics_id), vec3(x, y, z))).map(|p| p.y as f64)
}

#[napi(js_name = "screenZ")]
pub fn js_screen_z(graphics_id: BigInt, x: f64, y: f64, z: f64) -> Result<f64> {
    check(graphics_screen_point(ent(graphics_id), vec3(x, y, z))).map(|p| p.z as f64)
}

#[napi]
pub const PROCESSING_ATTR_FORMAT_FLOAT: u8 = 1;
#[napi]
pub const PROCESSING_ATTR_FORMAT_FLOAT2: u8 = 2;
#[napi]
pub const PROCESSING_ATTR_FORMAT_FLOAT3: u8 = 3;
#[napi]
pub const PROCESSING_ATTR_FORMAT_FLOAT4: u8 = 4;

#[napi]
pub const PROCESSING_TOPOLOGY_POINT_LIST: u8 = 0;
#[napi]
pub const PROCESSING_TOPOLOGY_LINE_LIST: u8 = 1;
#[napi]
pub const PROCESSING_TOPOLOGY_LINE_STRIP: u8 = 2;
#[napi]
pub const PROCESSING_TOPOLOGY_TRIANGLE_LIST: u8 = 3;
#[napi]
pub const PROCESSING_TOPOLOGY_TRIANGLE_STRIP: u8 = 4;

#[napi]
pub const PROCESSING_STROKE_CAP_ROUND: u8 = 0;
#[napi]
pub const PROCESSING_STROKE_CAP_SQUARE: u8 = 1;
#[napi]
pub const PROCESSING_STROKE_CAP_PROJECT: u8 = 2;

#[napi]
pub const PROCESSING_STROKE_JOIN_ROUND: u8 = 0;
#[napi]
pub const PROCESSING_STROKE_JOIN_MITER: u8 = 1;
#[napi]
pub const PROCESSING_STROKE_JOIN_BEVEL: u8 = 2;

#[napi]
pub const PROCESSING_BLEND_MODE_BLEND: u8 = 0;
#[napi]
pub const PROCESSING_BLEND_MODE_ADD: u8 = 1;
#[napi]
pub const PROCESSING_BLEND_MODE_SUBTRACT: u8 = 2;
#[napi]
pub const PROCESSING_BLEND_MODE_DARKEST: u8 = 3;
#[napi]
pub const PROCESSING_BLEND_MODE_LIGHTEST: u8 = 4;
#[napi]
pub const PROCESSING_BLEND_MODE_DIFFERENCE: u8 = 5;
#[napi]
pub const PROCESSING_BLEND_MODE_EXCLUSION: u8 = 6;
#[napi]
pub const PROCESSING_BLEND_MODE_MULTIPLY: u8 = 7;
#[napi]
pub const PROCESSING_BLEND_MODE_SCREEN: u8 = 8;
#[napi]
pub const PROCESSING_BLEND_MODE_REPLACE: u8 = 9;

#[napi]
pub const PROCESSING_BLEND_FACTOR_ZERO: u8 = 0;
#[napi]
pub const PROCESSING_BLEND_FACTOR_ONE: u8 = 1;
#[napi]
pub const PROCESSING_BLEND_FACTOR_SRC: u8 = 2;
#[napi]
pub const PROCESSING_BLEND_FACTOR_ONE_MINUS_SRC: u8 = 3;
#[napi]
pub const PROCESSING_BLEND_FACTOR_SRC_ALPHA: u8 = 4;
#[napi]
pub const PROCESSING_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA: u8 = 5;
#[napi]
pub const PROCESSING_BLEND_FACTOR_DST: u8 = 6;
#[napi]
pub const PROCESSING_BLEND_FACTOR_ONE_MINUS_DST: u8 = 7;
#[napi]
pub const PROCESSING_BLEND_FACTOR_DST_ALPHA: u8 = 8;
#[napi]
pub const PROCESSING_BLEND_FACTOR_ONE_MINUS_DST_ALPHA: u8 = 9;
#[napi]
pub const PROCESSING_BLEND_FACTOR_SRC_ALPHA_SATURATED: u8 = 10;

#[napi]
pub const PROCESSING_BLEND_OP_ADD: u8 = 0;
#[napi]
pub const PROCESSING_BLEND_OP_SUBTRACT: u8 = 1;
#[napi]
pub const PROCESSING_BLEND_OP_REVERSE_SUBTRACT: u8 = 2;
#[napi]
pub const PROCESSING_BLEND_OP_MIN: u8 = 3;
#[napi]
pub const PROCESSING_BLEND_OP_MAX: u8 = 4;

#[napi(js_name = "geometryLayoutCreate")]
pub fn js_geometry_layout_create() -> Result<BigInt> {
    check(geometry_layout_create()).map(id)
}

#[napi(js_name = "geometryLayoutAddPosition")]
pub fn js_geometry_layout_add_position(layout_id: BigInt) -> Result<()> {
    check(geometry_layout_add_position(ent(layout_id)))
}

#[napi(js_name = "geometryLayoutAddNormal")]
pub fn js_geometry_layout_add_normal(layout_id: BigInt) -> Result<()> {
    check(geometry_layout_add_normal(ent(layout_id)))
}

#[napi(js_name = "geometryLayoutAddColor")]
pub fn js_geometry_layout_add_color(layout_id: BigInt) -> Result<()> {
    check(geometry_layout_add_color(ent(layout_id)))
}

#[napi(js_name = "geometryLayoutAddUv")]
pub fn js_geometry_layout_add_uv(layout_id: BigInt) -> Result<()> {
    check(geometry_layout_add_uv(ent(layout_id)))
}

#[napi(js_name = "geometryLayoutAddAttribute")]
pub fn js_geometry_layout_add_attribute(layout_id: BigInt, attr_id: BigInt) -> Result<()> {
    check(geometry_layout_add_attribute(ent(layout_id), ent(attr_id)))
}

#[napi(js_name = "geometryLayoutDestroy")]
pub fn js_geometry_layout_destroy(layout_id: BigInt) -> Result<()> {
    check(geometry_layout_destroy(ent(layout_id)))
}

fn topology(value: u8) -> Result<geometry::Topology> {
    geometry::Topology::from_u8(value).ok_or_else(|| Error::from_reason("Invalid topology"))
}

#[napi(js_name = "geometryCreateWithLayout")]
pub fn js_geometry_create_with_layout(layout_id: BigInt, topology_id: u8) -> Result<BigInt> {
    let topo = topology(topology_id)?;
    check(geometry_create_with_layout(ent(layout_id), topo)).map(id)
}

#[napi(js_name = "geometryCreate")]
pub fn js_geometry_create(topology_id: u8) -> Result<BigInt> {
    let topo = topology(topology_id)?;
    check(geometry_create(topo)).map(id)
}

#[napi(js_name = "geometryNormal")]
pub fn js_geometry_normal(geo_id: BigInt, nx: f64, ny: f64, nz: f64) -> Result<()> {
    check(geometry_normal(ent(geo_id), vec3(nx, ny, nz)))
}

#[napi(js_name = "geometryColor")]
pub fn js_geometry_color(geo_id: BigInt, r: f64, g: f64, b: f64, a: f64) -> Result<()> {
    check(geometry_color(
        ent(geo_id),
        Vec4::new(r as f32, g as f32, b as f32, a as f32),
    ))
}

#[napi(js_name = "geometryUv")]
pub fn js_geometry_uv(geo_id: BigInt, u: f64, v: f64) -> Result<()> {
    check(geometry_uv(ent(geo_id), u as f32, v as f32))
}

#[napi(js_name = "geometryAttributeCreate")]
pub fn js_geometry_attribute_create(name: String, format: u8) -> Result<BigInt> {
    let attr_format = geometry::AttributeFormat::from_u8(format)
        .ok_or_else(|| Error::from_reason("Invalid attribute format"))?;
    check(geometry_attribute_create(&name, attr_format)).map(id)
}

#[napi(js_name = "geometryAttributeDestroy")]
pub fn js_geometry_attribute_destroy(attr_id: BigInt) -> Result<()> {
    check(geometry_attribute_destroy(ent(attr_id)))
}

#[napi(js_name = "geometryAttributePosition")]
pub fn js_geometry_attribute_position() -> BigInt {
    id(geometry_attribute_position())
}

#[napi(js_name = "geometryAttributeNormal")]
pub fn js_geometry_attribute_normal() -> BigInt {
    id(geometry_attribute_normal())
}

#[napi(js_name = "geometryAttributeColor")]
pub fn js_geometry_attribute_color() -> BigInt {
    id(geometry_attribute_color())
}

#[napi(js_name = "geometryAttributeUv")]
pub fn js_geometry_attribute_uv() -> BigInt {
    id(geometry_attribute_uv())
}

#[napi(js_name = "geometryAttributeRotation")]
pub fn js_geometry_attribute_rotation() -> BigInt {
    id(geometry_attribute_rotation())
}

#[napi(js_name = "geometryAttributeScale")]
pub fn js_geometry_attribute_scale() -> BigInt {
    id(geometry_attribute_scale())
}

#[napi(js_name = "geometryAttributeLife")]
pub fn js_geometry_attribute_life() -> BigInt {
    id(geometry_attribute_life())
}

#[napi(js_name = "geometryAttributeVelocity")]
pub fn js_geometry_attribute_velocity() -> BigInt {
    id(geometry_attribute_velocity())
}

#[napi(js_name = "geometryAttributeAge")]
pub fn js_geometry_attribute_age() -> BigInt {
    id(geometry_attribute_age())
}

/// Returns a PROCESSING_ATTR_FORMAT_* value.
#[napi(js_name = "geometryAttributeFormat")]
pub fn js_geometry_attribute_format(attr_id: BigInt) -> Result<u8> {
    let (_name, fmt) = check(geometry_attribute_info(ent(attr_id)))?;
    Ok(match fmt {
        geometry::AttributeFormat::Float => PROCESSING_ATTR_FORMAT_FLOAT,
        geometry::AttributeFormat::Float2 => PROCESSING_ATTR_FORMAT_FLOAT2,
        geometry::AttributeFormat::Float3 => PROCESSING_ATTR_FORMAT_FLOAT3,
        geometry::AttributeFormat::Float4 => PROCESSING_ATTR_FORMAT_FLOAT4,
    })
}

#[napi(js_name = "geometryAttributeName")]
pub fn js_geometry_attribute_name(attr_id: BigInt) -> Result<String> {
    check(geometry_attribute_info(ent(attr_id))).map(|(name, _)| name.to_string())
}

#[napi(js_name = "geometryAttributeFloat")]
pub fn js_geometry_attribute_float(geo_id: BigInt, attr_id: BigInt, v: f64) -> Result<()> {
    check(geometry_attribute_float(
        ent(geo_id),
        ent(attr_id),
        v as f32,
    ))
}

#[napi(js_name = "geometryAttributeFloat2")]
pub fn js_geometry_attribute_float2(geo_id: BigInt, attr_id: BigInt, x: f64, y: f64) -> Result<()> {
    check(geometry_attribute_float2(
        ent(geo_id),
        ent(attr_id),
        x as f32,
        y as f32,
    ))
}

#[napi(js_name = "geometryAttributeFloat3")]
pub fn js_geometry_attribute_float3(
    geo_id: BigInt,
    attr_id: BigInt,
    x: f64,
    y: f64,
    z: f64,
) -> Result<()> {
    check(geometry_attribute_float3(
        ent(geo_id),
        ent(attr_id),
        x as f32,
        y as f32,
        z as f32,
    ))
}

#[napi(js_name = "geometryAttributeFloat4")]
pub fn js_geometry_attribute_float4(
    geo_id: BigInt,
    attr_id: BigInt,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
) -> Result<()> {
    check(geometry_attribute_float4(
        ent(geo_id),
        ent(attr_id),
        x as f32,
        y as f32,
        z as f32,
        w as f32,
    ))
}

#[napi(js_name = "geometryVertex")]
pub fn js_geometry_vertex(geo_id: BigInt, x: f64, y: f64, z: f64) -> Result<()> {
    check(geometry_vertex(ent(geo_id), vec3(x, y, z)))
}

#[napi(js_name = "geometryIndex")]
pub fn js_geometry_index(geo_id: BigInt, i: u32) -> Result<()> {
    check(geometry_index(ent(geo_id), i))
}

#[napi(js_name = "geometryVertexCount")]
pub fn js_geometry_vertex_count(geo_id: BigInt) -> Result<u32> {
    check(geometry_vertex_count(ent(geo_id)))
}

#[napi(js_name = "geometryIndexCount")]
pub fn js_geometry_index_count(geo_id: BigInt) -> Result<u32> {
    check(geometry_index_count(ent(geo_id)))
}

#[napi(js_name = "geometryGetPositions")]
pub fn js_geometry_get_positions(geo_id: BigInt, start: u32, end: u32) -> Result<Float32Array> {
    check(geometry_get_positions(
        ent(geo_id),
        start as usize,
        end as usize,
    ))
    .map(|v| v.into_iter().flatten().collect::<Vec<_>>().into())
}

#[napi(js_name = "geometryGetNormals")]
pub fn js_geometry_get_normals(geo_id: BigInt, start: u32, end: u32) -> Result<Float32Array> {
    check(geometry_get_normals(
        ent(geo_id),
        start as usize,
        end as usize,
    ))
    .map(|v| v.into_iter().flatten().collect::<Vec<_>>().into())
}

#[napi(js_name = "geometryGetColors")]
pub fn js_geometry_get_colors(geo_id: BigInt, start: u32, end: u32) -> Result<Float32Array> {
    check(geometry_get_colors(
        ent(geo_id),
        start as usize,
        end as usize,
    ))
    .map(|v| v.into_iter().flatten().collect::<Vec<_>>().into())
}

#[napi(js_name = "geometryGetUvs")]
pub fn js_geometry_get_uvs(geo_id: BigInt, start: u32, end: u32) -> Result<Float32Array> {
    check(geometry_get_uvs(ent(geo_id), start as usize, end as usize))
        .map(|v| v.into_iter().flatten().collect::<Vec<_>>().into())
}

#[napi(js_name = "geometryGetIndices")]
pub fn js_geometry_get_indices(geo_id: BigInt, start: u32, end: u32) -> Result<Uint32Array> {
    check(geometry_get_indices(
        ent(geo_id),
        start as usize,
        end as usize,
    ))
    .map(Uint32Array::new)
}

#[napi(js_name = "geometrySetVertex")]
pub fn js_geometry_set_vertex(geo_id: BigInt, index: u32, x: f64, y: f64, z: f64) -> Result<()> {
    check(geometry_set_vertex(ent(geo_id), index, vec3(x, y, z)))
}

#[napi(js_name = "geometrySetNormal")]
pub fn js_geometry_set_normal(geo_id: BigInt, index: u32, nx: f64, ny: f64, nz: f64) -> Result<()> {
    check(geometry_set_normal(ent(geo_id), index, vec3(nx, ny, nz)))
}

#[napi(js_name = "geometrySetColor")]
pub fn js_geometry_set_color(
    geo_id: BigInt,
    index: u32,
    r: f64,
    g: f64,
    b: f64,
    a: f64,
) -> Result<()> {
    check(geometry_set_color(
        ent(geo_id),
        index,
        Vec4::new(r as f32, g as f32, b as f32, a as f32),
    ))
}

#[napi(js_name = "geometrySetUv")]
pub fn js_geometry_set_uv(geo_id: BigInt, index: u32, u: f64, v: f64) -> Result<()> {
    check(geometry_set_uv(
        ent(geo_id),
        index,
        Vec2::new(u as f32, v as f32),
    ))
}

#[napi(js_name = "geometryDestroy")]
pub fn js_geometry_destroy(geo_id: BigInt) -> Result<()> {
    check(geometry_destroy(ent(geo_id)))
}

#[napi(js_name = "model")]
pub fn js_model(graphics_id: BigInt, geo_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::Geometry(ent(geo_id)))
}

#[napi(js_name = "geometryBox")]
pub fn js_geometry_box(width: f64, height: f64, depth: f64) -> Result<BigInt> {
    check(geometry_box(width as f32, height as f32, depth as f32)).map(id)
}

#[napi(js_name = "geometrySphere")]
pub fn js_geometry_sphere(radius: f64, sectors: u32, stacks: u32) -> Result<BigInt> {
    check(geometry_sphere(radius as f32, sectors, stacks)).map(id)
}

#[napi(js_name = "lightCreateDirectional")]
pub fn js_light_create_directional(
    graphics_id: BigInt,
    c1: f64,
    c2: f64,
    c3: f64,
    a: f64,
    space: u8,
    illuminance: f64,
) -> Result<BigInt> {
    let graphics = ent(graphics_id);
    let color = Color::new(c1, c2, c3, a, space);
    check((|| {
        let mode = graphics_get_color_mode(graphics)?;
        light_create_directional(graphics, color.resolve(&mode), illuminance as f32)
    })())
    .map(id)
}

#[napi(js_name = "lightCreatePoint")]
pub fn js_light_create_point(
    graphics_id: BigInt,
    c1: f64,
    c2: f64,
    c3: f64,
    a: f64,
    space: u8,
    intensity: f64,
    range: f64,
    radius: f64,
) -> Result<BigInt> {
    let graphics = ent(graphics_id);
    let color = Color::new(c1, c2, c3, a, space);
    check((|| {
        let mode = graphics_get_color_mode(graphics)?;
        light_create_point(
            graphics,
            color.resolve(&mode),
            intensity as f32,
            range as f32,
            radius as f32,
        )
    })())
    .map(id)
}

#[napi(js_name = "lightCreateSpot")]
pub fn js_light_create_spot(
    graphics_id: BigInt,
    c1: f64,
    c2: f64,
    c3: f64,
    a: f64,
    space: u8,
    intensity: f64,
    range: f64,
    radius: f64,
    inner_angle: f64,
    outer_angle: f64,
) -> Result<BigInt> {
    let graphics = ent(graphics_id);
    let color = Color::new(c1, c2, c3, a, space);
    check((|| {
        let mode = graphics_get_color_mode(graphics)?;
        light_create_spot(
            graphics,
            color.resolve(&mode),
            intensity as f32,
            range as f32,
            radius as f32,
            inner_angle as f32,
            outer_angle as f32,
        )
    })())
    .map(id)
}

#[napi(js_name = "materialCreatePbr")]
pub fn js_material_create_pbr() -> Result<BigInt> {
    check(material_create_pbr()).map(id)
}

#[napi(js_name = "materialCreateCustom")]
pub fn js_material_create_custom(shader_id: BigInt) -> Result<BigInt> {
    check(material_create_custom(ent(shader_id))).map(id)
}

#[napi(js_name = "materialSetFloat")]
pub fn js_material_set_float(mat_id: BigInt, name: String, value: f64) -> Result<()> {
    check(material_set(
        ent(mat_id),
        &name,
        ShaderValue::Float(value as f32),
    ))
}

#[napi(js_name = "materialSetFloat4")]
pub fn js_material_set_float4(
    mat_id: BigInt,
    name: String,
    r: f64,
    g: f64,
    b: f64,
    a: f64,
) -> Result<()> {
    check(material_set(
        ent(mat_id),
        &name,
        ShaderValue::Float4([r as f32, g as f32, b as f32, a as f32]),
    ))
}

#[napi(js_name = "materialDestroy")]
pub fn js_material_destroy(mat_id: BigInt) -> Result<()> {
    check(material_destroy(ent(mat_id)))
}

#[napi(js_name = "material")]
pub fn js_material(graphics_id: BigInt, mat_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::Material(ent(mat_id)))
}

#[napi(js_name = "materialSetAlphaMode")]
pub fn js_material_set_alpha_mode(mat_id: BigInt, mode: u8, cutoff: f64) -> Result<()> {
    check(material_set_alpha_mode(ent(mat_id), mode, cutoff as f32))
}

#[napi(js_name = "materialSetDoubleSided")]
pub fn js_material_set_double_sided(mat_id: BigInt, value: bool) -> Result<()> {
    check(material_set_double_sided(ent(mat_id), value))
}

#[napi(js_name = "materialSetUnlit")]
pub fn js_material_set_unlit(mat_id: BigInt, value: bool) -> Result<()> {
    check(material_set_unlit(ent(mat_id), value))
}

#[napi(js_name = "materialSetDepthWrite")]
pub fn js_material_set_depth_write(mat_id: BigInt, value: bool) -> Result<()> {
    check(material_set_depth_write(ent(mat_id), value))
}

#[napi(js_name = "materialSetCustomBlendMode")]
pub fn js_material_set_custom_blend_mode(
    mat_id: BigInt,
    color_src: u8,
    color_dst: u8,
    color_op: u8,
    alpha_src: u8,
    alpha_dst: u8,
    alpha_op: u8,
) -> Result<()> {
    let mat = ent(mat_id);
    check((|| {
        let blend_state = custom_blend_state(
            color_src, color_dst, color_op, alpha_src, alpha_dst, alpha_op,
        )?;
        material_set_custom_blend(mat, blend_state)
    })())
}

#[napi(js_name = "materialSetAlbedoBuffer")]
pub fn js_material_set_albedo_buffer(mat_id: BigInt, buffer_id: BigInt) -> Result<()> {
    check(material_set_albedo_buffer(ent(mat_id), ent(buffer_id)))
}

#[napi(js_name = "materialSetEmissiveBuffer")]
pub fn js_material_set_emissive_buffer(mat_id: BigInt, buffer_id: BigInt) -> Result<()> {
    check(material_set_emissive_buffer(ent(mat_id), ent(buffer_id)))
}

#[napi(js_name = "graphicsApplyFilter")]
pub fn js_graphics_apply_filter(graphics_id: BigInt, filter_id: BigInt) -> Result<()> {
    check(graphics_apply_filter(ent(graphics_id), ent(filter_id)))
}

#[napi(js_name = "filterCreate")]
pub fn js_filter_create(shader_id: BigInt) -> Result<BigInt> {
    check(filter_create(ent(shader_id))).map(id)
}

#[napi(js_name = "filterDestroy")]
pub fn js_filter_destroy(filter_id: BigInt) -> Result<()> {
    check(filter_destroy(ent(filter_id)))
}

#[napi(js_name = "filterSetPasses")]
pub fn js_filter_set_passes(filter_id: BigInt, passes: u32) -> Result<()> {
    check(filter_set_passes(ent(filter_id), passes))
}

#[napi(js_name = "filterBlur")]
pub fn js_filter_blur() -> Result<BigInt> {
    check(filter_blur()).map(id)
}

#[napi(js_name = "filterInvert")]
pub fn js_filter_invert() -> Result<BigInt> {
    check(filter_invert()).map(id)
}

#[napi(js_name = "filterGray")]
pub fn js_filter_gray() -> Result<BigInt> {
    check(filter_gray()).map(id)
}

#[napi(js_name = "filterThreshold")]
pub fn js_filter_threshold() -> Result<BigInt> {
    check(filter_threshold()).map(id)
}

#[napi(js_name = "filterPosterize")]
pub fn js_filter_posterize() -> Result<BigInt> {
    check(filter_posterize()).map(id)
}

#[napi(js_name = "filterOpaque")]
pub fn js_filter_opaque() -> Result<BigInt> {
    check(filter_opaque()).map(id)
}

#[napi(js_name = "filterErode")]
pub fn js_filter_erode() -> Result<BigInt> {
    check(filter_erode()).map(id)
}

#[napi(js_name = "filterDilate")]
pub fn js_filter_dilate() -> Result<BigInt> {
    check(filter_dilate()).map(id)
}

/// WGSL source.
#[napi(js_name = "shaderCreate")]
pub fn js_shader_create(source: String) -> Result<BigInt> {
    check(shader_create(&source)).map(id)
}

#[napi(js_name = "shaderLoad")]
pub fn js_shader_load(path: String) -> Result<BigInt> {
    check(shader_load(&path)).map(id)
}

#[napi(js_name = "shaderDestroy")]
pub fn js_shader_destroy(shader_id: BigInt) -> Result<()> {
    check(shader_destroy(ent(shader_id)))
}

#[napi(js_name = "bufferCreate")]
pub fn js_buffer_create(size: i64) -> Result<BigInt> {
    check(buffer_create(size as u64)).map(id)
}

#[napi(js_name = "bufferCreateWithData")]
pub fn js_buffer_create_with_data(data: Uint8Array) -> Result<BigInt> {
    check(buffer_create_with_data(data.to_vec())).map(id)
}

#[napi(js_name = "bufferWrite")]
pub fn js_buffer_write(buf_id: BigInt, data: Uint8Array) -> Result<()> {
    check(buffer_write(ent(buf_id), data.to_vec()))
}

#[napi(js_name = "bufferSize")]
pub fn js_buffer_size(buf_id: BigInt) -> Result<i64> {
    check(buffer_size(ent(buf_id))).map(|s| s as i64)
}

#[napi(js_name = "bufferRead")]
pub fn js_buffer_read(buf_id: BigInt) -> Result<Uint8Array> {
    check(buffer_read(ent(buf_id))).map(Uint8Array::new)
}

#[napi(js_name = "bufferDestroy")]
pub fn js_buffer_destroy(buf_id: BigInt) -> Result<()> {
    check(buffer_destroy(ent(buf_id)))
}

#[napi(js_name = "computeCreate")]
pub fn js_compute_create(shader_id: BigInt) -> Result<BigInt> {
    check(compute_create(ent(shader_id))).map(id)
}

fn set_shader_value(entity_id: BigInt, name: &str, value: ShaderValue) -> Result<()> {
    check(shader_set(ent(entity_id), name, value))
}

#[napi(js_name = "shaderSetFloat")]
pub fn js_shader_set_float(entity_id: BigInt, name: String, value: f64) -> Result<()> {
    set_shader_value(entity_id, &name, ShaderValue::Float(value as f32))
}

#[napi(js_name = "shaderSetInt")]
pub fn js_shader_set_int(entity_id: BigInt, name: String, value: i32) -> Result<()> {
    set_shader_value(entity_id, &name, ShaderValue::Int(value))
}

#[napi(js_name = "shaderSetUint")]
pub fn js_shader_set_uint(entity_id: BigInt, name: String, value: u32) -> Result<()> {
    set_shader_value(entity_id, &name, ShaderValue::UInt(value))
}

#[napi(js_name = "shaderSetVec2")]
pub fn js_shader_set_vec2(entity_id: BigInt, name: String, x: f64, y: f64) -> Result<()> {
    set_shader_value(entity_id, &name, ShaderValue::Float2([x as f32, y as f32]))
}

#[napi(js_name = "shaderSetVec3")]
pub fn js_shader_set_vec3(entity_id: BigInt, name: String, x: f64, y: f64, z: f64) -> Result<()> {
    set_shader_value(
        entity_id,
        &name,
        ShaderValue::Float3([x as f32, y as f32, z as f32]),
    )
}

#[napi(js_name = "shaderSetVec4")]
pub fn js_shader_set_vec4(
    entity_id: BigInt,
    name: String,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
) -> Result<()> {
    set_shader_value(
        entity_id,
        &name,
        ShaderValue::Float4([x as f32, y as f32, z as f32, w as f32]),
    )
}

#[napi(js_name = "shaderSetIvec2")]
pub fn js_shader_set_ivec2(entity_id: BigInt, name: String, x: i32, y: i32) -> Result<()> {
    set_shader_value(entity_id, &name, ShaderValue::Int2([x, y]))
}

#[napi(js_name = "shaderSetIvec3")]
pub fn js_shader_set_ivec3(entity_id: BigInt, name: String, x: i32, y: i32, z: i32) -> Result<()> {
    set_shader_value(entity_id, &name, ShaderValue::Int3([x, y, z]))
}

#[napi(js_name = "shaderSetIvec4")]
pub fn js_shader_set_ivec4(
    entity_id: BigInt,
    name: String,
    x: i32,
    y: i32,
    z: i32,
    w: i32,
) -> Result<()> {
    set_shader_value(entity_id, &name, ShaderValue::Int4([x, y, z, w]))
}

#[napi(js_name = "shaderSetUvec2")]
pub fn js_shader_set_uvec2(entity_id: BigInt, name: String, x: u32, y: u32) -> Result<()> {
    set_shader_value(entity_id, &name, ShaderValue::UInt2([x, y]))
}

#[napi(js_name = "shaderSetUvec3")]
pub fn js_shader_set_uvec3(entity_id: BigInt, name: String, x: u32, y: u32, z: u32) -> Result<()> {
    set_shader_value(entity_id, &name, ShaderValue::UInt3([x, y, z]))
}

#[napi(js_name = "shaderSetUvec4")]
pub fn js_shader_set_uvec4(
    entity_id: BigInt,
    name: String,
    x: u32,
    y: u32,
    z: u32,
    w: u32,
) -> Result<()> {
    set_shader_value(entity_id, &name, ShaderValue::UInt4([x, y, z, w]))
}

/// `value` is a column-major 4x4.
#[napi(js_name = "shaderSetMat4")]
pub fn js_shader_set_mat4(entity_id: BigInt, name: String, value: Float32Array) -> Result<()> {
    let m: [f32; 16] = value
        .as_ref()
        .try_into()
        .map_err(|_| Error::from_reason("shaderSetMat4 expects 16 floats"))?;
    set_shader_value(entity_id, &name, ShaderValue::Mat4(m))
}

#[napi(js_name = "shaderSetTexture")]
pub fn js_shader_set_texture(entity_id: BigInt, name: String, image_id: BigInt) -> Result<()> {
    set_shader_value(entity_id, &name, ShaderValue::Texture(ent(image_id)))
}

#[napi(js_name = "shaderSetBuffer")]
pub fn js_shader_set_buffer(entity_id: BigInt, name: String, buf_id: BigInt) -> Result<()> {
    set_shader_value(entity_id, &name, ShaderValue::Buffer(ent(buf_id)))
}

/// Binds `{name}_offsets`, `{name}_sorted` and the `{name}` uniform.
#[napi(js_name = "shaderSetGrid")]
pub fn js_shader_set_grid(entity_id: BigInt, name: String, grid_id: BigInt) -> Result<()> {
    set_shader_value(entity_id, &name, ShaderValue::Grid(ent(grid_id)))
}

#[napi(js_name = "computeDispatch")]
pub fn js_compute_dispatch(compute_id: BigInt, x: u32, y: u32, z: u32) -> Result<()> {
    check(compute_dispatch(ent(compute_id), x, y, z))
}

#[napi(js_name = "computeDestroy")]
pub fn js_compute_destroy(compute_id: BigInt) -> Result<()> {
    check(compute_destroy(ent(compute_id)))
}

fn entities(ids: Option<Vec<BigInt>>) -> Vec<Entity> {
    ids.unwrap_or_default().into_iter().map(ent).collect()
}

#[napi(js_name = "particlesCreate")]
pub fn js_particles_create(capacity: u32, attr_ids: Option<Vec<BigInt>>) -> Result<BigInt> {
    check(particles_create(capacity, entities(attr_ids))).map(id)
}

#[napi(js_name = "particlesCreateFromGeometry")]
pub fn js_particles_create_from_geometry(
    geo_id: BigInt,
    attr_ids: Option<Vec<BigInt>>,
) -> Result<BigInt> {
    check(particles_create_from_geometry(
        ent(geo_id),
        entities(attr_ids),
    ))
    .map(id)
}

#[napi(js_name = "particlesDestroy")]
pub fn js_particles_destroy(particles_id: BigInt) -> Result<()> {
    check(particles_destroy(ent(particles_id)))
}

#[napi(js_name = "particlesCapacity")]
pub fn js_particles_capacity(particles_id: BigInt) -> Result<u32> {
    check(particles_capacity(ent(particles_id)))
}

#[napi(js_name = "particlesBuffer")]
pub fn js_particles_buffer(particles_id: BigInt, attr_id: BigInt) -> Result<Option<BigInt>> {
    check(particles_buffer(ent(particles_id), ent(attr_id))).map(|b| b.map(id))
}

/// `attr_ids[i]` is initialized from the raw bytes in `data[i]`.
#[napi(js_name = "particlesEmit")]
pub fn js_particles_emit(
    particles_id: BigInt,
    n: u32,
    attr_ids: Option<Vec<BigInt>>,
    data: Option<Vec<Uint8Array>>,
) -> Result<()> {
    let attr_ids = entities(attr_ids);
    let data = data.unwrap_or_default();
    if attr_ids.len() != data.len() {
        return Err(Error::from_reason(format!(
            "particlesEmit: {} attributes but {} data arrays",
            attr_ids.len(),
            data.len()
        )));
    }
    let attribute_data = attr_ids
        .into_iter()
        .zip(data.iter().map(|d| d.to_vec()))
        .collect();
    check(particles_emit(ent(particles_id), n, attribute_data))
}

#[napi(js_name = "particlesEmitGpu")]
pub fn js_particles_emit_gpu(particles_id: BigInt, n: u32, compute_id: BigInt) -> Result<()> {
    check(particles_emit_gpu(ent(particles_id), n, ent(compute_id)))
}

#[napi(js_name = "particlesAttributeAdd")]
pub fn js_particles_attribute_add(particles_id: BigInt, attribute_id: BigInt) -> Result<()> {
    check(particles_attribute_add(
        ent(particles_id),
        ent(attribute_id),
        None,
    ))
    .map(|_| ())
}

#[napi(js_name = "particlesApply")]
pub fn js_particles_apply(particles_id: BigInt, compute_id: BigInt) -> Result<()> {
    check(particles_apply(ent(particles_id), ent(compute_id)))
}

#[napi(js_name = "particlesKernelNoise")]
pub fn js_particles_kernel_noise() -> Result<BigInt> {
    check(particles_kernel_noise()).map(id)
}

#[napi(js_name = "particlesKernelTransform")]
pub fn js_particles_kernel_transform() -> Result<BigInt> {
    check(particles_kernel_transform()).map(id)
}

#[napi(js_name = "particlesKernelAttract")]
pub fn js_particles_kernel_attract() -> Result<BigInt> {
    check(particles_kernel_attract()).map(id)
}

#[napi(js_name = "particlesKernelDrag")]
pub fn js_particles_kernel_drag() -> Result<BigInt> {
    check(particles_kernel_drag()).map(id)
}

#[napi(js_name = "particlesKernelVortex")]
pub fn js_particles_kernel_vortex() -> Result<BigInt> {
    check(particles_kernel_vortex()).map(id)
}

#[napi(js_name = "particlesKernelForce")]
pub fn js_particles_kernel_force() -> Result<BigInt> {
    check(particles_kernel_force()).map(id)
}

#[napi(js_name = "particlesKernelIntegrate")]
pub fn js_particles_kernel_integrate() -> Result<BigInt> {
    check(particles_kernel_integrate()).map(id)
}

#[napi(js_name = "particlesKernelAge")]
pub fn js_particles_kernel_age() -> Result<BigInt> {
    check(particles_kernel_age()).map(id)
}

#[napi(js_name = "particlesKernelBoundsSphere")]
pub fn js_particles_kernel_bounds_sphere() -> Result<BigInt> {
    check(particles_kernel_bounds_sphere()).map(id)
}

#[napi(js_name = "particlesKernelBoundsBox")]
pub fn js_particles_kernel_bounds_box() -> Result<BigInt> {
    check(particles_kernel_bounds_box()).map(id)
}

#[napi(js_name = "particlesKernelBoundsGeometry")]
pub fn js_particles_kernel_bounds_geometry(geometry_id: BigInt) -> Result<BigInt> {
    check(particles_kernel_bounds_geometry(ent(geometry_id))).map(id)
}

#[napi(js_name = "particlesKernelImpulse")]
pub fn js_particles_kernel_impulse() -> Result<BigInt> {
    check(particles_kernel_impulse()).map(id)
}

#[napi(js_name = "particlesKernelOrient")]
pub fn js_particles_kernel_orient() -> Result<BigInt> {
    check(particles_kernel_orient()).map(id)
}

#[napi(js_name = "particlesKernelField")]
pub fn js_particles_kernel_field() -> Result<BigInt> {
    check(particles_kernel_field()).map(id)
}

#[napi(js_name = "particlesKernelAttrLinear")]
pub fn js_particles_kernel_attr_linear() -> Result<BigInt> {
    check(particles_kernel_attr_linear()).map(id)
}

#[napi(js_name = "particlesKernelAttrCombine")]
pub fn js_particles_kernel_attr_combine() -> Result<BigInt> {
    check(particles_kernel_attr_combine()).map(id)
}

#[napi(js_name = "particlesKernelAttrMix")]
pub fn js_particles_kernel_attr_mix() -> Result<BigInt> {
    check(particles_kernel_attr_mix()).map(id)
}

#[napi(js_name = "particlesKernelAttrLookup1d")]
pub fn js_particles_kernel_attr_lookup1d() -> Result<BigInt> {
    check(particles_kernel_attr_lookup1d()).map(id)
}

#[napi(js_name = "particlesKernelAttrLookup2d")]
pub fn js_particles_kernel_attr_lookup2d() -> Result<BigInt> {
    check(particles_kernel_attr_lookup2d()).map(id)
}

#[napi(js_name = "particlesScatterCreate")]
pub fn js_particles_scatter_create(geometry_id: BigInt) -> Result<BigInt> {
    check(particles_scatter_create(ent(geometry_id))).map(id)
}

#[napi(js_name = "particlesScatterVolumeCreate")]
pub fn js_particles_scatter_volume_create(geometry_id: BigInt) -> Result<BigInt> {
    check(particles_scatter_volume_create(ent(geometry_id))).map(id)
}

#[napi(js_name = "particlesGridCreate")]
pub fn js_particles_grid_create(
    particles_id: BigInt,
    min_x: f64,
    min_y: f64,
    min_z: f64,
    cell_size: f64,
    dims_x: u32,
    dims_y: u32,
    dims_z: u32,
) -> Result<BigInt> {
    let particles = ent(particles_id);
    check((|| {
        let capacity = particles_capacity(particles)?;
        grid_create(
            GridParams {
                min: [min_x as f32, min_y as f32, min_z as f32],
                cell_size: cell_size as f32,
                dims: [dims_x, dims_y, dims_z],
            },
            capacity,
        )
    })())
    .map(id)
}

#[napi(js_name = "particlesGridBuild")]
pub fn js_particles_grid_build(grid_id: BigInt, position_buf_id: BigInt) -> Result<()> {
    check(grid_build(ent(grid_id), ent(position_buf_id)))
}

#[napi(js_name = "particlesGridDestroy")]
pub fn js_particles_grid_destroy(grid_id: BigInt) -> Result<()> {
    check(grid_destroy(ent(grid_id)))
}

/// Writes each particle's `neighbors` and `neighbor_count`, rebuilding the grid first.
#[napi(js_name = "particlesFindNeighbors")]
pub fn js_particles_find_neighbors(
    particles_id: BigInt,
    grid_id: BigInt,
    radius: f64,
    max: u32,
) -> Result<()> {
    check(particles_find_neighbors(
        ent(particles_id),
        ent(grid_id),
        radius as f32,
        max,
    ))
}

#[napi(js_name = "particlesDraw")]
pub fn js_particles_draw(
    graphics_id: BigInt,
    particles_id: BigInt,
    geometry_id: BigInt,
) -> Result<()> {
    record(
        graphics_id,
        DrawCommand::Particles {
            particles: ent(particles_id),
            geometry: Some(ent(geometry_id)),
            topology: geometry::Topology::PointList,
        },
    )
}

#[napi(js_name = "fillBuffer")]
pub fn js_fill_buffer(graphics_id: BigInt, buffer_id: BigInt) -> Result<()> {
    record(graphics_id, DrawCommand::FillBuffer(ent(buffer_id)))
}

#[napi(js_name = "gltfLoad")]
pub fn js_gltf_load(graphics_id: BigInt, path: String) -> Result<BigInt> {
    check(gltf_load(ent(graphics_id), &path)).map(id)
}

#[napi(js_name = "gltfGeometry")]
pub fn js_gltf_geometry(gltf_id: BigInt, name: String) -> Result<BigInt> {
    check(gltf_geometry(ent(gltf_id), &name)).map(id)
}

#[napi(js_name = "gltfMaterial")]
pub fn js_gltf_material(gltf_id: BigInt, name: String) -> Result<BigInt> {
    check(gltf_material(ent(gltf_id), &name)).map(id)
}

#[napi(js_name = "gltfCamera")]
pub fn js_gltf_camera(gltf_id: BigInt, index: u32) -> Result<()> {
    check(gltf_camera(ent(gltf_id), index as usize))
}

#[napi(js_name = "gltfLight")]
pub fn js_gltf_light(gltf_id: BigInt, index: u32) -> Result<BigInt> {
    check(gltf_light(ent(gltf_id), index as usize)).map(id)
}

/// Returns [x, y, z].
#[napi(js_name = "graphicsWorldFromScreen")]
pub fn js_graphics_world_from_screen(
    graphics_id: BigInt,
    sx: f64,
    sy: f64,
    depth: f64,
) -> Result<Float32Array> {
    check(graphics_world_from_screen(
        ent(graphics_id),
        sx as f32,
        sy as f32,
        depth as f32,
    ))
    .map(|p| p.to_array().to_vec().into())
}

#[napi(js_name = "graphicsSetBloom")]
pub fn js_graphics_set_bloom(graphics_id: BigInt, intensity: f64, threshold: f64) -> Result<()> {
    check(graphics_bloom(
        ent(graphics_id),
        intensity as f32,
        threshold as f32,
    ))
}

#[napi(js_name = "graphicsRemoveBloom")]
pub fn js_graphics_remove_bloom(graphics_id: BigInt) -> Result<()> {
    check(graphics_no_bloom(ent(graphics_id)))
}

#[napi(js_name = "graphicsTonemapping")]
pub fn js_graphics_tonemapping(graphics_id: BigInt, mode: u8) -> Result<()> {
    let graphics = ent(graphics_id);
    check((|| {
        let tonemapping = graphics::tonemapping_from_u8(mode).ok_or_else(|| {
            ProcessingError::InvalidArgument(format!("unknown tonemapping: {mode}"))
        })?;
        graphics_tonemapping(graphics, tonemapping)
    })())
}

#[napi(js_name = "graphicsNoTonemapping")]
pub fn js_graphics_no_tonemapping(graphics_id: BigInt) -> Result<()> {
    check(graphics_no_tonemapping(ent(graphics_id)))
}

#[napi]
pub const PROCESSING_MOUSE_LEFT: u8 = 0;
#[napi]
pub const PROCESSING_MOUSE_MIDDLE: u8 = 1;
#[napi]
pub const PROCESSING_MOUSE_RIGHT: u8 = 2;

// GLFW key code values
#[napi]
pub const PROCESSING_KEY_SPACE: u32 = 32;
#[napi]
pub const PROCESSING_KEY_QUOTE: u32 = 39;
#[napi]
pub const PROCESSING_KEY_COMMA: u32 = 44;
#[napi]
pub const PROCESSING_KEY_MINUS: u32 = 45;
#[napi]
pub const PROCESSING_KEY_PERIOD: u32 = 46;
#[napi]
pub const PROCESSING_KEY_SLASH: u32 = 47;
#[napi]
pub const PROCESSING_KEY_0: u32 = 48;
#[napi]
pub const PROCESSING_KEY_1: u32 = 49;
#[napi]
pub const PROCESSING_KEY_2: u32 = 50;
#[napi]
pub const PROCESSING_KEY_3: u32 = 51;
#[napi]
pub const PROCESSING_KEY_4: u32 = 52;
#[napi]
pub const PROCESSING_KEY_5: u32 = 53;
#[napi]
pub const PROCESSING_KEY_6: u32 = 54;
#[napi]
pub const PROCESSING_KEY_7: u32 = 55;
#[napi]
pub const PROCESSING_KEY_8: u32 = 56;
#[napi]
pub const PROCESSING_KEY_9: u32 = 57;
#[napi]
pub const PROCESSING_KEY_SEMICOLON: u32 = 59;
#[napi]
pub const PROCESSING_KEY_EQUAL: u32 = 61;
#[napi]
pub const PROCESSING_KEY_A: u32 = 65;
#[napi]
pub const PROCESSING_KEY_B: u32 = 66;
#[napi]
pub const PROCESSING_KEY_C: u32 = 67;
#[napi]
pub const PROCESSING_KEY_D: u32 = 68;
#[napi]
pub const PROCESSING_KEY_E: u32 = 69;
#[napi]
pub const PROCESSING_KEY_F: u32 = 70;
#[napi]
pub const PROCESSING_KEY_G: u32 = 71;
#[napi]
pub const PROCESSING_KEY_H: u32 = 72;
#[napi]
pub const PROCESSING_KEY_I: u32 = 73;
#[napi]
pub const PROCESSING_KEY_J: u32 = 74;
#[napi]
pub const PROCESSING_KEY_K: u32 = 75;
#[napi]
pub const PROCESSING_KEY_L: u32 = 76;
#[napi]
pub const PROCESSING_KEY_M: u32 = 77;
#[napi]
pub const PROCESSING_KEY_N: u32 = 78;
#[napi]
pub const PROCESSING_KEY_O: u32 = 79;
#[napi]
pub const PROCESSING_KEY_P: u32 = 80;
#[napi]
pub const PROCESSING_KEY_Q: u32 = 81;
#[napi]
pub const PROCESSING_KEY_R: u32 = 82;
#[napi]
pub const PROCESSING_KEY_S: u32 = 83;
#[napi]
pub const PROCESSING_KEY_T: u32 = 84;
#[napi]
pub const PROCESSING_KEY_U: u32 = 85;
#[napi]
pub const PROCESSING_KEY_V: u32 = 86;
#[napi]
pub const PROCESSING_KEY_W: u32 = 87;
#[napi]
pub const PROCESSING_KEY_X: u32 = 88;
#[napi]
pub const PROCESSING_KEY_Y: u32 = 89;
#[napi]
pub const PROCESSING_KEY_Z: u32 = 90;
#[napi]
pub const PROCESSING_KEY_BRACKET_LEFT: u32 = 91;
#[napi]
pub const PROCESSING_KEY_BACKSLASH: u32 = 92;
#[napi]
pub const PROCESSING_KEY_BRACKET_RIGHT: u32 = 93;
#[napi]
pub const PROCESSING_KEY_BACKQUOTE: u32 = 96;
#[napi]
pub const PROCESSING_KEY_ESCAPE: u32 = 256;
#[napi]
pub const PROCESSING_KEY_ENTER: u32 = 257;
#[napi]
pub const PROCESSING_KEY_TAB: u32 = 258;
#[napi]
pub const PROCESSING_KEY_BACKSPACE: u32 = 259;
#[napi]
pub const PROCESSING_KEY_INSERT: u32 = 260;
#[napi]
pub const PROCESSING_KEY_DELETE: u32 = 261;
#[napi]
pub const PROCESSING_KEY_RIGHT: u32 = 262;
#[napi]
pub const PROCESSING_KEY_LEFT: u32 = 263;
#[napi]
pub const PROCESSING_KEY_DOWN: u32 = 264;
#[napi]
pub const PROCESSING_KEY_UP: u32 = 265;
#[napi]
pub const PROCESSING_KEY_PAGE_UP: u32 = 266;
#[napi]
pub const PROCESSING_KEY_PAGE_DOWN: u32 = 267;
#[napi]
pub const PROCESSING_KEY_HOME: u32 = 268;
#[napi]
pub const PROCESSING_KEY_END: u32 = 269;
#[napi]
pub const PROCESSING_KEY_CAPS_LOCK: u32 = 280;
#[napi]
pub const PROCESSING_KEY_SCROLL_LOCK: u32 = 281;
#[napi]
pub const PROCESSING_KEY_NUM_LOCK: u32 = 282;
#[napi]
pub const PROCESSING_KEY_PRINT_SCREEN: u32 = 283;
#[napi]
pub const PROCESSING_KEY_PAUSE: u32 = 284;
#[napi]
pub const PROCESSING_KEY_F1: u32 = 290;
#[napi]
pub const PROCESSING_KEY_F2: u32 = 291;
#[napi]
pub const PROCESSING_KEY_F3: u32 = 292;
#[napi]
pub const PROCESSING_KEY_F4: u32 = 293;
#[napi]
pub const PROCESSING_KEY_F5: u32 = 294;
#[napi]
pub const PROCESSING_KEY_F6: u32 = 295;
#[napi]
pub const PROCESSING_KEY_F7: u32 = 296;
#[napi]
pub const PROCESSING_KEY_F8: u32 = 297;
#[napi]
pub const PROCESSING_KEY_F9: u32 = 298;
#[napi]
pub const PROCESSING_KEY_F10: u32 = 299;
#[napi]
pub const PROCESSING_KEY_F11: u32 = 300;
#[napi]
pub const PROCESSING_KEY_F12: u32 = 301;
#[napi]
pub const PROCESSING_KEY_NUMPAD_0: u32 = 320;
#[napi]
pub const PROCESSING_KEY_NUMPAD_1: u32 = 321;
#[napi]
pub const PROCESSING_KEY_NUMPAD_2: u32 = 322;
#[napi]
pub const PROCESSING_KEY_NUMPAD_3: u32 = 323;
#[napi]
pub const PROCESSING_KEY_NUMPAD_4: u32 = 324;
#[napi]
pub const PROCESSING_KEY_NUMPAD_5: u32 = 325;
#[napi]
pub const PROCESSING_KEY_NUMPAD_6: u32 = 326;
#[napi]
pub const PROCESSING_KEY_NUMPAD_7: u32 = 327;
#[napi]
pub const PROCESSING_KEY_NUMPAD_8: u32 = 328;
#[napi]
pub const PROCESSING_KEY_NUMPAD_9: u32 = 329;
#[napi]
pub const PROCESSING_KEY_NUMPAD_DECIMAL: u32 = 330;
#[napi]
pub const PROCESSING_KEY_NUMPAD_DIVIDE: u32 = 331;
#[napi]
pub const PROCESSING_KEY_NUMPAD_MULTIPLY: u32 = 332;
#[napi]
pub const PROCESSING_KEY_NUMPAD_SUBTRACT: u32 = 333;
#[napi]
pub const PROCESSING_KEY_NUMPAD_ADD: u32 = 334;
#[napi]
pub const PROCESSING_KEY_NUMPAD_ENTER: u32 = 335;
#[napi]
pub const PROCESSING_KEY_NUMPAD_EQUAL: u32 = 336;
#[napi]
pub const PROCESSING_KEY_SHIFT_LEFT: u32 = 340;
#[napi]
pub const PROCESSING_KEY_CONTROL_LEFT: u32 = 341;
#[napi]
pub const PROCESSING_KEY_ALT_LEFT: u32 = 342;
#[napi]
pub const PROCESSING_KEY_SUPER_LEFT: u32 = 343;
#[napi]
pub const PROCESSING_KEY_SHIFT_RIGHT: u32 = 344;
#[napi]
pub const PROCESSING_KEY_CONTROL_RIGHT: u32 = 345;
#[napi]
pub const PROCESSING_KEY_ALT_RIGHT: u32 = 346;
#[napi]
pub const PROCESSING_KEY_SUPER_RIGHT: u32 = 347;
#[napi]
pub const PROCESSING_KEY_CONTEXT_MENU: u32 = 348;

#[napi]
pub const PROCESSING_CURSOR_ARROW: u8 = 0;
#[napi]
pub const PROCESSING_CURSOR_CROSS: u8 = 1;
#[napi]
pub const PROCESSING_CURSOR_HAND: u8 = 2;
#[napi]
pub const PROCESSING_CURSOR_MOVE: u8 = 3;
#[napi]
pub const PROCESSING_CURSOR_TEXT: u8 = 4;
#[napi]
pub const PROCESSING_CURSOR_WAIT: u8 = 5;

fn cursor_icon(kind: u8) -> bevy::window::SystemCursorIcon {
    use bevy::window::SystemCursorIcon;
    match kind {
        PROCESSING_CURSOR_CROSS => SystemCursorIcon::Crosshair,
        PROCESSING_CURSOR_HAND => SystemCursorIcon::Pointer,
        PROCESSING_CURSOR_MOVE => SystemCursorIcon::Move,
        PROCESSING_CURSOR_TEXT => SystemCursorIcon::Text,
        PROCESSING_CURSOR_WAIT => SystemCursorIcon::Wait,
        _ => SystemCursorIcon::Default,
    }
}

#[napi(js_name = "inputMouseMove")]
pub fn js_input_mouse_move(surface_id: BigInt, x: f64, y: f64) -> Result<()> {
    check(input_set_mouse_move(ent(surface_id), x as f32, y as f32))
}

#[napi(js_name = "inputMouseButton")]
pub fn js_input_mouse_button(surface_id: BigInt, button: u8, pressed: bool) -> Result<()> {
    let surface = ent(surface_id);
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
        input_set_mouse_button(surface, btn, pressed)
    })())
}

#[napi(js_name = "inputScroll")]
pub fn js_input_scroll(surface_id: BigInt, x: f64, y: f64) -> Result<()> {
    check(input_set_scroll(ent(surface_id), x as f32, y as f32))
}

#[napi(js_name = "inputKey")]
pub fn js_input_key(surface_id: BigInt, key_code: u32, pressed: bool) -> Result<()> {
    let surface = ent(surface_id);
    check((|| {
        let kc = key_code_from_u32(key_code)?;
        input_set_key(surface, kc, pressed)
    })())
}

#[napi(js_name = "inputChar")]
pub fn js_input_char(surface_id: BigInt, key_code: u32, codepoint: u32) -> Result<()> {
    let surface = ent(surface_id);
    check((|| {
        let kc = key_code_from_u32(key_code)?;
        let ch = char::from_u32(codepoint).ok_or_else(|| {
            ProcessingError::InvalidArgument(format!("invalid codepoint: {codepoint}"))
        })?;
        input_set_char(surface, kc, ch)
    })())
}

#[napi(js_name = "inputCursorEnter")]
pub fn js_input_cursor_enter(surface_id: BigInt) -> Result<()> {
    check(input_set_cursor_enter(ent(surface_id)))
}

#[napi(js_name = "inputCursorLeave")]
pub fn js_input_cursor_leave(surface_id: BigInt) -> Result<()> {
    check(input_set_cursor_leave(ent(surface_id)))
}

#[napi(js_name = "inputFocus")]
pub fn js_input_focus(surface_id: BigInt, focused: bool) -> Result<()> {
    check(input_set_focus(ent(surface_id), focused))
}

/// Shows the cursor with the given PROCESSING_CURSOR_* icon.
#[napi(js_name = "cursor")]
pub fn js_cursor(surface_id: BigInt, kind: u8) -> Result<()> {
    let surface = ent(surface_id);
    check(input_set_cursor_visible(surface, true))?;
    check(input_set_cursor_icon(surface, cursor_icon(kind)))
}

#[napi(js_name = "noCursor")]
pub fn js_no_cursor(surface_id: BigInt) -> Result<()> {
    check(input_set_cursor_visible(ent(surface_id), false))
}

#[napi(js_name = "inputFlush")]
pub fn js_input_flush() -> Result<()> {
    check(input_flush())
}

#[napi(js_name = "mouseX")]
pub fn js_mouse_x(surface_id: BigInt) -> Result<f64> {
    check(input_mouse_x(ent(surface_id))).map(f64::from)
}

#[napi(js_name = "mouseY")]
pub fn js_mouse_y(surface_id: BigInt) -> Result<f64> {
    check(input_mouse_y(ent(surface_id))).map(f64::from)
}

#[napi(js_name = "pmouseX")]
pub fn js_pmouse_x(surface_id: BigInt) -> Result<f64> {
    check(input_pmouse_x(ent(surface_id))).map(f64::from)
}

#[napi(js_name = "pmouseY")]
pub fn js_pmouse_y(surface_id: BigInt) -> Result<f64> {
    check(input_pmouse_y(ent(surface_id))).map(f64::from)
}

#[napi(js_name = "mouseIsPressed")]
pub fn js_mouse_is_pressed() -> Result<bool> {
    check(input_mouse_is_pressed())
}

/// Returns a PROCESSING_MOUSE_* value, or -1 if no button is pressed.
#[napi(js_name = "mouseButton")]
pub fn js_mouse_button() -> Result<i32> {
    check(input_mouse_button()).map(|opt| match opt {
        Some(MouseButton::Left) => PROCESSING_MOUSE_LEFT as i32,
        Some(MouseButton::Middle) => PROCESSING_MOUSE_MIDDLE as i32,
        Some(MouseButton::Right) => PROCESSING_MOUSE_RIGHT as i32,
        _ => -1,
    })
}

#[napi(js_name = "keyIsPressed")]
pub fn js_key_is_pressed() -> Result<bool> {
    check(input_key_is_pressed())
}

#[napi(js_name = "keyIsDown")]
pub fn js_key_is_down(key_code: u32) -> Result<bool> {
    check(key_code_from_u32(key_code).and_then(input_key_is_down))
}

#[napi(js_name = "keyJustPressed")]
pub fn js_key_just_pressed(key_code: u32) -> Result<bool> {
    check(key_code_from_u32(key_code).and_then(input_key_just_pressed))
}

/// The last typed character, or null.
#[napi(js_name = "key")]
pub fn js_key() -> Result<Option<String>> {
    check(input_key()).map(|opt| opt.map(|c| c.to_string()))
}

/// Returns a PROCESSING_KEY_* value, or 0 if none.
#[napi(js_name = "keyCode")]
pub fn js_key_code() -> Result<u32> {
    check(input_key_code()).map(|opt| opt.map(key_code_to_u32).unwrap_or(0))
}

#[napi(js_name = "movedX")]
pub fn js_moved_x() -> Result<f64> {
    check(input_moved_x()).map(f64::from)
}

#[napi(js_name = "movedY")]
pub fn js_moved_y() -> Result<f64> {
    check(input_moved_y()).map(f64::from)
}

#[napi(js_name = "mouseWheel")]
pub fn js_mouse_wheel() -> Result<f64> {
    check(input_mouse_wheel()).map(f64::from)
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
