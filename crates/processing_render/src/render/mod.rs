pub mod command;
pub mod material;
pub mod mesh_builder;
pub mod primitive;
pub mod transform;

use bevy::{
    camera::visibility::RenderLayers, ecs::system::SystemParam, math::Affine3A, prelude::*,
};
use command::{CommandBuffer, DrawCommand};
use material::MaterialKey;
use primitive::{TessellationMode, empty_mesh};
use transform::TransformStack;

use crate::{
    Flush, geometry::Geometry, graphics::SurfaceSize, image::Image, render::primitive::rect,
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
    pub transform: TransformStack,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            fill_color: Some(Color::WHITE),
            stroke_color: Some(Color::BLACK),
            stroke_weight: 1.0,
            transform: TransformStack::new(),
        }
    }
}

impl RenderState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn fill_is_transparent(&self) -> bool {
        self.fill_color.map(|c| c.alpha() < 1.0).unwrap_or(false)
    }

    pub fn stroke_is_transparent(&self) -> bool {
        self.stroke_color.map(|c| c.alpha() < 1.0).unwrap_or(false)
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
            &SurfaceSize,
        ),
        With<Flush>,
    >,
    p_images: Query<&Image>,
    p_geometries: Query<&Geometry>,
) {
    for (graphics_entity, mut cmd_buffer, mut state, render_layers, SurfaceSize(width, height)) in
        graphics.iter_mut()
    {
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
                    add_fill(&mut res, &mut batch, &state, |mesh, _| {
                        rect(
                            mesh,
                            0.0,
                            0.0,
                            *width as f32,
                            *height as f32,
                            [0.0; 4],
                            color,
                            TessellationMode::Fill,
                        )
                    });
                }
                DrawCommand::BackgroundImage(entity) => {
                    let Some(p_image) = p_images.get(entity).ok() else {
                        warn!("Could not find PImage for entity {:?}", entity);
                        continue;
                    };

                    flush_batch(&mut res, &mut batch);

                    let material_key = MaterialKey {
                        transparent: false,
                        background_image: Some(p_image.handle.clone()),
                    };

                    batch.material_key = Some(material_key);
                    batch.current_mesh = Some(empty_mesh());

                    if let Some(ref mut mesh) = batch.current_mesh {
                        rect(
                            mesh,
                            0.0,
                            0.0,
                            *width as f32,
                            *height as f32,
                            [0.0; 4],
                            Color::WHITE,
                            TessellationMode::Fill,
                        )
                    }

                    flush_batch(&mut res, &mut batch);
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

                    flush_batch(&mut res, &mut batch);

                    // TODO: Implement state based material API
                    // https://github.com/processing/libprocessing/issues/10
                    let material_key = MaterialKey {
                        transparent: false, // TODO: detect from geometry colors
                        background_image: None,
                    };

                    let material_handle = res.materials.add(material_key.to_material());
                    let z_offset = -(batch.draw_index as f32 * 0.001);

                    let mut transform = state.transform.to_bevy_transform();
                    transform.translation.z += z_offset;

                    res.commands.spawn((
                        Mesh3d(geometry.handle.clone()),
                        MeshMaterial3d(material_handle),
                        BelongsToGraphics(batch.graphics_entity),
                        transform,
                        batch.render_layers.clone(),
                    ));

                    batch.draw_index += 1;
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
    let Some(material_key) = &batch.material_key else {
        return;
    };

    let mesh_handle = res.meshes.add(mesh);
    let material_handle = res.materials.add(material_key.to_material());

    let (scale, rotation, translation) = batch.transform.to_scale_rotation_translation();
    let transform = Transform {
        translation: translation + Vec3::new(0.0, 0.0, z_offset),
        rotation,
        scale,
    };

    res.commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        BelongsToGraphics(batch.graphics_entity),
        transform,
        batch.render_layers.clone(),
    ));
}

fn needs_batch(batch: &BatchState, state: &RenderState, material_key: &MaterialKey) -> bool {
    let current_transform = state.transform.current();
    let material_changed = batch.material_key.as_ref() != Some(material_key);
    let transform_changed = batch.transform != current_transform;
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

fn add_fill(
    res: &mut RenderResources,
    batch: &mut BatchState,
    state: &RenderState,
    tessellate: impl FnOnce(&mut Mesh, Color),
) {
    let Some(color) = state.fill_color else {
        return;
    };
    let material_key = MaterialKey {
        transparent: state.fill_is_transparent(),
        background_image: None,
    };

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
    let material_key = MaterialKey {
        transparent: state.stroke_is_transparent(),
        background_image: None,
    };

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
