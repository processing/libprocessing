pub mod command;
pub mod filter;
pub mod material;
pub mod mesh_builder;
pub mod primitive;
pub mod style;
pub mod transform;

use bevy::{
    camera::Camera3dDepthLoadOp,
    camera::{
        CameraOutputMode, ClearColorConfig, RenderTarget,
        primitives::Aabb,
        visibility::{NoFrustumCulling, RenderLayers},
    },
    ecs::{query::Has, system::SystemParam},
    math::{Affine2, Affine3A, Vec3A},
    pbr::gpu_instance_batch::GpuBatchedMesh3d,
    prelude::*,
    render::render_resource::BlendState,
    window::WindowRef,
};
use command::{CommandBuffer, DrawCommand, ShapeMode};
use material::{MaterialKey, ProcessingExtendedMaterial};
use primitive::{
    ShapeBuilder, TessellationMode, VertexType, arc_fill, arc_stroke, bezier, box_mesh,
    build_direct_fill, build_direct_stroke, build_polygon_fill, build_polygon_stroke, capsule_mesh,
    cone_mesh, conical_frustum_mesh, curve, cylinder_mesh, ellipse, empty_mesh, line, plane_mesh,
    quad, sphere_mesh, tetrahedron_mesh, torus_mesh, triangle,
};
use style::{Fill, StyleStack};
use transform::TransformStack;

use crate::{
    Flush,
    geometry::Geometry,
    gltf::GltfNodeTransform,
    image::Image,
    light::ProcessingLight,
    material::ProcessingMaterial,
    material::custom::CustomMaterial,
    particles::{Particles, ParticlesDraw},
    render::{material::UntypedMaterial, primitive::rect},
    text::font::TextContext,
};

pub(crate) const BATCH_INDEX_STEP: f32 = 0.001;

/// Despawned at the graphics' next flush.
#[derive(Component)]
#[relationship(relationship_target = FlushTransients)]
pub struct FlushTransient(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = FlushTransient, linked_spawn)]
pub struct FlushTransients(Vec<Entity>);

/// Despawned when the graphics begins its next draw.
#[derive(Component)]
#[relationship(relationship_target = DrawTransients)]
pub struct DrawTransient(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = DrawTransient, linked_spawn)]
pub struct DrawTransients(Vec<Entity>);

#[derive(SystemParam)]
pub struct RenderResources<'w, 's> {
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ProcessingExtendedMaterial>>,
    custom_materials: ResMut<'w, Assets<CustomMaterial>>,
    particles_materials: ResMut<'w, Assets<crate::particles::material::ParticlesMaterial>>,
    particle_buffers: Query<'w, 's, &'static crate::compute::Buffer>,
}

struct BatchState {
    current_mesh: Option<Mesh>,
    material_key: Option<MaterialKey>,
    transform: Affine3A,
    draw_index: u32,
    render_layers: RenderLayers,
    graphics_entity: Entity,
}

impl BatchState {
    fn new(graphics_entity: Entity, render_layers: RenderLayers) -> Self {
        Self {
            current_mesh: None,
            material_key: None,
            transform: Affine3A::IDENTITY,
            draw_index: 0,
            render_layers,
            graphics_entity,
        }
    }
}

/// Applies a text-style command to `style`; shared by the flush and by text queries, which
/// replay the frame's pending commands. `font_family` resolves a font entity.
pub(crate) fn apply_text_style_command(
    style: &mut StyleStack,
    cmd: DrawCommand,
    font_family: impl Fn(Entity) -> Option<String>,
) {
    match cmd {
        DrawCommand::TextFont(font_entity) => {
            if let Some(entity) = font_entity {
                if let Some(family) = font_family(entity) {
                    style.text_font_family = Some(family);
                }
            } else {
                style.text_font_family = None;
            }
        }
        DrawCommand::TextStyle(text_style) => style.text_style = text_style,
        DrawCommand::TextWeight(weight) => style.text_weight = Some(weight),
        DrawCommand::TextVariation { tag, value } => {
            if let Some(existing) = style.text_variations.iter_mut().find(|(t, _)| *t == tag) {
                existing.1 = value;
            } else {
                style.text_variations.push((tag, value));
            }
        }
        DrawCommand::ClearTextVariations => style.text_variations.clear(),
        DrawCommand::TextFeature { tag, value } => {
            if let Some(existing) = style.text_features.iter_mut().find(|(t, _)| *t == tag) {
                existing.1 = value;
            } else {
                style.text_features.push((tag, value));
            }
        }
        DrawCommand::NoTextFeature { tag } => style.text_features.retain(|(t, _)| *t != tag),
        DrawCommand::ClearTextFeatures => style.text_features.clear(),
        DrawCommand::TextSize(size) => {
            style.text_size = size;
            style.text_leading = None;
        }
        DrawCommand::TextAlign { h, v } => {
            style.text_align_h = h;
            style.text_align_v = v;
        }
        DrawCommand::TextLeading(leading) => style.text_leading = Some(leading),
        DrawCommand::TextWrap(mode) => style.text_wrap = mode,
        DrawCommand::TextGlyphColors(colors) => style.text_glyph_colors = Some(colors),
        _ => {}
    }
}

#[derive(Debug, Component)]
pub struct RenderState {
    pub style: StyleStack,
    pub transform: TransformStack,
    pub shape_builder: Option<ShapeBuilder>,
    /// Like Processing, every frame starts with a fresh depth buffer
    pub clear_depth: bool,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            style: StyleStack::new(),
            transform: TransformStack::new(),
            shape_builder: None,
            clear_depth: true,
        }
    }

    pub fn reset(&mut self) {
        self.style = StyleStack::new();
        self.transform = TransformStack::new();
        self.shape_builder = None;
    }

    pub fn begin_frame(&mut self) {
        self.transform = TransformStack::new();
        self.style.clear_saved();
        self.shape_builder = None;
        self.clear_depth = true;
    }

    pub fn fill_is_transparent(&self) -> bool {
        self.style
            .fill
            .color()
            .map(|c| c.alpha() < 1.0)
            .unwrap_or(false)
    }

    pub fn stroke_is_transparent(&self) -> bool {
        self.style
            .stroke_color
            .map(|c| c.alpha() < 1.0)
            .unwrap_or(false)
    }
}

impl Default for RenderState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn flush_draw_commands(
    mut res: RenderResources,
    mut graphics: Query<
        (
            Entity,
            &mut CommandBuffer,
            &mut RenderState,
            &RenderLayers,
            &mut Camera,
            &mut Camera3d,
            &mut crate::image::BackgroundBlit,
            Has<crate::graphics::Present>,
            Option<&RenderTarget>,
        ),
        With<Flush>,
    >,
    p_windows: Query<&Window>,
    p_images: Query<&Image>,
    p_geometries: Query<(&Geometry, Option<&GltfNodeTransform>)>,
    p_material_handles: Query<&UntypedMaterial>,
    mut p_particles: Query<&mut Particles>,
    p_particle_buffers: Query<&crate::compute::Buffer>,
    builtin_attributes: Res<crate::geometry::attribute::BuiltinAttributes>,
    p_fonts: Query<&crate::text::font::Font>,
    text_cx: Res<TextContext>,
    p_raster_draws: Query<
        (Entity, &RenderLayers),
        Or<(
            With<crate::particles::point_render::ParticleRasterDraw>,
            With<ParticlesDraw>,
        )>,
    >,
    p_lights: Query<(&ProcessingLight, &Transform)>,
) {
    for (
        graphics_entity,
        mut cmd_buffer,
        mut state,
        render_layers,
        mut camera,
        mut camera_3d,
        mut background_blit,
        presents,
        target,
    ) in graphics.iter_mut()
    {
        // an opaque window ignores background alpha, as in Processing
        let opaque_window = matches!(
            target,
            Some(RenderTarget::Window(WindowRef::Entity(window)))
                if p_windows.get(*window).is_ok_and(|w| !w.transparent)
        );

        // color is kept unless `background()` opens the pass; depth is kept only within a frame
        camera.clear_color = ClearColorConfig::None;
        camera_3d.depth_load_op = if std::mem::take(&mut state.clear_depth) {
            Camera3dDepthLoadOp::Clear(0.0)
        } else {
            Camera3dDepthLoadOp::Load
        };

        for (raster_entity, raster_layers) in p_raster_draws.iter() {
            if raster_layers.intersects(render_layers) {
                res.commands
                    .entity(raster_entity)
                    .insert(Visibility::Hidden);
            }
        }

        let draw_commands = std::mem::take(&mut cmd_buffer.commands);
        let mut batch = BatchState::new(graphics_entity, render_layers.clone());
        let mut pending = draw_commands.into_iter();
        let mut drained = true;

        while let Some(cmd) = pending.next() {
            // check whether we are the first flush after a present
            let opens_pass =
                |batch: &BatchState| batch.draw_index == 0 && batch.current_mesh.is_none();
            match cmd {
                DrawCommand::Light(entity) => {
                    if let Ok((light, transform)) = p_lights.get(entity) {
                        let placed = crate::light::place(state.transform.current(), transform);
                        crate::light::spawn_bevy_light(
                            &mut res.commands,
                            graphics_entity,
                            render_layers,
                            light.0,
                            placed,
                        );
                    }
                }
                DrawCommand::Fill(color) => {
                    state.style.fill = Fill::Color(color);
                }
                DrawCommand::FillBuffer(buf_entity) => {
                    state.style.fill = Fill::Buffer(buf_entity);
                }
                DrawCommand::NoFill => {
                    state.style.fill = Fill::None;
                }
                DrawCommand::StrokeColor(color) => {
                    state.style.stroke_color = Some(color);
                }
                DrawCommand::NoStroke => {
                    state.style.stroke_color = None;
                }
                DrawCommand::StrokeWeight(weight) => {
                    state.style.stroke_weight = weight;
                }
                DrawCommand::StrokeCap(cap) => {
                    state.style.stroke_config.line_cap = cap;
                }
                DrawCommand::StrokeJoin(join) => {
                    state.style.stroke_config.line_join = join;
                }
                DrawCommand::Roughness(r) => {
                    let mut pbr = state.style.material_key.as_pbr();
                    pbr.roughness = (r * 255.0) as u8;
                    pbr.blend_state = None;
                    state.style.material_key = pbr.into();
                }
                DrawCommand::Metallic(m) => {
                    let mut pbr = state.style.material_key.as_pbr();
                    pbr.metallic = (m * 255.0) as u8;
                    pbr.blend_state = None;
                    state.style.material_key = pbr.into();
                }
                DrawCommand::Emissive(color) => {
                    let mut pbr = state.style.material_key.as_pbr();
                    pbr.emissive = color.to_srgba().to_u8_array();
                    pbr.blend_state = None;
                    state.style.material_key = pbr.into();
                }
                DrawCommand::Texture(entity) => {
                    if let Ok(img) = p_images.get(entity) {
                        let mut pbr = state.style.material_key.as_pbr();
                        pbr.base_color_texture = Some(img.handle.clone());
                        pbr.transparent = img.translucent;
                        state.style.material_key = pbr.into();
                    }
                }
                DrawCommand::NoTexture => {
                    if let MaterialKey::Pbr { .. } = &state.style.material_key {
                        let mut pbr = state.style.material_key.as_pbr();
                        pbr.base_color_texture = None;
                        pbr.transparent = false;
                        state.style.material_key = pbr.into();
                    }
                }
                DrawCommand::TextureTransform(transform) => {
                    let mut pbr = state.style.material_key.as_pbr();
                    pbr.uv_transform = transform;
                    state.style.material_key = pbr.into();
                }
                DrawCommand::Unlit => {
                    state.style.material_key = MaterialKey::Color {
                        transparent: state.fill_is_transparent(),
                        background_image: None,
                        uv_transform: Affine2::IDENTITY,
                        blend_state: None,
                    };
                }
                DrawCommand::RectMode(mode) => {
                    state.style.rect_mode = mode;
                }
                DrawCommand::EllipseMode(mode) => {
                    state.style.ellipse_mode = mode;
                }
                DrawCommand::Rect { x, y, w, h, radii } => {
                    let (x, y, w, h) = apply_shape_mode(state.style.rect_mode, x, y, w, h);
                    let stroke_config = state.style.stroke_config;
                    add_fill(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color| {
                            rect(
                                mesh,
                                x,
                                y,
                                w,
                                h,
                                radii,
                                color,
                                TessellationMode::Fill,
                                &stroke_config,
                            )
                        },
                        &p_material_handles,
                    );

                    add_stroke(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color, weight| {
                            rect(
                                mesh,
                                x,
                                y,
                                w,
                                h,
                                radii,
                                color,
                                TessellationMode::Stroke(weight),
                                &stroke_config,
                            )
                        },
                        &p_material_handles,
                    );
                }
                DrawCommand::Ellipse { cx, cy, w, h } => {
                    // apply_shape_mode converts to top-left corner form, then we
                    // compute center from that
                    let (x, y, w, h) = apply_shape_mode(state.style.ellipse_mode, cx, cy, w, h);
                    let cx = x + w / 2.0;
                    let cy = y + h / 2.0;
                    let stroke_config = state.style.stroke_config;
                    add_fill(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color| {
                            ellipse(
                                mesh,
                                cx,
                                cy,
                                w,
                                h,
                                color,
                                TessellationMode::Fill,
                                &stroke_config,
                            )
                        },
                        &p_material_handles,
                    );

                    add_stroke(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color, weight| {
                            ellipse(
                                mesh,
                                cx,
                                cy,
                                w,
                                h,
                                color,
                                TessellationMode::Stroke(weight),
                                &stroke_config,
                            )
                        },
                        &p_material_handles,
                    );
                }
                DrawCommand::Line { x1, y1, x2, y2 } => {
                    let stroke_config = state.style.stroke_config;
                    add_stroke(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color, weight| {
                            line(mesh, x1, y1, x2, y2, color, weight, &stroke_config)
                        },
                        &p_material_handles,
                    );
                }
                DrawCommand::Triangle {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                } => {
                    let stroke_config = state.style.stroke_config;
                    add_fill(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color| {
                            triangle(
                                mesh,
                                x1,
                                y1,
                                x2,
                                y2,
                                x3,
                                y3,
                                color,
                                TessellationMode::Fill,
                                &stroke_config,
                            )
                        },
                        &p_material_handles,
                    );

                    add_stroke(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color, weight| {
                            triangle(
                                mesh,
                                x1,
                                y1,
                                x2,
                                y2,
                                x3,
                                y3,
                                color,
                                TessellationMode::Stroke(weight),
                                &stroke_config,
                            )
                        },
                        &p_material_handles,
                    );
                }
                DrawCommand::Quad {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                    x4,
                    y4,
                } => {
                    let stroke_config = state.style.stroke_config;
                    add_fill(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color| {
                            quad(
                                mesh,
                                x1,
                                y1,
                                x2,
                                y2,
                                x3,
                                y3,
                                x4,
                                y4,
                                color,
                                TessellationMode::Fill,
                                &stroke_config,
                            )
                        },
                        &p_material_handles,
                    );

                    add_stroke(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color, weight| {
                            quad(
                                mesh,
                                x1,
                                y1,
                                x2,
                                y2,
                                x3,
                                y3,
                                x4,
                                y4,
                                color,
                                TessellationMode::Stroke(weight),
                                &stroke_config,
                            )
                        },
                        &p_material_handles,
                    );
                }
                DrawCommand::Point { x, y } => {
                    if let Some(color) = state.style.stroke_color {
                        let d = state.style.stroke_weight;
                        let material_key = material_key_with_color(
                            &state.style.material_key,
                            color,
                            state.style.blend_state,
                        );

                        if needs_batch(&batch, &state, &material_key) {
                            start_batch(
                                &mut res,
                                &mut batch,
                                &state,
                                material_key,
                                &p_material_handles,
                            );
                        }

                        if let Some(ref mut mesh) = batch.current_mesh {
                            let stroke_config = state.style.stroke_config;
                            ellipse(
                                mesh,
                                x,
                                y,
                                d,
                                d,
                                color,
                                TessellationMode::Fill,
                                &stroke_config,
                            );
                        }
                    }
                }
                DrawCommand::Arc {
                    cx,
                    cy,
                    w,
                    h,
                    start,
                    stop,
                    mode,
                } => {
                    let (x, y, w, h) = apply_shape_mode(state.style.ellipse_mode, cx, cy, w, h);
                    let cx = x + w / 2.0;
                    let cy = y + h / 2.0;
                    let stroke_config = state.style.stroke_config;
                    add_fill(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color| {
                            arc_fill(mesh, cx, cy, w, h, start, stop, mode, color, &stroke_config)
                        },
                        &p_material_handles,
                    );

                    add_stroke(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color, weight| {
                            arc_stroke(
                                mesh,
                                cx,
                                cy,
                                w,
                                h,
                                start,
                                stop,
                                mode,
                                color,
                                weight,
                                &stroke_config,
                            )
                        },
                        &p_material_handles,
                    );
                }
                DrawCommand::Bezier {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                    x4,
                    y4,
                } => {
                    let stroke_config = state.style.stroke_config;
                    add_stroke(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color, weight| {
                            bezier(
                                mesh,
                                x1,
                                y1,
                                x2,
                                y2,
                                x3,
                                y3,
                                x4,
                                y4,
                                color,
                                weight,
                                &stroke_config,
                            )
                        },
                        &p_material_handles,
                    );
                }
                DrawCommand::Curve {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                    x4,
                    y4,
                } => {
                    let stroke_config = state.style.stroke_config;
                    add_stroke(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color, weight| {
                            curve(
                                mesh,
                                x1,
                                y1,
                                x2,
                                y2,
                                x3,
                                y3,
                                x4,
                                y4,
                                color,
                                weight,
                                &stroke_config,
                            )
                        },
                        &p_material_handles,
                    );
                }
                DrawCommand::BeginShape { kind } => {
                    state.shape_builder = Some(ShapeBuilder::new(kind));
                }
                DrawCommand::ShapeVertex { x, y } => {
                    if let Some(ref mut sb) = state.shape_builder {
                        sb.push_vertex(VertexType::Normal(x, y));
                    }
                }
                DrawCommand::ShapeBezierVertex {
                    cx1,
                    cy1,
                    cx2,
                    cy2,
                    x,
                    y,
                } => {
                    if let Some(ref mut sb) = state.shape_builder {
                        sb.push_vertex(VertexType::CubicBezier {
                            cx1,
                            cy1,
                            cx2,
                            cy2,
                            x,
                            y,
                        });
                    }
                }
                DrawCommand::ShapeQuadraticVertex { cx, cy, x, y } => {
                    if let Some(ref mut sb) = state.shape_builder {
                        sb.push_vertex(VertexType::QuadraticBezier { cx, cy, x, y });
                    }
                }
                DrawCommand::ShapeCurveVertex { x, y } => {
                    if let Some(ref mut sb) = state.shape_builder {
                        sb.push_vertex(VertexType::CurveVertex(x, y));
                    }
                }
                DrawCommand::BeginContour => {
                    if let Some(ref mut sb) = state.shape_builder {
                        sb.begin_contour();
                    }
                }
                DrawCommand::EndContour => {
                    if let Some(ref mut sb) = state.shape_builder {
                        sb.end_contour();
                    }
                }
                DrawCommand::EndShape { close } => {
                    if let Some(sb) = state.shape_builder.take() {
                        let stroke_config = state.style.stroke_config;
                        use crate::render::command::ShapeKind;

                        match sb.kind {
                            ShapeKind::Polygon => {
                                add_fill(
                                    &mut res,
                                    &mut batch,
                                    &state,
                                    |mesh, color| {
                                        build_polygon_fill(mesh, &sb, close, color, &stroke_config)
                                    },
                                    &p_material_handles,
                                );
                                add_stroke(
                                    &mut res,
                                    &mut batch,
                                    &state,
                                    |mesh, color, weight| {
                                        build_polygon_stroke(
                                            mesh,
                                            &sb,
                                            close,
                                            color,
                                            weight,
                                            &stroke_config,
                                        )
                                    },
                                    &p_material_handles,
                                );
                            }
                            ShapeKind::Points => {
                                if let Some(color) = state.style.stroke_color {
                                    let d = state.style.stroke_weight;
                                    let material_key = material_key_with_color(
                                        &state.style.material_key,
                                        color,
                                        state.style.blend_state,
                                    );
                                    if needs_batch(&batch, &state, &material_key) {
                                        start_batch(
                                            &mut res,
                                            &mut batch,
                                            &state,
                                            material_key,
                                            &p_material_handles,
                                        );
                                    }
                                    if let Some(ref mut mesh) = batch.current_mesh {
                                        for v in &sb.contours[0].vertices {
                                            if let VertexType::Normal(x, y) = v {
                                                ellipse(
                                                    mesh,
                                                    *x,
                                                    *y,
                                                    d,
                                                    d,
                                                    color,
                                                    TessellationMode::Fill,
                                                    &stroke_config,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            ShapeKind::Lines => {
                                add_stroke(
                                    &mut res,
                                    &mut batch,
                                    &state,
                                    |mesh, color, weight| {
                                        build_direct_stroke(
                                            mesh,
                                            &sb,
                                            color,
                                            weight,
                                            &stroke_config,
                                        )
                                    },
                                    &p_material_handles,
                                );
                            }
                            _ => {
                                // Triangles, TriangleFan, TriangleStrip, Quads, QuadStrip
                                add_fill(
                                    &mut res,
                                    &mut batch,
                                    &state,
                                    |mesh, color| build_direct_fill(mesh, &sb, color),
                                    &p_material_handles,
                                );
                                add_stroke(
                                    &mut res,
                                    &mut batch,
                                    &state,
                                    |mesh, color, weight| {
                                        build_direct_stroke(
                                            mesh,
                                            &sb,
                                            color,
                                            weight,
                                            &stroke_config,
                                        )
                                    },
                                    &p_material_handles,
                                );
                            }
                        }
                    }
                }
                DrawCommand::Tint(color) => {
                    state.style.tint_color = Some(color);
                }
                DrawCommand::NoTint => {
                    state.style.tint_color = None;
                }
                DrawCommand::ImageMode(mode) => {
                    state.style.image_mode = mode;
                }
                DrawCommand::Image {
                    entity,
                    dx,
                    dy,
                    d_width,
                    d_height,
                    sx,
                    sy,
                    s_width,
                    s_height,
                } => {
                    let Some(p_image) = p_images.get(entity).ok() else {
                        warn!("Could not find PImage for entity {:?}", entity);
                        continue;
                    };

                    let img_w = p_image.size.width as f32;
                    let img_h = p_image.size.height as f32;
                    let dw = d_width.unwrap_or(img_w);
                    let dh = d_height.unwrap_or(img_h);
                    let (x, y, w, h) = apply_shape_mode(state.style.image_mode, dx, dy, dw, dh);

                    let uv_xform = match (sx, sy, s_width, s_height) {
                        (Some(sx), Some(sy), Some(sw), Some(sh)) => {
                            Affine2::from_scale_angle_translation(
                                Vec2::new(sw / img_w, sh / img_h),
                                0.0,
                                Vec2::new(sx / img_w, sy / img_h),
                            )
                        }
                        _ => Affine2::IDENTITY,
                    };

                    let tint = state.style.tint_color.unwrap_or(Color::WHITE);
                    let material_key = MaterialKey::Color {
                        transparent: tint.alpha() < 1.0 || p_image.translucent,
                        background_image: Some(p_image.handle.clone()),
                        uv_transform: uv_xform,
                        blend_state: state.style.blend_state,
                    };
                    let stroke_config = state.style.stroke_config;

                    flush_batch(&mut res, &mut batch, &p_material_handles);
                    start_batch(
                        &mut res,
                        &mut batch,
                        &state,
                        material_key,
                        &p_material_handles,
                    );

                    if let Some(ref mut mesh) = batch.current_mesh {
                        rect(
                            mesh,
                            x,
                            y,
                            w,
                            h,
                            [0.0; 4],
                            tint,
                            TessellationMode::Fill,
                            &stroke_config,
                        );
                    }

                    flush_batch(&mut res, &mut batch, &p_material_handles);
                }
                DrawCommand::BackgroundColor(color) => {
                    if !opens_pass(&batch) {
                        cmd_buffer.commands = std::iter::once(cmd).chain(pending).collect();
                        drained = false;
                        break;
                    }
                    let color = if opaque_window {
                        color.with_alpha(1.0)
                    } else {
                        color
                    };
                    camera.clear_color = ClearColorConfig::Custom(color);
                    camera_3d.depth_load_op = Camera3dDepthLoadOp::Clear(0.0);
                }
                DrawCommand::BackgroundImage(entity) => {
                    let Some(p_image) = p_images.get(entity).ok() else {
                        warn!("Could not find PImage for entity {:?}", entity);
                        continue;
                    };
                    if !opens_pass(&batch) {
                        cmd_buffer.commands = std::iter::once(cmd).chain(pending).collect();
                        drained = false;
                        break;
                    }
                    // the image is blitted over the target before the pass, which loads it
                    camera_3d.depth_load_op = Camera3dDepthLoadOp::Clear(0.0);
                    background_blit.0 = Some(p_image.handle.clone());
                }
                DrawCommand::PushMatrix => state.transform.push(),
                DrawCommand::PopMatrix => state.transform.pop(),
                DrawCommand::ResetMatrix => state.transform.reset(),
                DrawCommand::ApplyMatrix(m) => state.transform.apply(m),
                DrawCommand::PushStyle => state.style.push(),
                DrawCommand::PopStyle => state.style.pop(),
                DrawCommand::Translate(v) => state.transform.apply(Affine3A::from_translation(v)),
                DrawCommand::Rotate { angle, axis } => state
                    .transform
                    .apply(Affine3A::from_axis_angle(axis.normalize(), angle)),
                DrawCommand::Scale(v) => state.transform.apply(Affine3A::from_scale(v)),
                DrawCommand::ShearX { angle } => state.transform.apply(transform::shear_x(angle)),
                DrawCommand::ShearY { angle } => state.transform.apply(transform::shear_y(angle)),
                DrawCommand::Geometry(entity) => {
                    let Some((geometry, node_transform)) = p_geometries.get(entity).ok() else {
                        warn!("Could not find Geometry for entity {:?}", entity);
                        continue;
                    };

                    let mut material_key = material_key_with_fill(&state);
                    if geometry.translucent {
                        material_key = material_key.transparent();
                    }
                    let material_handle = match &material_key {
                        MaterialKey::Custom {
                            entity: mat_entity,
                            blend_state,
                        } => {
                            let Some(untyped) = p_material_handles.get(*mat_entity).ok() else {
                                warn!("Could not find material for entity {:?}", mat_entity);
                                continue;
                            };
                            clone_custom_material_with_blend(
                                &mut res.custom_materials,
                                &untyped.0,
                                *blend_state,
                            )
                        }
                        _ => material_key
                            .to_material(&mut res.materials, sort_bias(batch.draw_index)),
                    };

                    flush_batch(&mut res, &mut batch, &p_material_handles);

                    let z_offset = layer_z(batch.draw_index);
                    let mut transform = state.transform.to_bevy_transform();

                    // if the "source" geometry was parented in a gltf scene, we need to make sure that
                    // we apply the parent transform here to ensure the correct final transform
                    // TODO: think about how hierarchies should work, especially for retained
                    if let Some(nt) = node_transform {
                        transform =
                            Transform::from_matrix(transform.to_matrix() * nt.0.to_matrix());
                    }
                    transform.translation.z += z_offset;

                    res.commands.spawn((
                        Mesh3d(geometry.handle.clone()),
                        UntypedMaterial(material_handle),
                        FlushTransient(batch.graphics_entity),
                        transform,
                        batch.render_layers.clone(),
                    ));

                    batch.draw_index += 1;
                }
                DrawCommand::Particles {
                    particles,
                    geometry: None,
                    topology,
                } => {
                    let Ok(mut particles_data) = p_particles.get_mut(particles) else {
                        warn!("Could not find Particles for entity {:?}", particles);
                        continue;
                    };

                    let position_attr = builtin_attributes.position;
                    let Some(&buffer_entity) = particles_data.buffers.get(&position_attr) else {
                        warn!(
                            "particles(p) with no geometry needs a materialized `position` buffer"
                        );
                        continue;
                    };
                    let Ok(buffer) = p_particle_buffers.get(buffer_entity) else {
                        warn!("position buffer {:?} has no compute::Buffer", buffer_entity);
                        continue;
                    };
                    let position = buffer.handle.clone();
                    let count = particles_data.capacity;

                    let (index, indirect) = match particles_data.connectivity {
                        Some(connectivity) => {
                            let (Ok(index_buf), Ok(indirect_buf)) = (
                                p_particle_buffers.get(connectivity.index_buffer),
                                p_particle_buffers.get(connectivity.indirect_buffer),
                            ) else {
                                warn!("connectivity buffers for {:?} not found", particles);
                                continue;
                            };
                            (
                                Some(index_buf.handle.clone()),
                                Some(indirect_buf.handle.clone()),
                            )
                        }
                        None => (None, None),
                    };

                    let color = particles_data
                        .buffers
                        .get(&builtin_attributes.color)
                        .and_then(|&e| p_particle_buffers.get(e).ok())
                        .map(|b| b.handle.clone());
                    let normal = particles_data
                        .buffers
                        .get(&builtin_attributes.normal)
                        .and_then(|&e| p_particle_buffers.get(e).ok())
                        .map(|b| b.handle.clone());
                    let render_layers = batch.render_layers.clone();

                    flush_batch(&mut res, &mut batch, &p_material_handles);

                    let raster_draw = crate::particles::point_render::ParticleRasterDraw {
                        position,
                        count,
                        topology,
                        index,
                        indirect,
                        color,
                        normal,
                        blend: state.style.blend_state,
                    };
                    match particles_data.raster_draw_entity {
                        Some(e) => {
                            res.commands.entity(e).insert((
                                raster_draw,
                                render_layers,
                                Visibility::Visible,
                            ));
                        }
                        None => {
                            let e = res
                                .commands
                                .spawn((
                                    raster_draw,
                                    Visibility::Visible,
                                    Transform::default(),
                                    NoFrustumCulling,
                                    render_layers,
                                ))
                                .id();
                            particles_data.raster_draw_entity = Some(e);
                        }
                    }

                    batch.draw_index += 1;
                }
                DrawCommand::Particles {
                    particles,
                    geometry: Some(geometry),
                    topology: _,
                } => {
                    let Some((geometry_data, _)) = p_geometries.get(geometry).ok() else {
                        warn!("Could not find Geometry for entity {:?}", geometry);
                        continue;
                    };
                    let Ok(mut particles_data) = p_particles.get_mut(particles) else {
                        warn!("Could not find Particles for entity {:?}", particles);
                        continue;
                    };

                    let material_handle = if let Fill::Buffer(buf_entity) = state.style.fill {
                        match particles_fill_material(&mut res, buf_entity) {
                            Some(h) => h,
                            None => {
                                warn!("fill(buffer) entity {:?} not found", buf_entity);
                                continue;
                            }
                        }
                    } else {
                        let material_key = material_key_with_fill(&state);
                        match &material_key {
                            MaterialKey::Custom {
                                entity: mat_entity,
                                blend_state,
                            } => {
                                let Some(untyped) = p_material_handles.get(*mat_entity).ok() else {
                                    warn!("Could not find material for entity {:?}", mat_entity);
                                    continue;
                                };
                                clone_custom_material_with_blend(
                                    &mut res.custom_materials,
                                    &untyped.0,
                                    *blend_state,
                                )
                            }
                            _ => material_key
                                .to_material(&mut res.materials, sort_bias(batch.draw_index)),
                        }
                    };

                    flush_batch(&mut res, &mut batch, &p_material_handles);

                    let mesh_handle = geometry_data.handle.clone();
                    let capacity = particles_data.capacity;
                    let render_layers = batch.render_layers.clone();
                    match particles_data.draw_entity {
                        Some(e) => {
                            res.commands.entity(e).insert((
                                GpuBatchedMesh3d {
                                    mesh: mesh_handle,
                                    max_capacity: capacity,
                                },
                                UntypedMaterial(material_handle),
                                render_layers,
                                Visibility::Visible,
                            ));
                        }
                        None => {
                            let e = res
                                .commands
                                .spawn((
                                    GpuBatchedMesh3d {
                                        mesh: mesh_handle,
                                        max_capacity: capacity,
                                    },
                                    UntypedMaterial(material_handle),
                                    Aabb {
                                        center: Vec3A::ZERO,
                                        half_extents: Vec3A::splat(1000.0),
                                    },
                                    ParticlesDraw { particles },
                                    render_layers,
                                    Visibility::Visible,
                                ))
                                .id();
                            particles_data.draw_entity = Some(e);
                        }
                    }

                    batch.draw_index += 1;
                }
                DrawCommand::BlendMode(blend_state) => {
                    state.style.blend_state = blend_state;
                }
                DrawCommand::Material(entity) => {
                    state.style.material_key = MaterialKey::Custom {
                        entity,
                        blend_state: None,
                    };
                }
                DrawCommand::Box {
                    width,
                    height,
                    depth,
                } => {
                    add_shape3d(
                        &mut res,
                        &mut batch,
                        &state,
                        box_mesh(width, height, depth),
                        &p_material_handles,
                    );
                }
                DrawCommand::Sphere {
                    radius,
                    sectors,
                    stacks,
                } => {
                    add_shape3d(
                        &mut res,
                        &mut batch,
                        &state,
                        sphere_mesh(radius, sectors, stacks),
                        &p_material_handles,
                    );
                }
                DrawCommand::Cylinder {
                    radius,
                    height,
                    detail,
                } => {
                    add_shape3d(
                        &mut res,
                        &mut batch,
                        &state,
                        cylinder_mesh(radius, height, detail),
                        &p_material_handles,
                    );
                }
                DrawCommand::Cone {
                    radius,
                    height,
                    detail,
                } => {
                    add_shape3d(
                        &mut res,
                        &mut batch,
                        &state,
                        cone_mesh(radius, height, detail),
                        &p_material_handles,
                    );
                }
                DrawCommand::Torus {
                    radius,
                    tube_radius,
                    major_segments,
                    minor_segments,
                } => {
                    add_shape3d(
                        &mut res,
                        &mut batch,
                        &state,
                        torus_mesh(radius, tube_radius, major_segments, minor_segments),
                        &p_material_handles,
                    );
                }
                DrawCommand::Plane { width, height } => {
                    add_shape3d(
                        &mut res,
                        &mut batch,
                        &state,
                        plane_mesh(width, height),
                        &p_material_handles,
                    );
                }
                DrawCommand::Capsule {
                    radius,
                    length,
                    detail,
                } => {
                    add_shape3d(
                        &mut res,
                        &mut batch,
                        &state,
                        capsule_mesh(radius, length, detail),
                        &p_material_handles,
                    );
                }
                DrawCommand::ConicalFrustum {
                    radius_top,
                    radius_bottom,
                    height,
                    detail,
                } => {
                    add_shape3d(
                        &mut res,
                        &mut batch,
                        &state,
                        conical_frustum_mesh(radius_top, radius_bottom, height, detail),
                        &p_material_handles,
                    );
                }
                DrawCommand::Tetrahedron { radius } => {
                    add_shape3d(
                        &mut res,
                        &mut batch,
                        &state,
                        tetrahedron_mesh(radius),
                        &p_material_handles,
                    );
                }
                cmd @ (DrawCommand::TextFont(_)
                | DrawCommand::TextStyle(_)
                | DrawCommand::TextWeight(_)
                | DrawCommand::TextVariation { .. }
                | DrawCommand::ClearTextVariations
                | DrawCommand::TextFeature { .. }
                | DrawCommand::NoTextFeature { .. }
                | DrawCommand::ClearTextFeatures
                | DrawCommand::TextSize(_)
                | DrawCommand::TextAlign { .. }
                | DrawCommand::TextLeading(_)
                | DrawCommand::TextWrap(_)
                | DrawCommand::TextGlyphColors(_)) => {
                    apply_text_style_command(&mut state.style, cmd, |entity| {
                        p_fonts.get(entity).ok().map(|f| f.family_name.clone())
                    });
                }
                DrawCommand::Text {
                    content,
                    x,
                    y,
                    z,
                    max_w,
                    max_h,
                } => {
                    // rectMode applies to the bounding-box form
                    let (x, y, max_w, max_h) = if let (Some(w), Some(h)) = (max_w, max_h) {
                        let (bx, by, bw, bh) = apply_shape_mode(state.style.rect_mode, x, y, w, h);
                        (bx, by, Some(bw), Some(bh))
                    } else {
                        (x, y, max_w, max_h)
                    };

                    let mut text_params =
                        primitive::text::OwnedTextParams::from_render_state(&state, max_w, max_h);
                    // per-glyph colors apply to this one text() call only
                    text_params.glyph_colors = state.style.text_glyph_colors.take();
                    let text_cx = text_cx.clone();

                    if z != 0.0 {
                        state
                            .transform
                            .apply(Affine3A::from_translation(Vec3::new(0.0, 0.0, z)));
                    }

                    add_fill(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color| {
                            primitive::text::text(
                                mesh,
                                &content,
                                x,
                                y,
                                color,
                                &text_params.as_params(),
                                &text_cx,
                            );
                        },
                        &p_material_handles,
                    );

                    add_stroke(
                        &mut res,
                        &mut batch,
                        &state,
                        |mesh, color, weight| {
                            // per-glyph fill colors don't apply to the stroke
                            let mut params = text_params.as_params();
                            params.glyph_colors = None;
                            primitive::text::text_stroke(
                                mesh, &content, x, y, color, weight, &params, &text_cx,
                            );
                        },
                        &p_material_handles,
                    );

                    if z != 0.0 {
                        state
                            .transform
                            .apply(Affine3A::from_translation(Vec3::new(0.0, 0.0, -z)));
                    }
                }
            }
        }

        flush_batch(&mut res, &mut batch, &p_material_handles);

        // a pass cut short by a mid-frame `background()` is wiped by the next one, so it
        // must not reach the window
        camera.output_mode = if presents && drained {
            CameraOutputMode::Write {
                blend_state: None,
                clear_color: ClearColorConfig::None,
            }
        } else {
            CameraOutputMode::Skip
        };
    }
}

pub fn activate_cameras(mut cameras: Query<(&mut Camera, Option<&Flush>)>) {
    for (mut camera, flush) in cameras.iter_mut() {
        let active = flush.is_some();
        camera.is_active = active;
    }
}

pub fn clear_flush_transients(
    mut commands: Commands,
    surfaces: Query<&FlushTransients, With<Flush>>,
) {
    for transients in surfaces.iter() {
        for &entity in transients.0.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_mesh(
    res: &mut RenderResources,
    batch: &mut BatchState,
    mesh: Mesh,
    z_offset: f32,
    material_handles: &Query<&UntypedMaterial>,
) {
    let Some(key) = &batch.material_key else {
        return;
    };

    let mesh_handle = res.meshes.add(mesh);

    let (scale, rotation, translation) = batch.transform.to_scale_rotation_translation();
    let transform = Transform {
        translation: translation + Vec3::new(0.0, 0.0, z_offset),
        rotation,
        scale,
    };

    let material_handle = match key {
        MaterialKey::Custom {
            entity,
            blend_state,
        } => {
            let Some(untyped) = material_handles.get(*entity).ok() else {
                warn!("Custom material entity {:?} not found", entity);
                return;
            };
            clone_custom_material_with_blend(&mut res.custom_materials, &untyped.0, *blend_state)
        }
        _ => key.to_material(&mut res.materials, sort_bias(batch.draw_index)),
    };

    res.commands.spawn((
        Mesh3d(mesh_handle),
        UntypedMaterial(material_handle),
        FlushTransient(batch.graphics_entity),
        transform,
        batch.render_layers.clone(),
    ));
}

fn needs_batch(batch: &BatchState, state: &RenderState, material_key: &MaterialKey) -> bool {
    let material_changed = batch.material_key.as_ref() != Some(material_key);
    let transform_changed = batch.transform != state.transform.current();
    let requires_separate_draws = state.style.blend_state.is_some();
    material_changed || transform_changed || requires_separate_draws
}

fn start_batch(
    res: &mut RenderResources,
    batch: &mut BatchState,
    state: &RenderState,
    material_key: MaterialKey,
    material_handles: &Query<&UntypedMaterial>,
) {
    flush_batch(res, batch, material_handles);
    batch.material_key = Some(material_key);
    batch.transform = state.transform.current();
    batch.current_mesh = Some(empty_mesh());
}

fn apply_shape_mode(mode: ShapeMode, a: f32, b: f32, c: f32, d: f32) -> (f32, f32, f32, f32) {
    match mode {
        ShapeMode::Corner => (a, b, c, d),
        ShapeMode::Corners => (a, b, c - a, d - b),
        ShapeMode::Center => (a - c / 2.0, b - d / 2.0, c, d),
        ShapeMode::Radius => (a - c, b - d, c * 2.0, d * 2.0),
    }
}

fn clone_custom_material_with_blend(
    custom_materials: &mut Assets<CustomMaterial>,
    original: &UntypedHandle,
    blend_state: Option<BlendState>,
) -> UntypedHandle {
    match blend_state {
        None => original.clone(),
        Some(bs) => {
            let Ok(handle) = original.clone().try_typed::<CustomMaterial>() else {
                return original.clone();
            };
            let Some(original_mat) = custom_materials.get(&handle) else {
                return original.clone();
            };
            let mut variant = original_mat.clone();
            variant.blend_state = Some(bs);
            custom_materials.add(variant).untyped()
        }
    }
}

fn material_key_with_color(
    key: &MaterialKey,
    color: Color,
    blend_state: Option<BlendState>,
) -> MaterialKey {
    match key {
        MaterialKey::Color {
            background_image,
            uv_transform,
            ..
        } => MaterialKey::Color {
            transparent: color.alpha() < 1.0,
            background_image: background_image.clone(),
            uv_transform: *uv_transform,
            blend_state,
        },
        MaterialKey::Pbr { .. } => {
            let mut pbr = key.as_pbr();
            pbr.albedo = color.to_srgba().to_u8_array();
            pbr.blend_state = blend_state;
            pbr.into()
        }
        MaterialKey::Custom { entity, .. } => MaterialKey::Custom {
            entity: *entity,
            blend_state,
        },
    }
}

/// new `ParticlesMaterial` per call; not cached because per-frame alloc is cheap.
fn particles_fill_material(
    res: &mut RenderResources,
    buf_entity: Entity,
) -> Option<bevy::asset::UntypedHandle> {
    use crate::particles::material::{ParticlesExtension, ParticlesMaterial};

    let buf = res.particle_buffers.get(buf_entity).ok()?;
    let handle = res.particles_materials.add(ParticlesMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.4,
            metallic: 0.0,
            cull_mode: None,
            ..Default::default()
        },
        extension: ParticlesExtension {
            colors: Some(buf.handle.clone()),
            emissive_colors: None,
        },
    });
    Some(handle.untyped())
}

fn material_key_with_fill(state: &RenderState) -> MaterialKey {
    let color = state.style.fill.color().unwrap_or(Color::WHITE);
    material_key_with_color(&state.style.material_key, color, state.style.blend_state)
}

fn add_fill(
    res: &mut RenderResources,
    batch: &mut BatchState,
    state: &RenderState,
    tessellate: impl FnOnce(&mut Mesh, Color),
    material_handles: &Query<&UntypedMaterial>,
) {
    let Some(color) = state.style.fill.color() else {
        return;
    };
    let material_key =
        material_key_with_color(&state.style.material_key, color, state.style.blend_state);

    if needs_batch(batch, state, &material_key) {
        start_batch(res, batch, state, material_key, material_handles);
    }

    if let Some(ref mut mesh) = batch.current_mesh {
        tessellate(mesh, color);
    }
}

fn add_stroke(
    res: &mut RenderResources,
    batch: &mut BatchState,
    state: &RenderState,
    tessellate: impl FnOnce(&mut Mesh, Color, f32),
    material_handles: &Query<&UntypedMaterial>,
) {
    let Some(color) = state.style.stroke_color else {
        return;
    };
    let stroke_weight = state.style.stroke_weight;
    let material_key =
        material_key_with_color(&state.style.material_key, color, state.style.blend_state);

    if needs_batch(batch, state, &material_key) {
        start_batch(res, batch, state, material_key, material_handles);
    }

    if let Some(ref mut mesh) = batch.current_mesh {
        tessellate(mesh, color, stroke_weight);
    }
}

fn flush_batch(
    res: &mut RenderResources,
    batch: &mut BatchState,
    material_handles: &Query<&UntypedMaterial>,
) {
    if let Some(mesh) = batch.current_mesh.take() {
        let z_offset = layer_z(batch.draw_index);
        spawn_mesh(res, batch, mesh, z_offset, material_handles);
        batch.draw_index += 1;
    }
    batch.material_key = None;
}

fn add_shape3d(
    res: &mut RenderResources,
    batch: &mut BatchState,
    state: &RenderState,
    mesh: Mesh,
    material_handles: &Query<&UntypedMaterial>,
) {
    use bevy::pbr::wireframe::{Wireframe, WireframeColor, WireframeLineWidth, WireframeTopology};

    flush_batch(res, batch, material_handles);

    let mesh_handle = res.meshes.add(mesh);
    let fill_color = state.style.fill.color().unwrap_or(Color::WHITE);
    let material_handle = match &state.style.material_key {
        MaterialKey::Custom { entity, .. } => {
            let Some(untyped) = material_handles.get(*entity).ok() else {
                warn!("Custom material entity {:?} not found", entity);
                return;
            };
            clone_custom_material_with_blend(
                &mut res.custom_materials,
                &untyped.0,
                state.style.blend_state,
            )
        }
        // TODO: in 2d, we use vertex colors. `to_material` becomes complicated if we also encode
        // a base color in the material, so for simplicity we just create a new material here
        // that is unlit and uses the fill color as the base color
        MaterialKey::Color { .. } => {
            let base = StandardMaterial {
                base_color: fill_color,
                unlit: true,
                cull_mode: None,
                alpha_mode: if state.style.blend_state.is_some() || state.fill_is_transparent() {
                    AlphaMode::Blend
                } else {
                    AlphaMode::Opaque
                },
                depth_bias: sort_bias(batch.draw_index),
                ..default()
            };
            let extended = ProcessingExtendedMaterial {
                base,
                extension: ProcessingMaterial {
                    blend_state: state.style.blend_state,
                    depth_write: None,
                },
            };
            res.materials.add(extended).untyped()
        }
        _ => {
            let key = material_key_with_fill(state);
            key.to_material(&mut res.materials, sort_bias(batch.draw_index))
        }
    };

    let z_offset = layer_z(batch.draw_index);
    let mut transform = state.transform.to_bevy_transform();
    transform.translation.z += z_offset;

    let mut entity = res.commands.spawn((
        Mesh3d(mesh_handle),
        UntypedMaterial(material_handle),
        FlushTransient(batch.graphics_entity),
        transform,
        batch.render_layers.clone(),
    ));

    if let Some(stroke_color) = state.style.stroke_color {
        entity.insert((
            Wireframe,
            WireframeColor {
                color: stroke_color,
            },
            WireframeLineWidth {
                width: state.style.stroke_weight,
            },
            WireframeTopology::Quads,
        ));
    }

    batch.draw_index += 1;
}

/// Each draw gets its own world-z layer, growing toward the default camera
fn layer_z(draw_index: u32) -> f32 {
    draw_index as f32 * BATCH_INDEX_STEP
}

/// Tiebreak for the blended phase's depth sort
fn sort_bias(draw_index: u32) -> f32 {
    draw_index as f32 * 1e-6
}
