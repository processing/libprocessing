//! A graphics object is the core rendering context in Processing, responsible for managing the
//! draw state and recording draw commands to be executed each frame.
//!
//! In Bevy terms, a graphics object is represented as an entity with a camera component
//! configured to render to a specific surface (either a window or an offscreen image).
use bevy::{
    camera::{
        CameraMainTextureUsages, CameraOutputMode, CameraProjection, ClearColorConfig,
        ImageRenderTarget, MsaaWriteback, Projection, RenderTarget, visibility::RenderLayers,
    },
    core_pipeline::tonemapping::Tonemapping,
    ecs::{entity::EntityHashMap, query::QueryEntityError},
    math::{Mat4, Vec3A},
    prelude::*,
    render::{
        Render, RenderSystems,
        render_resource::{
            CommandEncoderDescriptor, Extent3d, MapMode, Origin3d, PollType, TexelCopyBufferInfo,
            TexelCopyBufferLayout, TexelCopyTextureInfo, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        sync_world::MainEntity,
        view::{Hdr, ViewTarget, prepare_view_targets},
    },
    window::WindowRef,
};

use crate::{
    Flush,
    error::{ProcessingError, Result},
    image::{Image, bytes_to_pixels, create_readback_buffer, pixel_size, pixels_to_bytes},
    render::{RenderState, command::DrawCommand},
    surface::Surface,
};

use processing_utils::CommandBuffer;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderLayersManager>();

        let (tx, rx) = crossbeam_channel::unbounded::<(Entity, ViewTarget)>();
        app.init_resource::<GraphicsTargets>()
            .insert_resource(GraphicsTargetReceiver(rx))
            .add_systems(First, update_view_targets);

        let render_app = app.sub_app_mut(bevy::render::RenderApp);
        render_app
            .add_systems(
                Render,
                send_view_targets
                    .in_set(RenderSystems::ManageViews)
                    .after(prepare_view_targets),
            )
            .insert_resource(GraphicsTargetSender(tx));
    }
}

#[derive(Component)]
pub struct Graphics {
    readback_buffer: bevy::render::render_resource::Buffer,
    pub texture_format: TextureFormat,
    pub size: Extent3d,
}

// We store a mapping of graphics target entities to their GPU `ViewTarget`s. In the
// Processing API, graphics *are* images, so we need to be able to look up the `ViewTarget` for a
// given graphics entity when referencing it as an image.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct GraphicsTargets(EntityHashMap<ViewTarget>);

#[derive(Resource, Deref, DerefMut)]
pub struct GraphicsTargetReceiver(crossbeam_channel::Receiver<(Entity, ViewTarget)>);

#[derive(Resource, Deref, DerefMut)]
pub struct GraphicsTargetSender(crossbeam_channel::Sender<(Entity, ViewTarget)>);

fn send_view_targets(
    view_targets: Query<(MainEntity, &ViewTarget), Changed<ViewTarget>>,
    sender: Res<GraphicsTargetSender>,
) {
    for (main_entity, view_target) in view_targets.iter() {
        sender
            .send((main_entity, view_target.clone()))
            .expect("Failed to send updated view target");
    }
}

pub fn update_view_targets(
    mut graphics_targets: ResMut<GraphicsTargets>,
    receiver: Res<GraphicsTargetReceiver>,
) {
    while let Ok((entity, view_target)) = receiver.0.try_recv() {
        graphics_targets.insert(entity, view_target);
    }
}

macro_rules! graphics_mut {
    ($app:expr, $entity:expr) => {
        $app.world_mut()
            .get_entity_mut($entity)
            .map_err(|_| ProcessingError::GraphicsNotFound)?
    };
}

#[derive(Component)]
pub struct SurfaceSize(pub u32, pub u32);

/// Custom orthographic projection for Processing's coordinate system.
/// Origin at top-left, Y-axis down, in pixel units (aka screen space).
#[derive(Debug, Clone, Reflect)]
#[reflect(Default)]
pub struct ProcessingProjection {
    pub width: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for ProcessingProjection {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            near: 0.0,
            far: 1000.0,
        }
    }
}

impl CameraProjection for ProcessingProjection {
    fn get_clip_from_view(&self) -> Mat4 {
        Mat4::orthographic_rh(
            0.0,
            self.width,
            self.height, // bottom = height
            0.0,         // top = 0
            self.near,
            self.far,
        )
    }

    fn get_clip_from_view_for_sub(&self, _sub_view: &bevy::camera::SubCameraView) -> Mat4 {
        // TODO: implement sub-view support if needed (probably not)
        self.get_clip_from_view()
    }

    fn update(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    fn far(&self) -> f32 {
        self.far
    }

    fn get_frustum_corners(&self, z_near: f32, z_far: f32) -> [Vec3A; 8] {
        // order: bottom-right, top-right, top-left, bottom-left for near, then far
        let near_center = Vec3A::new(self.width / 2.0, self.height / 2.0, z_near);
        let far_center = Vec3A::new(self.width / 2.0, self.height / 2.0, z_far);

        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;

        [
            // near plane
            near_center + Vec3A::new(half_width, half_height, 0.0), // bottom-right
            near_center + Vec3A::new(half_width, -half_height, 0.0), // top-right
            near_center + Vec3A::new(-half_width, -half_height, 0.0), // top-left
            near_center + Vec3A::new(-half_width, half_height, 0.0), // bottom-left
            // far plane
            far_center + Vec3A::new(half_width, half_height, 0.0), // bottom-right
            far_center + Vec3A::new(half_width, -half_height, 0.0), // top-right
            far_center + Vec3A::new(-half_width, -half_height, 0.0), // top-left
            far_center + Vec3A::new(-half_width, half_height, 0.0), // bottom-left
        ]
    }
}

pub fn create(
    In((width, height, surface_entity)): In<(u32, u32, Entity)>,
    mut commands: Commands,
    mut layer_manager: ResMut<RenderLayersManager>,
    p_images: Query<&Image, With<Surface>>,
    render_device: Res<RenderDevice>,
) -> Result<Entity> {
    // find the surface entity, if it is an image, we will render to that image
    // otherwise we will render to the window
    let target = match p_images.get(surface_entity) {
        Ok(p_image) => RenderTarget::Image(ImageRenderTarget::from(p_image.handle.clone())),
        Err(QueryEntityError::QueryDoesNotMatch(..)) => {
            RenderTarget::Window(WindowRef::Entity(surface_entity))
        }
        Err(_) => return Err(ProcessingError::SurfaceNotFound),
    };
    // allocate a new render layer for this graphics entity, which ensures that anything
    // drawn to this camera will only be visible to this camera
    let render_layer = layer_manager.allocate();

    // TODO: make this configurable, right now we are hard-coding hdr camera
    let texture_format = TextureFormat::Rgba16Float;
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let readback_buffer = create_readback_buffer(
        &render_device,
        width,
        height,
        texture_format,
        "Graphics Readback Buffer",
    )
    .expect("Failed to create readback buffer");

    let entity = commands
        .spawn((
            Camera3d::default(),
            Camera {
                target,
                // always load the previous frame (provides sketch like behavior)
                clear_color: ClearColorConfig::None,
                // TODO: toggle this conditionally based on whether we need to write back MSAA
                // when doing manual pixel udpates
                msaa_writeback: MsaaWriteback::Always,
                ..default()
            },
            // default to floating point texture format
            Hdr,
            // tonemapping prevents color accurate readback, so we disable it
            Tonemapping::None,
            // we need to be able to write to the texture
            CameraMainTextureUsages::default().with(TextureUsages::COPY_DST),
            Projection::custom(ProcessingProjection {
                width: width as f32,
                height: height as f32,
                near: 0.0,
                far: 1000.0,
            }),
            Transform::from_xyz(0.0, 0.0, 999.9),
            render_layer,
            CommandBuffer::<DrawCommand>::new(),
            RenderState::default(),
            SurfaceSize(width, height),
            Graphics {
                readback_buffer,
                texture_format,
                size,
            },
        ))
        .id();

    Ok(entity)
}

#[allow(dead_code)]
pub fn resize(
    In((entity, width, height)): In<(Entity, u32, u32)>,
    mut graphics_query: Query<&mut Projection>,
) -> Result<()> {
    let mut projection = graphics_query
        .get_mut(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    if let Projection::Custom(ref mut custom_proj) = *projection {
        custom_proj.update(width as f32, height as f32);
        Ok(())
    } else {
        panic!(
            "Expected custom projection for Processing graphics entity, this should not happen. If you are seeing this message, please report a bug."
        );
    }
}

pub fn mode_3d(
    In(entity): In<Entity>,
    mut projections: Query<&mut Projection>,
    mut transforms: Query<&mut Transform>,
    sizes: Query<&SurfaceSize>,
) -> Result<()> {
    let SurfaceSize(width, height) = sizes
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    let width = *width as f32;
    let height = *height as f32;

    let fov = std::f32::consts::PI / 3.0; // 60 degrees
    let aspect = width / height;
    let camera_z = (height / 2.0) / (fov / 2.0).tan();
    let near = camera_z / 10.0;
    let far = camera_z * 10.0;

    let mut projection = projections
        .get_mut(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    *projection = Projection::Perspective(PerspectiveProjection {
        fov,
        aspect_ratio: aspect,
        near,
        far,
    });

    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    *transform = Transform::from_xyz(0.0, 0.0, camera_z).looking_at(Vec3::ZERO, Vec3::Y);

    Ok(())
}

pub fn mode_2d(
    In(entity): In<Entity>,
    mut projections: Query<&mut Projection>,
    mut transforms: Query<&mut Transform>,
    sizes: Query<&SurfaceSize>,
) -> Result<()> {
    let SurfaceSize(width, height) = sizes
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    let mut projection = projections
        .get_mut(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    *projection = Projection::custom(ProcessingProjection {
        width: *width as f32,
        height: *height as f32,
        near: 0.0,
        far: 1000.0,
    });

    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    *transform = Transform::from_xyz(0.0, 0.0, 999.9);

    Ok(())
}

pub fn perspective(
    In((
        entity,
        PerspectiveProjection {
            fov,
            aspect_ratio,
            near,
            far,
        },
    )): In<(Entity, PerspectiveProjection)>,
    mut projections: Query<&mut Projection>,
) -> Result<()> {
    let mut projection = projections
        .get_mut(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    *projection = Projection::Perspective(PerspectiveProjection {
        fov,
        aspect_ratio,
        near,
        far,
    });

    Ok(())
}

pub struct OrthoArgs {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

pub fn ortho(
    In((
        entity,
        OrthoArgs {
            left,
            right,
            bottom,
            top,
            near,
            far,
        },
    )): In<(Entity, OrthoArgs)>,
    mut projections: Query<&mut Projection>,
) -> Result<()> {
    let mut projection = projections
        .get_mut(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    // we need a custom projection to support processing's coordinate system
    // but this is in effect an orthographic projection with the given bounds
    *projection = Projection::custom(ProcessingProjection {
        width: right - left,
        height: top - bottom,
        near,
        far,
    });

    Ok(())
}

pub fn destroy(
    In(entity): In<Entity>,
    mut commands: Commands,
    mut layer_manager: ResMut<RenderLayersManager>,
    graphics_query: Query<&RenderLayers>,
) -> Result<()> {
    let Ok(render_layers) = graphics_query.get(entity) else {
        return Err(ProcessingError::GraphicsNotFound);
    };

    layer_manager.free(render_layers.clone());
    commands.entity(entity).despawn();
    Ok(())
}

pub fn begin_draw(In(entity): In<Entity>, mut state_query: Query<&mut RenderState>) -> Result<()> {
    let mut state = state_query
        .get_mut(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;
    state.reset();
    Ok(())
}

pub fn flush(app: &mut App, entity: Entity) -> Result<()> {
    graphics_mut!(app, entity).insert(Flush);
    app.update();
    graphics_mut!(app, entity).remove::<Flush>();
    // ensure graphics targets are available immediately after flush
    app.world_mut()
        .run_system_cached(update_view_targets)
        .expect("Failed to run update_view_targets");

    Ok(())
}

pub fn end_draw(app: &mut App, entity: Entity) -> Result<()> {
    // Enable output to present the frame, but don't clear (preserve pixel writes)
    graphics_mut!(app, entity)
        .get_mut::<Camera>()
        .ok_or(ProcessingError::GraphicsNotFound)?
        .output_mode = CameraOutputMode::Write {
        blend_state: None,
        clear_color: ClearColorConfig::None,
    };
    // flush any remaining draw commands, this ensures that the frame is presented even if there
    // is no remaining draw commands
    flush(app, entity)?;
    graphics_mut!(app, entity)
        .get_mut::<Camera>()
        .ok_or(ProcessingError::GraphicsNotFound)?
        .output_mode = CameraOutputMode::Skip;
    Ok(())
}

pub fn record_command(
    In((graphics_entity, cmd)): In<(Entity, DrawCommand)>,
    mut graphics_query: Query<&mut CommandBuffer<DrawCommand>>,
) -> Result<()> {
    let mut command_buffer = graphics_query
        .get_mut(graphics_entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    command_buffer.push(cmd);
    Ok(())
}

pub fn readback(
    In(entity): In<Entity>,
    graphics_query: Query<&Graphics>,
    graphics_targets: Res<GraphicsTargets>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) -> Result<Vec<LinearRgba>> {
    let graphics = graphics_query
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    let view_target = graphics_targets
        .get(&entity)
        .ok_or(ProcessingError::GraphicsNotFound)?;

    let texture = view_target.main_texture();

    let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());

    let px_size = pixel_size(graphics.texture_format)?;
    let padded_bytes_per_row =
        RenderDevice::align_copy_bytes_per_row(graphics.size.width as usize * px_size);

    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        TexelCopyBufferInfo {
            buffer: &graphics.readback_buffer,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(
                    std::num::NonZero::<u32>::new(padded_bytes_per_row as u32)
                        .unwrap()
                        .into(),
                ),
                rows_per_image: None,
            },
        },
        graphics.size,
    );

    render_queue.submit(std::iter::once(encoder.finish()));

    let buffer_slice = graphics.readback_buffer.slice(..);

    let (s, r) = crossbeam_channel::bounded(1);

    buffer_slice.map_async(MapMode::Read, move |r| match r {
        Ok(r) => s.send(r).expect("Failed to send map update"),
        Err(err) => panic!("Failed to map buffer {err}"),
    });

    render_device
        .poll(PollType::Wait)
        .expect("Failed to poll device for map async");

    r.recv().expect("Failed to receive the map_async message");

    let data = buffer_slice.get_mapped_range().to_vec();

    graphics.readback_buffer.unmap();

    bytes_to_pixels(
        &data,
        graphics.texture_format,
        graphics.size.width,
        graphics.size.height,
        padded_bytes_per_row,
    )
}

pub fn update_region_write(
    In((entity, x, y, width, height, data, px_size)): In<(
        Entity,
        u32,
        u32,
        u32,
        u32,
        Vec<u8>,
        u32,
    )>,
    graphics_query: Query<&Graphics>,
    graphics_targets: Res<GraphicsTargets>,
    render_queue: Res<RenderQueue>,
) -> Result<()> {
    let graphics = graphics_query
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    // bounds check
    if x + width > graphics.size.width || y + height > graphics.size.height {
        return Err(ProcessingError::InvalidArgument(format!(
            "Region ({}, {}, {}, {}) exceeds graphics bounds ({}, {})",
            x, y, width, height, graphics.size.width, graphics.size.height
        )));
    }

    let view_target = graphics_targets
        .get(&entity)
        .ok_or(ProcessingError::GraphicsNotFound)?;

    let texture = view_target.main_texture();
    eprintln!(
        "update_region: writing to texture {:p} at ({}, {}) size {}x{}",
        texture as *const _, x, y, width, height
    );
    let bytes_per_row = width * px_size;

    render_queue.write_texture(
        TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: Origin3d { x, y, z: 0 },
            aspect: Default::default(),
        },
        &data,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: None,
        },
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    Ok(())
}

pub fn prepare_update_region(
    world: &World,
    entity: Entity,
    width: u32,
    height: u32,
    pixels: &[LinearRgba],
) -> Result<(Vec<u8>, u32)> {
    let expected_count = (width * height) as usize;
    if pixels.len() != expected_count {
        return Err(ProcessingError::InvalidArgument(format!(
            "Expected {} pixels for {}x{} region, got {}",
            expected_count,
            width,
            height,
            pixels.len()
        )));
    }

    let graphics = world
        .get::<Graphics>(entity)
        .ok_or(ProcessingError::GraphicsNotFound)?;
    let px_size = pixel_size(graphics.texture_format)? as u32;
    let data = pixels_to_bytes(pixels, graphics.texture_format)?;

    Ok((data, px_size))
}

#[derive(Resource, Debug, Clone, Reflect)]
pub struct RenderLayersManager {
    used: RenderLayers,
    next_free: usize,
}

impl Default for RenderLayersManager {
    fn default() -> Self {
        RenderLayersManager {
            used: RenderLayers::none(),
            next_free: 1,
        }
    }
}

impl RenderLayersManager {
    pub fn allocate(&mut self) -> RenderLayers {
        let layer = self.next_free;
        if layer >= Self::max_layer() {
            // if the user is hitting this limit, they are probably doing something wrong
            // as this is a very large number of layers that would likely cause serious
            // performance issues long before reaching this point
            panic!(
                "Exceeded maximum number of render layers, this should not happen. If you are seeing this message, please report a bug."
            );
        }

        self.used = self.used.clone().with(layer);

        self.next_free = (layer + 1..Self::max_layer())
            .find(|&l| !self.is_used(l))
            .unwrap_or(Self::max_layer());

        RenderLayers::none().with(layer)
    }

    pub fn free(&mut self, layers: RenderLayers) {
        for layer in layers.iter() {
            if layer == 0 {
                continue;
            }
            self.used = self.used.clone().without(layer);
            if layer < self.next_free {
                self.next_free = layer;
            }
        }
    }

    pub fn is_used(&self, layer: usize) -> bool {
        let single = RenderLayers::none().with(layer);
        self.used.intersects(&single)
    }

    const fn max_layer() -> usize {
        // an arbitrary limit, in theory we could keep going forever but
        // if we reach this point something is probably wrong
        4096
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_projection() {
        let proj = ProcessingProjection {
            width: 800.0,
            height: 600.0,
            near: 0.1,
            far: 1000.0,
        };
        let clip_matrix = proj.get_clip_from_view();
        // Check some values in the matrix to ensure it's correct
        // In [0,1] depth orthographic projection, w_axis.z = -near/(far-near)
        let expected = -0.1 / (1000.0 - 0.1);
        assert!((clip_matrix.w_axis.z - expected).abs() < 1e-6);
    }

    #[test]
    fn test_layer_reservation() {
        let mut manager = RenderLayersManager::default();
        let layer1 = manager.allocate();
        let layer1_clone = layer1.clone();
        let layer2 = manager.allocate();
        assert_ne!(layer1, layer2);
        manager.free(layer1);
        let layer3 = manager.allocate();
        assert_eq!(layer1_clone, layer3);
    }
}
