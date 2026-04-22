//! A graphics object is the core rendering context in Processing, responsible for managing the
//! draw state and recording draw commands to be executed each frame.
//!
//! In Bevy terms, a graphics object is represented as an entity with a camera component
//! configured to render to a specific surface (either a window or an offscreen image).
use bevy::{
    camera::{
        CameraMainTextureUsages, CameraOutputMode, CameraProjection, ClearColorConfig, Hdr,
        ImageRenderTarget, MsaaWriteback, Projection, RenderTarget, visibility::RenderLayers,
    },
    core_pipeline::tonemapping::Tonemapping,
    ecs::query::QueryEntityError,
    math::{Mat4, Vec3A},
    prelude::*,
    render::{
        RenderApp,
        render_resource::{
            CommandEncoderDescriptor, Extent3d, MapMode, Origin3d, PollType, TexelCopyBufferInfo,
            TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        sync_world::MainEntity,
        view::ViewTarget,
    },
    window::WindowRef,
};

use crate::{
    Flush,
    image::{Image, create_readback_buffer, pixel_size, pixels_to_bytes},
    render::{
        BATCH_INDEX_STEP, RenderState,
        command::{CommandBuffer, DrawCommand},
    },
    surface::Surface,
};
use processing_core::error::{ProcessingError, Result};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderLayersManager>()
            .add_systems(PostUpdate, sync_to_surface);
    }
}

#[derive(Component)]
pub struct Graphics {
    readback_buffer: bevy::render::render_resource::Buffer,
    pub texture_format: TextureFormat,
    pub size: Extent3d,
}

pub fn view_target(app: &mut App, entity: Entity) -> Result<&ViewTarget> {
    let rw = app.sub_app_mut(RenderApp).world_mut();
    let mut query = rw.query::<(&MainEntity, &ViewTarget)>();
    for (main_entity, vt) in query.iter(rw) {
        if **main_entity == entity {
            return Ok(vt);
        }
    }
    Err(ProcessingError::GraphicsNotFound)
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
pub struct ProcessingProjection {
    pub width: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
}

impl ProcessingProjection {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
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

    fn update(&mut self, _width: f32, _height: f32) {
        // this gets called with the render target's physical dimensions (i.e. accounting for
        // scale factor), but our projection is in logical pixel units
        // TODO: handle resizes?
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
    In((width, height, surface_entity, texture_format)): In<(u32, u32, Entity, TextureFormat)>,
    mut commands: Commands,
    mut layer_manager: ResMut<RenderLayersManager>,
    p_images: Query<&Image, With<Surface>>,
    windows: Query<&Window, With<Surface>>,
    render_device: Res<RenderDevice>,
) -> Result<Entity> {
    // find the surface entity, if it is an image, we will render to that image
    // otherwise we will render to the window
    let (target, physical_width, physical_height) = match p_images.get(surface_entity) {
        Ok(p_image) => (
            RenderTarget::Image(ImageRenderTarget::from(p_image.handle.clone())),
            p_image.size.width,
            p_image.size.height,
        ),
        Err(QueryEntityError::QueryDoesNotMatch(..)) => {
            let window = windows
                .get(surface_entity)
                .map_err(|_| ProcessingError::SurfaceNotFound)?;
            (
                RenderTarget::Window(WindowRef::Entity(surface_entity)),
                window.resolution.physical_width(),
                window.resolution.physical_height(),
            )
        }
        Err(_) => return Err(ProcessingError::SurfaceNotFound),
    };
    // allocate a new render layer for this graphics entity, which ensures that anything
    // drawn to this camera will only be visible to this camera
    let render_layer = layer_manager.allocate();

    let size = Extent3d {
        width: physical_width,
        height: physical_height,
        depth_or_array_layers: 1,
    };
    let readback_buffer = create_readback_buffer(
        &render_device,
        physical_width,
        physical_height,
        texture_format,
        "Graphics Readback Buffer",
    )
    .expect("Failed to create readback buffer");

    let is_hdr = matches!(
        texture_format,
        TextureFormat::Rgba16Float | TextureFormat::Rgba32Float
    );

    let mut entity_commands = commands.spawn((
        Camera3d::default(),
        Camera {
            // always load the previous frame (provides sketch like behavior)
            clear_color: ClearColorConfig::None,
            msaa_writeback: MsaaWriteback::Auto,
            ..default()
        },
        target,
        // tonemapping prevents color accurate readback, so we disable it
        Tonemapping::None,
        // we need to be able to write to the texture
        CameraMainTextureUsages::default().with(TextureUsages::COPY_DST),
        Projection::custom(ProcessingProjection::new(width as f32, height as f32)),
        Transform::from_xyz(0.0, 0.0, BATCH_INDEX_STEP),
        render_layer,
        CommandBuffer::new(),
        RenderState::default(),
        crate::color::ColorMode::default(),
        SurfaceSize(width, height),
        Graphics {
            readback_buffer,
            texture_format,
            size,
        },
    ));

    // only enable Hdr for floating-point texture formats
    if is_hdr {
        entity_commands.insert(Hdr);
    }

    let entity = entity_commands.id();

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

pub fn sync_to_surface(
    mut graphics_query: Query<(&mut Graphics, &RenderTarget)>,
    windows: Query<&Window, (With<Surface>, Changed<Window>)>,
    render_device: Res<RenderDevice>,
) {
    for (mut graphics, target) in graphics_query.iter_mut() {
        let RenderTarget::Window(WindowRef::Entity(surface_entity)) = *target else {
            continue;
        };
        let Ok(window) = windows.get(surface_entity) else {
            continue;
        };
        let physical_w = window.resolution.physical_width();
        let physical_h = window.resolution.physical_height();
        if graphics.size.width == physical_w && graphics.size.height == physical_h {
            continue;
        }
        graphics.size = Extent3d {
            width: physical_w,
            height: physical_h,
            depth_or_array_layers: 1,
        };
        graphics.readback_buffer = create_readback_buffer(
            &render_device,
            physical_w,
            physical_h,
            graphics.texture_format,
            "Graphics Readback Buffer",
        )
        .expect("Failed to reallocate readback buffer");
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

    // TODO: Setting this as a default, but we need to think about API around
    // a user defined value
    let near_clip_plane = vec4(0.0, 0.0, -1.0, -near);

    let mut projection = projections
        .get_mut(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    *projection = Projection::Perspective(PerspectiveProjection {
        fov,
        aspect_ratio: aspect,
        near,
        far,
        near_clip_plane,
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

    *projection = Projection::custom(ProcessingProjection::new(*width as f32, *height as f32));

    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    *transform = Transform::from_xyz(0.0, 0.0, BATCH_INDEX_STEP);

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
            near_clip_plane,
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
        near_clip_plane,
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
    state.begin_frame();
    Ok(())
}

pub fn flush(app: &mut App, entity: Entity) -> Result<()> {
    // f there's nothing to render, skip the whole render pass. this avoids some issues on
    // macos with msaa resolve where nothing is rendered
    let is_empty = graphics_mut!(app, entity)
        .get::<CommandBuffer>()
        .map(|c| c.commands.is_empty())
        .unwrap_or(true);
    if is_empty {
        return Ok(());
    }

    graphics_mut!(app, entity).insert(Flush);
    app.update();
    graphics_mut!(app, entity).remove::<Flush>();
    Ok(())
}

pub fn present(app: &mut App, entity: Entity) -> Result<()> {
    graphics_mut!(app, entity)
        .get_mut::<Camera>()
        .ok_or(ProcessingError::GraphicsNotFound)?
        .output_mode = CameraOutputMode::Write {
        blend_state: None,
        clear_color: ClearColorConfig::None,
    };
    flush(app, entity)?;
    graphics_mut!(app, entity)
        .get_mut::<Camera>()
        .ok_or(ProcessingError::GraphicsNotFound)?
        .output_mode = CameraOutputMode::Skip;

    Ok(())
}

/// End the current draw
pub fn end_draw(app: &mut App, entity: Entity) -> Result<()> {
    present(app, entity)
}

pub fn record_command(
    In((graphics_entity, cmd)): In<(Entity, DrawCommand)>,
    mut graphics_query: Query<&mut CommandBuffer>,
) -> Result<()> {
    let mut command_buffer = graphics_query
        .get_mut(graphics_entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    command_buffer.push(cmd);
    Ok(())
}

pub struct ReadbackData {
    pub bytes: Vec<u8>,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
}

pub fn readback_raw(
    In((entity, texture)): In<(Entity, Texture)>,
    graphics_query: Query<&Graphics>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) -> Result<ReadbackData> {
    let graphics = graphics_query
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

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
        .poll(PollType::wait_indefinitely())
        .expect("Failed to poll device for map async");

    r.recv().expect("Failed to receive the map_async message");

    let data = buffer_slice.get_mapped_range().to_vec();

    graphics.readback_buffer.unmap();

    // strip row padding
    let bytes_per_row = graphics.size.width as usize * px_size;
    let unpadded = if padded_bytes_per_row != bytes_per_row {
        data.chunks_exact(padded_bytes_per_row)
            .take(graphics.size.height as usize)
            .flat_map(|row| &row[..bytes_per_row])
            .copied()
            .collect()
    } else {
        data
    };

    Ok(ReadbackData {
        bytes: unpadded,
        format: graphics.texture_format,
        width: graphics.size.width,
        height: graphics.size.height,
    })
}

pub fn update_region_write(
    In((entity, texture, x, y, width, height, data, px_size)): In<(
        Entity,
        Texture,
        u32,
        u32,
        u32,
        u32,
        Vec<u8>,
        u32,
    )>,
    graphics_query: Query<&Graphics>,
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
    let bytes_per_row = width * px_size;

    render_queue.write_texture(
        TexelCopyTextureInfo {
            texture: &texture,
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
        let expected: f32 = -0.1 / (1000.0 - 0.1);
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
