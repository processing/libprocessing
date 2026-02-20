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
};
use command::{CommandBuffer, DrawCommand};
use material::MaterialKey;
use primitive::{TessellationMode, box_mesh, empty_mesh, sphere_mesh};
use transform::TransformStack;

use crate::{
    Flush,
    geometry::Geometry,
    image::Image,
    render::{material::UntypedMaterial, primitive::rect},
};

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
    materials: ResMut<'w, Assets<StandardMaterial>>,
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
    pub material_key: MaterialKey,
    pub transform: TransformStack,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            fill_color: Some(Color::WHITE),
            stroke_color: Some(Color::BLACK),
            stroke_weight: 1.0,
            material_key: MaterialKey::Color {
                transparent: false,
                background_image: None,
            },
            transform: TransformStack::new(),
        }
    }

    pub fn reset(&mut self) {
        self.fill_color = Some(Color::WHITE);
        self.stroke_color = Some(Color::BLACK);
        self.stroke_weight = 1.0;
        self.material_key = MaterialKey::Color {
            transparent: false,
            background_image: None,
        };
        self.transform = TransformStack::new();
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
    p_geometries: Query<&Geometry>,
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
                        },
                        _ => MaterialKey::Pbr {
                            albedo: [255, 255, 255, 255],
                            roughness: (r * 255.0) as u8,
                            metallic: 0,
                            emissive: [0, 0, 0, 0],
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
                        },
                        _ => MaterialKey::Pbr {
                            albedo: [255, 255, 255, 255],
                            roughness: 128,
                            metallic: (m * 255.0) as u8,
                            emissive: [0, 0, 0, 0],
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
                        },
                        _ => MaterialKey::Pbr {
                            albedo: [255, 255, 255, 255],
                            roughness: 128,
                            metallic: 0,
                            emissive: [r, g, b, a],
                        },
                    };
                }
                DrawCommand::Unlit => {
                    state.material_key = MaterialKey::Color {
                        transparent: state.fill_is_transparent(),
                        background_image: None,
                    };
                }
                DrawCommand::Rect { x, y, w, h, radii } => {
                    add_fill(&mut res, &mut batch, &state, |mesh, color| {
                        rect(mesh, x, y, w, h, radii, color, TessellationMode::Fill)
                    });

                    add_stroke(&mut res, &mut batch, &state, |mesh, color, weight| {
                        rect(
                            mesh,
                            x,
                            y,
                            w,
                            h,
                            radii,
                            color,
                            TessellationMode::Stroke(weight),
                        )
                    });
                }
                DrawCommand::BackgroundColor(color) => {
                    flush_batch(&mut res, &mut batch);

                    let mesh = create_ndc_background_quad(world_from_clip, color, false);
                    let mesh_handle = res.meshes.add(mesh);

                    let material_key = MaterialKey::Color {
                        transparent: color.alpha() < 1.0,
                        background_image: None,
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

                    flush_batch(&mut res, &mut batch);

                    let mesh = create_ndc_background_quad(world_from_clip, Color::WHITE, true);
                    let mesh_handle = res.meshes.add(mesh);

                    let material_key = MaterialKey::Color {
                        transparent: false,
                        background_image: Some(p_image.handle.clone()),
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
                DrawCommand::Translate { x, y } => state.transform.translate(x, y),
                DrawCommand::Rotate { angle } => state.transform.rotate(angle),
                DrawCommand::Scale { x, y } => state.transform.scale(x, y),
                DrawCommand::ShearX { angle } => state.transform.shear_x(angle),
                DrawCommand::ShearY { angle } => state.transform.shear_y(angle),
                DrawCommand::Geometry(entity) => {
                    let Some(geometry) = p_geometries.get(entity).ok() else {
                        warn!("Could not find Geometry for entity {:?}", entity);
                        continue;
                    };

                    let material_key = material_key_with_fill(&state);
                    let material_handle = match &material_key {
                        MaterialKey::Custom(mat_entity) => {
                            let Some(handle) = p_material_handles.get(*mat_entity).ok() else {
                                warn!("Could not find material for entity {:?}", mat_entity);
                                continue;
                            };
                            handle.0.clone()
                        }
                        _ => material_key.to_material(&mut res.materials),
                    };

                    flush_batch(&mut res, &mut batch);

                    let z_offset = -(batch.draw_index as f32 * 0.001);
                    let mut transform = state.transform.to_bevy_transform();
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
                DrawCommand::Material(entity) => {
                    state.material_key = MaterialKey::Custom(entity);
                }
                DrawCommand::Box {
                    width,
                    height,
                    depth,
                } => {
                    add_shape3d(&mut res, &mut batch, &state, box_mesh(width, height, depth));
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
                    );
                }
            }
        }

        flush_batch(&mut res, &mut batch);
    }
}

pub fn activate_cameras(mut cameras: Query<(&mut Camera, Option<&Flush>)>) {
    for (mut camera, flush) in cameras.iter_mut() {
        camera.is_active = flush.is_some();
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

fn spawn_mesh(res: &mut RenderResources, batch: &mut BatchState, mesh: Mesh, z_offset: f32) {
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

    let material_handle = key.to_material(&mut res.materials);

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
    material_changed || transform_changed
}

fn start_batch(
    res: &mut RenderResources,
    batch: &mut BatchState,
    state: &RenderState,
    material_key: MaterialKey,
) {
    flush_batch(res, batch);
    batch.material_key = Some(material_key);
    batch.transform = state.transform.current();
    batch.current_mesh = Some(empty_mesh());
}

fn material_key_with_color(key: &MaterialKey, color: Color) -> MaterialKey {
    match key {
        MaterialKey::Color {
            background_image, ..
        } => MaterialKey::Color {
            transparent: color.alpha() < 1.0,
            background_image: background_image.clone(),
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
            }
        }
        MaterialKey::Custom(e) => MaterialKey::Custom(*e),
    }
}

fn material_key_with_fill(state: &RenderState) -> MaterialKey {
    let color = state.fill_color.unwrap_or(Color::WHITE);
    material_key_with_color(&state.material_key, color)
}

fn add_fill(
    res: &mut RenderResources,
    batch: &mut BatchState,
    state: &RenderState,
    tessellate: impl FnOnce(&mut Mesh, Color),
) {
    let Some(color) = state.fill_color else {
        return;
    };
    let material_key = material_key_with_color(&state.material_key, color);

    if needs_batch(batch, state, &material_key) {
        start_batch(res, batch, state, material_key);
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
) {
    let Some(color) = state.stroke_color else {
        return;
    };
    let stroke_weight = state.stroke_weight;
    let material_key = material_key_with_color(&state.material_key, color);

    if needs_batch(batch, state, &material_key) {
        start_batch(res, batch, state, material_key);
    }

    if let Some(ref mut mesh) = batch.current_mesh {
        tessellate(mesh, color, stroke_weight);
    }
}

fn flush_batch(res: &mut RenderResources, batch: &mut BatchState) {
    if let Some(mesh) = batch.current_mesh.take() {
        let z_offset = -(batch.draw_index as f32 * 0.001);
        spawn_mesh(res, batch, mesh, z_offset);
        batch.draw_index += 1;
    }
    batch.material_key = None;
}

fn add_shape3d(res: &mut RenderResources, batch: &mut BatchState, state: &RenderState, mesh: Mesh) {
    flush_batch(res, batch);

    let mesh_handle = res.meshes.add(mesh);
    let material_key = material_key_with_fill(state);
    let material_handle = material_key.to_material(&mut res.materials);

    let z_offset = -(batch.draw_index as f32 * 0.001);
    let mut transform = state.transform.to_bevy_transform();
    transform.translation.z += z_offset;

    res.commands.spawn((
        Mesh3d(mesh_handle),
        UntypedMaterial(material_handle),
        BelongsToGraphics(batch.graphics_entity),
        transform,
        batch.render_layers.clone(),
    ));

    batch.draw_index += 1;
}

/// Creates a fullscreen quad by transforming NDC fullscreen by inverse of the clip-from-world matrix
/// so that when the vertex shader applies clip_from_world, the vertices end up correctly back in
/// NDC space.
fn create_ndc_background_quad(world_from_clip: Mat4, color: Color, with_uvs: bool) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};

    let ndc_z = 0.001; // near far plane (bevy uses reverse-z)
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
