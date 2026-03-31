pub mod command;
pub mod material;
pub mod mesh_builder;
pub mod primitive;
pub mod transform;

use bevy::{
    camera::visibility::RenderLayers,
    ecs::system::SystemParam,
    math::{Affine3A, Mat4, Vec4},
    prelude::*,
    render::render_resource::BlendState,
};
use command::{CommandBuffer, DrawCommand, ShapeMode};
use material::{MaterialKey, ProcessingExtendedMaterial};
use primitive::{
    ShapeBuilder, StrokeConfig, TessellationMode, VertexType, arc_fill, arc_stroke, bezier,
    box_mesh, build_direct_fill, build_direct_stroke, build_polygon_fill, build_polygon_stroke,
    capsule_mesh, cone_mesh, conical_frustum_mesh, curve, cylinder_mesh, ellipse, empty_mesh, line,
    plane_mesh, quad, sphere_mesh, tetrahedron_mesh, torus_mesh, triangle,
};
use transform::TransformStack;

use crate::{
    Flush,
    geometry::Geometry,
    gltf::GltfNodeTransform,
    image::Image,
    material::ProcessingMaterial,
    material::custom::CustomMaterial,
    render::{material::UntypedMaterial, primitive::rect},
};

pub(crate) const BATCH_INDEX_STEP: f32 = 0.001;

#[derive(Component)]
#[relationship(relationship_target = TransientMeshes)]
pub struct BelongsToGraphics(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = BelongsToGraphics)]
pub struct TransientMeshes(Vec<Entity>);

#[derive(SystemParam)]
pub struct RenderResources<'w, 's> {
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ProcessingExtendedMaterial>>,
    custom_materials: ResMut<'w, Assets<CustomMaterial>>,
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

#[derive(Debug, Component)]
pub struct RenderState {
    pub fill_color: Option<Color>,
    pub stroke_color: Option<Color>,
    pub stroke_weight: f32,
    pub stroke_config: StrokeConfig,
    pub material_key: MaterialKey,
    pub blend_state: Option<BlendState>,
    pub transform: TransformStack,
    pub rect_mode: ShapeMode,
    pub ellipse_mode: ShapeMode,
    pub shape_builder: Option<ShapeBuilder>,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            fill_color: Some(Color::WHITE),
            stroke_color: Some(Color::BLACK),
            stroke_weight: 1.0,
            stroke_config: StrokeConfig::default(),
            material_key: MaterialKey::Color {
                transparent: false,
                background_image: None,
                blend_state: None,
            },
            blend_state: None,
            transform: TransformStack::new(),
            rect_mode: ShapeMode::Corner,
            ellipse_mode: ShapeMode::Center,
            shape_builder: None,
        }
    }

    pub fn reset(&mut self) {
        self.fill_color = Some(Color::WHITE);
        self.stroke_color = Some(Color::BLACK);
        self.stroke_weight = 1.0;
        self.stroke_config = StrokeConfig::default();
        self.material_key = MaterialKey::Color {
            transparent: false,
            background_image: None,
            blend_state: None,
        };
        self.blend_state = None;
        self.transform = TransformStack::new();
        self.rect_mode = ShapeMode::Corner;
        self.ellipse_mode = ShapeMode::Center;
        self.shape_builder = None;
    }

    pub fn fill_is_transparent(&self) -> bool {
        self.fill_color.map(|c| c.alpha() < 1.0).unwrap_or(false)
    }

    pub fn stroke_is_transparent(&self) -> bool {
        self.stroke_color.map(|c| c.alpha() < 1.0).unwrap_or(false)
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
            &Projection,
            &Transform,
        ),
        With<Flush>,
    >,
    p_images: Query<&Image>,
    p_geometries: Query<(&Geometry, Option<&GltfNodeTransform>)>,
    p_material_handles: Query<&UntypedMaterial>,
) {
    for (graphics_entity, mut cmd_buffer, mut state, render_layers, projection, camera_transform) in
        graphics.iter_mut()
    {
        let clip_from_view = projection.get_clip_from_view();
        let view_from_world = camera_transform.to_matrix().inverse();
        let world_from_clip = (clip_from_view * view_from_world).inverse();
        let draw_commands = std::mem::take(&mut cmd_buffer.commands);
        let mut batch = BatchState::new(graphics_entity, render_layers.clone());

        for cmd in draw_commands {
            match cmd {
                DrawCommand::Fill(color) => {
                    state.fill_color = Some(color);
                }
                DrawCommand::NoFill => {
                    state.fill_color = None;
                }
                DrawCommand::StrokeColor(color) => {
                    state.stroke_color = Some(color);
                }
                DrawCommand::NoStroke => {
                    state.stroke_color = None;
                }
                DrawCommand::StrokeWeight(weight) => {
                    state.stroke_weight = weight;
                }
                DrawCommand::StrokeCap(cap) => {
                    state.stroke_config.line_cap = cap;
                }
                DrawCommand::StrokeJoin(join) => {
                    state.stroke_config.line_join = join;
                }
                DrawCommand::Roughness(r) => {
                    state.material_key = match state.material_key {
                        MaterialKey::Pbr {
                            albedo,
                            metallic,
                            emissive,
                            ..
                        } => MaterialKey::Pbr {
                            albedo,
                            roughness: (r * 255.0) as u8,
                            metallic,
                            emissive,
                            blend_state: None,
                        },
                        _ => MaterialKey::Pbr {
                            albedo: [255, 255, 255, 255],
                            roughness: (r * 255.0) as u8,
                            metallic: 0,
                            emissive: [0, 0, 0, 0],
                            blend_state: None,
                        },
                    };
                }
                DrawCommand::Metallic(m) => {
                    state.material_key = match state.material_key {
                        MaterialKey::Pbr {
                            albedo,
                            roughness,
                            emissive,
                            ..
                        } => MaterialKey::Pbr {
                            albedo,
                            roughness,
                            metallic: (m * 255.0) as u8,
                            emissive,
                            blend_state: None,
                        },
                        _ => MaterialKey::Pbr {
                            albedo: [255, 255, 255, 255],
                            roughness: 128,
                            metallic: (m * 255.0) as u8,
                            emissive: [0, 0, 0, 0],
                            blend_state: None,
                        },
                    };
                }
                DrawCommand::Emissive(color) => {
                    let [r, g, b, a] = color.to_srgba().to_u8_array();
                    state.material_key = match state.material_key {
                        MaterialKey::Pbr {
                            albedo,
                            roughness,
                            metallic,
                            ..
                        } => MaterialKey::Pbr {
                            albedo,
                            roughness,
                            metallic,
                            emissive: [r, g, b, a],
                            blend_state: None,
                        },
                        _ => MaterialKey::Pbr {
                            albedo: [255, 255, 255, 255],
                            roughness: 128,
                            metallic: 0,
                            emissive: [r, g, b, a],
                            blend_state: None,
                        },
                    };
                }
                DrawCommand::Unlit => {
                    state.material_key = MaterialKey::Color {
                        transparent: state.fill_is_transparent(),
                        background_image: None,
                        blend_state: None,
                    };
                }
                DrawCommand::RectMode(mode) => {
                    state.rect_mode = mode;
                }
                DrawCommand::EllipseMode(mode) => {
                    state.ellipse_mode = mode;
                }
                DrawCommand::Rect { x, y, w, h, radii } => {
                    let (x, y, w, h) = apply_shape_mode(state.rect_mode, x, y, w, h);
                    let stroke_config = state.stroke_config;
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
                    let (x, y, w, h) = apply_shape_mode(state.ellipse_mode, cx, cy, w, h);
                    let cx = x + w / 2.0;
                    let cy = y + h / 2.0;
                    let stroke_config = state.stroke_config;
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
                    let stroke_config = state.stroke_config;
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
                    let stroke_config = state.stroke_config;
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
                    let stroke_config = state.stroke_config;
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
                    // Point is rendered as a filled circle using stroke color and weight
                    if let Some(color) = state.stroke_color {
                        let d = state.stroke_weight;
                        let material_key =
                            material_key_with_color(&state.material_key, color, state.blend_state);

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
                            let stroke_config = state.stroke_config;
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
                    let (x, y, w, h) = apply_shape_mode(state.ellipse_mode, cx, cy, w, h);
                    let cx = x + w / 2.0;
                    let cy = y + h / 2.0;
                    let stroke_config = state.stroke_config;
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
                    let stroke_config = state.stroke_config;
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
                    let stroke_config = state.stroke_config;
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
                        let stroke_config = state.stroke_config;
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
                                // Each vertex rendered as a filled circle
                                if let Some(color) = state.stroke_color {
                                    let d = state.stroke_weight;
                                    let material_key = material_key_with_color(
                                        &state.material_key,
                                        color,
                                        state.blend_state,
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
                DrawCommand::BackgroundColor(color) => {
                    flush_batch(&mut res, &mut batch, &p_material_handles);

                    let mesh = create_ndc_background_quad(world_from_clip, color, false);
                    let mesh_handle = res.meshes.add(mesh);

                    let material_key = MaterialKey::Color {
                        transparent: color.alpha() < 1.0,
                        background_image: None,
                        blend_state: Some(BlendState::REPLACE),
                    };
                    let material_handle = material_key.to_material(&mut res.materials);

                    res.commands.spawn((
                        Mesh3d(mesh_handle),
                        UntypedMaterial(material_handle),
                        BelongsToGraphics(batch.graphics_entity),
                        Transform::IDENTITY,
                        batch.render_layers.clone(),
                    ));

                    batch.draw_index += 1;
                }
                DrawCommand::BackgroundImage(entity) => {
                    let Some(p_image) = p_images.get(entity).ok() else {
                        warn!("Could not find PImage for entity {:?}", entity);
                        continue;
                    };

                    flush_batch(&mut res, &mut batch, &p_material_handles);

                    let mesh = create_ndc_background_quad(world_from_clip, Color::WHITE, true);
                    let mesh_handle = res.meshes.add(mesh);

                    let material_key = MaterialKey::Color {
                        transparent: false,
                        background_image: Some(p_image.handle.clone()),
                        blend_state: Some(BlendState::REPLACE),
                    };
                    let material_handle = material_key.to_material(&mut res.materials);

                    res.commands.spawn((
                        Mesh3d(mesh_handle),
                        UntypedMaterial(material_handle),
                        BelongsToGraphics(batch.graphics_entity),
                        Transform::IDENTITY,
                        batch.render_layers.clone(),
                    ));

                    batch.draw_index += 1;
                }
                DrawCommand::PushMatrix => state.transform.push(),
                DrawCommand::PopMatrix => state.transform.pop(),
                DrawCommand::ResetMatrix => state.transform.reset(),
                DrawCommand::Translate(v) => state.transform.translate(v.x, v.y),
                DrawCommand::Rotate { angle } => state.transform.rotate(angle),
                DrawCommand::Scale(v) => state.transform.scale(v.x, v.y),
                DrawCommand::ShearX { angle } => state.transform.shear_x(angle),
                DrawCommand::ShearY { angle } => state.transform.shear_y(angle),
                DrawCommand::Geometry(entity) => {
                    let Some((geometry, node_transform)) = p_geometries.get(entity).ok() else {
                        warn!("Could not find Geometry for entity {:?}", entity);
                        continue;
                    };

                    let material_key = material_key_with_fill(&state);
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
                        _ => material_key.to_material(&mut res.materials),
                    };

                    flush_batch(&mut res, &mut batch, &p_material_handles);

                    let z_offset = -(batch.draw_index as f32 * BATCH_INDEX_STEP);
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
                        BelongsToGraphics(batch.graphics_entity),
                        transform,
                        batch.render_layers.clone(),
                    ));

                    batch.draw_index += 1;
                }
                DrawCommand::BlendMode(blend_state) => {
                    state.blend_state = blend_state;
                }
                DrawCommand::Material(entity) => {
                    state.material_key = MaterialKey::Custom {
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
            }
        }

        flush_batch(&mut res, &mut batch, &p_material_handles);
    }
}

pub fn activate_cameras(mut cameras: Query<(&mut Camera, Option<&Flush>)>) {
    for (mut camera, flush) in cameras.iter_mut() {
        let active = flush.is_some();
        camera.is_active = active;
    }
}

pub fn clear_transient_meshes(
    mut commands: Commands,
    surfaces: Query<&TransientMeshes, With<Flush>>,
) {
    for transient_meshes in surfaces.iter() {
        for &mesh_entity in transient_meshes.0.iter() {
            commands.entity(mesh_entity).despawn();
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
        _ => key.to_material(&mut res.materials),
    };

    res.commands.spawn((
        Mesh3d(mesh_handle),
        UntypedMaterial(material_handle),
        BelongsToGraphics(batch.graphics_entity),
        transform,
        batch.render_layers.clone(),
    ));
}

fn needs_batch(batch: &BatchState, state: &RenderState, material_key: &MaterialKey) -> bool {
    let material_changed = batch.material_key.as_ref() != Some(material_key);
    let transform_changed = batch.transform != state.transform.current();
    let requires_separate_draws = state.blend_state.is_some();
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
            background_image, ..
        } => MaterialKey::Color {
            transparent: color.alpha() < 1.0,
            background_image: background_image.clone(),
            blend_state,
        },
        MaterialKey::Pbr {
            roughness,
            metallic,
            emissive,
            ..
        } => {
            let [r, g, b, a] = color.to_srgba().to_u8_array();
            MaterialKey::Pbr {
                albedo: [r, g, b, a],
                roughness: *roughness,
                metallic: *metallic,
                emissive: *emissive,
                blend_state,
            }
        }
        MaterialKey::Custom { entity, .. } => MaterialKey::Custom {
            entity: *entity,
            blend_state,
        },
    }
}

fn material_key_with_fill(state: &RenderState) -> MaterialKey {
    let color = state.fill_color.unwrap_or(Color::WHITE);
    material_key_with_color(&state.material_key, color, state.blend_state)
}

fn add_fill(
    res: &mut RenderResources,
    batch: &mut BatchState,
    state: &RenderState,
    tessellate: impl FnOnce(&mut Mesh, Color),
    material_handles: &Query<&UntypedMaterial>,
) {
    let Some(color) = state.fill_color else {
        return;
    };
    let material_key = material_key_with_color(&state.material_key, color, state.blend_state);

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
    let Some(color) = state.stroke_color else {
        return;
    };
    let stroke_weight = state.stroke_weight;
    let material_key = material_key_with_color(&state.material_key, color, state.blend_state);

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
        let z_offset = -(batch.draw_index as f32 * BATCH_INDEX_STEP);
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
    let fill_color = state.fill_color.unwrap_or(Color::WHITE);
    let material_handle = match &state.material_key {
        MaterialKey::Custom { entity, .. } => {
            let Some(untyped) = material_handles.get(*entity).ok() else {
                warn!("Custom material entity {:?} not found", entity);
                return;
            };
            clone_custom_material_with_blend(
                &mut res.custom_materials,
                &untyped.0,
                state.blend_state,
            )
        }
        // TODO: in 2d, we use vertex colors. `to_material` becomes complicated if we also encode
        // a base color in the material, so for simplicity we just create a new material here
        // that is unlit and uses the fill color as the base color
        MaterialKey::Color { transparent, .. } => {
            let base = StandardMaterial {
                base_color: fill_color,
                unlit: true,
                cull_mode: None,
                alpha_mode: if state.blend_state.is_some() || *transparent {
                    AlphaMode::Blend
                } else {
                    AlphaMode::Opaque
                },
                ..default()
            };
            let extended = ProcessingExtendedMaterial {
                base,
                extension: ProcessingMaterial {
                    blend_state: state.blend_state,
                },
            };
            res.materials.add(extended).untyped()
        }
        _ => {
            let key = material_key_with_fill(state);
            key.to_material(&mut res.materials)
        }
    };

    let z_offset = -(batch.draw_index as f32 * BATCH_INDEX_STEP);
    let mut transform = state.transform.to_bevy_transform();
    transform.translation.z += z_offset;

    let mut entity = res.commands.spawn((
        Mesh3d(mesh_handle),
        UntypedMaterial(material_handle),
        BelongsToGraphics(batch.graphics_entity),
        transform,
        batch.render_layers.clone(),
    ));

    if let Some(stroke_color) = state.stroke_color {
        entity.insert((
            Wireframe,
            WireframeColor {
                color: stroke_color,
            },
            WireframeLineWidth {
                width: state.stroke_weight,
            },
            WireframeTopology::Quads,
        ));
    }

    batch.draw_index += 1;
}

/// Creates a fullscreen quad by transforming NDC fullscreen by inverse of the clip-from-world matrix
/// so that when the vertex shader applies clip_from_world, the vertices end up correctly back in
/// NDC space.
fn create_ndc_background_quad(world_from_clip: Mat4, color: Color, with_uvs: bool) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};

    let ndc_z = 0.0;
    let ndc_corners = [
        Vec4::new(-1.0, -1.0, ndc_z, 1.0), // bl
        Vec4::new(1.0, -1.0, ndc_z, 1.0),  // br
        Vec4::new(1.0, 1.0, ndc_z, 1.0),   // tr
        Vec4::new(-1.0, 1.0, ndc_z, 1.0),  // tl
    ];

    let world_positions: Vec<[f32; 3]> = ndc_corners
        .iter()
        .map(|ndc| {
            let world = world_from_clip * *ndc;
            [world.x / world.w, world.y / world.w, world.z / world.w]
        })
        .collect();

    let uvs: Vec<[f32; 2]> = vec![
        [0.0, 1.0], // bl
        [1.0, 1.0], // br
        [1.0, 0.0], // tr
        [0.0, 0.0], // tl
    ];

    let color_array: [f32; 4] = color.to_linear().to_f32_array();
    let colors: Vec<[f32; 4]> = vec![color_array; 4];

    // two tris
    let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, world_positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    if with_uvs {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }
    mesh.insert_indices(Indices::U32(indices));

    mesh
}
