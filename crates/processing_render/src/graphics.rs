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
    post_process::bloom::Bloom,
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
        filter,
    },
    surface::Surface,
};
use processing_core::error::{ProcessingError, Result};

pub const DEFAULT_CLEAR_COLOR: Color = Color::srgba_u8(208, 208, 208, 255);

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
        self.width = _width;
        self.height = _height;
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

/// Off-axis (asymmetric) perspective projection — classic OpenGL/Processing
/// `frustum()`. `left`/`right`/`bottom`/`top` bound the view volume on the
/// near plane, in view-space units.
#[derive(Debug, Clone, Reflect)]
pub struct FrustumProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl CameraProjection for FrustumProjection {
    fn get_clip_from_view(&self) -> Mat4 {
        // The standard glFrustum matrix, except depth is REVERSED (near → 1,
        // far → 0) to match the rest of the pipeline — bevy's perspective is
        // infinite-reverse and its ortho swaps near/far for the same reason.
        // An ortho can reverse by swapping near/far in the standard formula;
        // a frustum cannot (near also sets the x/y scale), so the z column is
        // derived directly: it hits 1 at z_view = −near and 0 at z_view = −far.
        let (l, r, b, t, n, f) = (
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        );
        Mat4::from_cols(
            Vec4::new(2.0 * n / (r - l), 0.0, 0.0, 0.0),
            Vec4::new(0.0, 2.0 * n / (t - b), 0.0, 0.0),
            Vec4::new((r + l) / (r - l), (t + b) / (t - b), n / (f - n), -1.0),
            Vec4::new(0.0, 0.0, n * f / (f - n), 0.0),
        )
    }

    fn get_clip_from_view_for_sub(&self, _sub_view: &bevy::camera::SubCameraView) -> Mat4 {
        self.get_clip_from_view()
    }

    fn update(&mut self, _width: f32, _height: f32) {
        // Explicit view-volume bounds don't track the window size.
    }

    fn far(&self) -> f32 {
        self.far
    }

    fn get_frustum_corners(&self, z_near: f32, z_far: f32) -> [Vec3A; 8] {
        // The near-plane bounds scale linearly with distance along the rays.
        let sn = z_near.abs() / self.near;
        let sf = z_far.abs() / self.near;
        [
            // near plane
            Vec3A::new(self.right * sn, self.bottom * sn, z_near), // bottom-right
            Vec3A::new(self.right * sn, self.top * sn, z_near),    // top-right
            Vec3A::new(self.left * sn, self.top * sn, z_near),     // top-left
            Vec3A::new(self.left * sn, self.bottom * sn, z_near),  // bottom-left
            // far plane
            Vec3A::new(self.right * sf, self.bottom * sf, z_far), // bottom-right
            Vec3A::new(self.right * sf, self.top * sf, z_far),    // top-right
            Vec3A::new(self.left * sf, self.top * sf, z_far),     // top-left
            Vec3A::new(self.left * sf, self.bottom * sf, z_far),  // bottom-left
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
        // overridden below for hdr targets
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

    if is_hdr {
        entity_commands.insert((Hdr, Bloom::NATURAL, Tonemapping::TonyMcMapface));
    }

    // TEMP(bevy-020 debug): force direct drawing to bisect the indirect-args
    // build for sorted instance batches.
    if std::env::var("PROCESSING_NO_INDIRECT").is_ok() {
        entity_commands.insert(bevy::render::view::NoIndirectDrawing);
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
    // Processing4 uses near = camera_z / 10, but that clips anything closer
    // than ~camera_z/10 to the camera. Since `transform_set_position` lets the
    // user move the camera without recomputing the projection, a small fixed
    // near is safer and matches most engines' defaults.
    let near = 1.0;
    let far = camera_z * 10.0;
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
            near_clip_plane: _,
        },
    )): In<(Entity, PerspectiveProjection)>,
    mut projections: Query<&mut Projection>,
) -> Result<()> {
    let mut projection = projections
        .get_mut(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    // An explicit perspective() must honor ALL its parameters — including a
    // deliberately anamorphic aspect ratio. Bevy's PerspectiveProjection
    // cannot: its update() overwrites aspect_ratio with the viewport aspect
    // on every render-area pass, silently discarding the caller's value. So
    // express the same view volume as the equivalent symmetric frustum,
    // whose update() is inert. The window-tracking default stays with
    // mode_3d's Projection::Perspective.
    let top = near * (fov / 2.0).tan();
    let right = top * aspect_ratio;
    *projection = Projection::custom(FrustumProjection {
        left: -right,
        right,
        bottom: -top,
        top,
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

pub struct FrustumArgs {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

/// glFrustum-style asymmetric perspective: the bounds are on the NEAR plane.
pub fn frustum(
    In((
        entity,
        FrustumArgs {
            left,
            right,
            bottom,
            top,
            near,
            far,
        },
    )): In<(Entity, FrustumArgs)>,
    mut projections: Query<&mut Projection>,
) -> Result<()> {
    let mut projection = projections
        .get_mut(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    if !(near > 0.0 && far > near && right != left && top != bottom) {
        return Err(ProcessingError::InvalidArgument(format!(
            "frustum requires 0 < near < far and non-empty bounds, \
             got left={left} right={right} bottom={bottom} top={top} near={near} far={far}"
        )));
    }

    *projection = Projection::custom(FrustumProjection {
        left,
        right,
        bottom,
        top,
        near,
        far,
    });

    Ok(())
}

pub fn world_from_screen(
    In((entity, sx, sy, depth)): In<(Entity, f32, f32, f32)>,
    cameras: Query<(&bevy::camera::Camera, &GlobalTransform)>,
) -> Result<Vec3> {
    let (camera, transform) = cameras
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    let ndc_xy = camera
        .viewport_to_ndc(Vec2::new(sx, sy))
        .map_err(|_| ProcessingError::GraphicsNotFound)?;
    let ndc_z = (1.0 - depth).max(f32::EPSILON);
    let world: Vec3 = camera
        .ndc_to_world(transform, ndc_xy.extend(ndc_z))
        .ok_or(ProcessingError::GraphicsNotFound)?;
    Ok(world)
}

pub fn set_bloom(
    In((entity, intensity, threshold)): In<(Entity, f32, f32)>,
    mut commands: Commands,
    mut tonemapping_query: Query<&mut Tonemapping>,
) -> Result<()> {
    use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};

    let mut bloom = Bloom::NATURAL;
    bloom.intensity = intensity;
    if threshold > 0.0 {
        bloom.composite_mode = BloomCompositeMode::Additive;
        bloom.prefilter = BloomPrefilter {
            threshold,
            threshold_softness: 0.5,
        };
    }

    commands.entity(entity).insert((bloom, Hdr));

    if let Ok(mut tm) = tonemapping_query.get_mut(entity) {
        if *tm == Tonemapping::None {
            *tm = Tonemapping::TonyMcMapface;
        }
    }

    Ok(())
}

pub fn remove_bloom(
    In(entity): In<Entity>,
    mut commands: Commands,
    mut tonemapping_query: Query<&mut Tonemapping>,
) -> Result<()> {
    use bevy::post_process::bloom::Bloom;

    commands.entity(entity).remove::<Bloom>();

    if let Ok(mut tm) = tonemapping_query.get_mut(entity) {
        *tm = Tonemapping::None;
    }

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

pub fn get_matrix(In(entity): In<Entity>, states: Query<&RenderState>) -> Result<Mat4> {
    let state = states
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;
    Ok(Mat4::from(state.transform.current()))
}

pub fn model_point(
    In((entity, point)): In<(Entity, Vec3)>,
    states: Query<&RenderState>,
) -> Result<Vec3> {
    let state = states
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;
    Ok(state.transform.transform_point(point))
}

pub fn screen_point(
    In((entity, point)): In<(Entity, Vec3)>,
    query: Query<(&RenderState, &Projection, &Transform, &SurfaceSize)>,
) -> Result<Vec3> {
    let (state, projection, camera_transform, size) = query
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;
    let world = state.transform.transform_point(point);
    let clip_from_view = projection.get_clip_from_view();
    let view_from_world = camera_transform.to_matrix().inverse();
    let clip = clip_from_view * view_from_world * world.extend(1.0);
    if clip.w == 0.0 {
        return Ok(Vec3::ZERO);
    }
    let ndc = clip.truncate() / clip.w;
    let SurfaceSize(width, height) = *size;
    Ok(Vec3::new(
        (ndc.x + 1.0) * 0.5 * width as f32,
        (1.0 - ndc.y) * 0.5 * height as f32,
        ndc.z,
    ))
}

pub fn flush(app: &mut App, entity: Entity) -> Result<()> {
    graphics_mut!(app, entity).insert(Flush);
    app.update();
    graphics_mut!(app, entity).remove::<Flush>();
    Ok(())
}

pub fn apply_filter(app: &mut App, graphics: Entity, filter: Entity) -> Result<()> {
    filter::apply(app, graphics, filter)
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

/// Do some work on the GPU to ensure that the render target texture is initialized and can be read
/// from/written to.
///
/// This is necessary on some platforms (notably macOS) to avoid issues with the first few frames of
/// rendering being corrupted or not appearing at all.
///
// TODO: why is metal particularly affected by this? can we remove this?
pub fn warmup(app: &mut App, entity: Entity) -> Result<()> {
    for _ in 0..3 {
        app.world_mut()
            .run_system_cached_with(
                record_command,
                (entity, DrawCommand::BackgroundColor(DEFAULT_CLEAR_COLOR)),
            )
            .unwrap()?;
        flush(app, entity)?;
    }
    Ok(())
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

    let data = buffer_slice
        .get_mapped_range()
        .expect("Failed to get mapped range for readback")
        .to_vec();

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

// ── Asynchronous readback ring ──────────────────────────────────────────────
// The synchronous readback above stalls the caller for a full pipeline flush
// plus the GPU→CPU copy every call — at 3840×4320 that is most of a frame's
// wall time and dominates recording. The ring splits the operation: `enqueue`
// submits the copy and returns immediately; `fetch` hands back frames whose
// map has completed, in submission order. With a few frames in flight the
// copy of frame N overlaps the render of frame N+1 and readback mostly
// leaves the critical path. Consumers that tolerate a frame or two of
// latency (the video recorder) use this; point queries keep the sync path.

/// Frames allowed in flight before `readback_ring_enqueue` refuses. Bounds
/// memory (one full frame each — ~66MB at 3840×4320) and bounds how far the
/// producer can run ahead.
pub const READBACK_RING_DEPTH: usize = 3;

struct PendingReadback {
    buffer: bevy::render::render_resource::Buffer,
    ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// The copy's own submission, so a blocking fetch waits for exactly this
    /// copy — waiting for "the most recent submission" would drain the whole
    /// queue (including newer copies and the in-flight render) and serialize
    /// the pipeline into burst-stall cycles.
    submission: wgpu::SubmissionIndex,
    padded_bytes_per_row: usize,
    bytes_per_row: usize,
    format: TextureFormat,
    width: u32,
    height: u32,
}

/// GPU-side format conversion in front of the readback: HDR (or any
/// non-8-bit) surfaces are blitted into an 8-bit sRGB texture BEFORE the
/// copy, so the readback moves half the bytes and the CPU never touches a
/// float pixel. (A 3840×4320 Rgba16Float frame is 132MB and a per-pixel CPU
/// conversion of it costs ~350ms — it, not the copy, was the recording
/// bottleneck.)
struct ConvertPass {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    blitter: wgpu::util::TextureBlitter,
    width: u32,
    height: u32,
}

/// Per-graphics async readback state; created lazily on first enqueue.
#[derive(Component, Default)]
pub struct ReadbackRing {
    free: Vec<(bevy::render::render_resource::Buffer, u64)>,
    pending: std::collections::VecDeque<PendingReadback>,
    convert: Option<ConvertPass>,
}

pub fn readback_ring_enqueue(
    In((entity, texture)): In<(Entity, Texture)>,
    world: &mut World,
) -> Result<()> {
    let render_device = world.resource::<RenderDevice>().clone();
    let render_queue = world.resource::<RenderQueue>().clone();
    let (source_format, size) = {
        let graphics = world
            .get::<Graphics>(entity)
            .ok_or(ProcessingError::GraphicsNotFound)?;
        (graphics.texture_format, graphics.size)
    };
    // 8-bit surfaces read back as they are; everything else converts on the
    // GPU to 8-bit sRGB first (see ConvertPass).
    let needs_convert = !matches!(
        source_format,
        TextureFormat::Rgba8UnormSrgb
            | TextureFormat::Rgba8Unorm
            | TextureFormat::Bgra8UnormSrgb
            | TextureFormat::Bgra8Unorm
    );
    let format = if needs_convert {
        TextureFormat::Rgba8UnormSrgb
    } else {
        source_format
    };
    let px_size = pixel_size(format)?;
    let bytes_per_row = size.width as usize * px_size;
    let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(bytes_per_row);
    let buffer_size = padded_bytes_per_row as u64 * size.height as u64;

    if world.get::<ReadbackRing>(entity).is_none() {
        world.entity_mut(entity).insert(ReadbackRing::default());
    }
    {
        let ring = world.get::<ReadbackRing>(entity).unwrap();
        if ring.pending.len() >= READBACK_RING_DEPTH {
            return Err(ProcessingError::InvalidArgument(
                "readback ring full; fetch completed frames first".to_string(),
            ));
        }
    }

    // Reuse a free buffer of the right size; a resize just retires old ones.
    let buffer = {
        let mut ring = world.get_mut::<ReadbackRing>(entity).unwrap();
        ring.free.retain(|(_, sz)| *sz == buffer_size);
        match ring.free.pop() {
            Some((b, _)) => b,
            None => create_readback_buffer(
                &render_device,
                size.width,
                size.height,
                format,
                "async readback ring buffer",
            )?,
        }
    };

    if needs_convert {
        let stale = match world.get::<ReadbackRing>(entity).unwrap().convert {
            Some(ref c) => c.width != size.width || c.height != size.height,
            None => true,
        };
        if stale {
            let device = render_device.wgpu_device();
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("async readback convert target"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
            let blitter =
                wgpu::util::TextureBlitter::new(device, TextureFormat::Rgba8UnormSrgb);
            world.get_mut::<ReadbackRing>(entity).unwrap().convert = Some(ConvertPass {
                texture: tex,
                view,
                blitter,
                width: size.width,
                height: size.height,
            });
        }
    }

    let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());
    let copy_src = if needs_convert {
        let src_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let ring = world.get::<ReadbackRing>(entity).unwrap();
        let convert = ring.convert.as_ref().unwrap();
        convert
            .blitter
            .copy(render_device.wgpu_device(), &mut encoder, &src_view, &convert.view);
        convert.texture.clone()
    } else {
        // bevy's Texture derefs to the raw wgpu texture.
        wgpu::Texture::clone(&texture)
    };
    encoder.copy_texture_to_buffer(
        copy_src.as_image_copy(),
        TexelCopyBufferInfo {
            buffer: &buffer,
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
        size,
    );
    let submission = render_queue.submit(std::iter::once(encoder.finish()));

    let ready = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let ready_cb = std::sync::Arc::clone(&ready);
    buffer.slice(..).map_async(MapMode::Read, move |r| {
        r.expect("Failed to map async readback buffer");
        ready_cb.store(true, std::sync::atomic::Ordering::Release);
    });

    world
        .get_mut::<ReadbackRing>(entity)
        .unwrap()
        .pending
        .push_back(PendingReadback {
            buffer,
            ready,
            submission,
            padded_bytes_per_row,
            bytes_per_row,
            format,
            width: size.width,
            height: size.height,
        });
    Ok(())
}

/// Number of enqueued readbacks not yet fetched.
pub fn readback_ring_pending(In(entity): In<Entity>, world: &mut World) -> Result<usize> {
    Ok(world
        .get::<ReadbackRing>(entity)
        .map(|r| r.pending.len())
        .unwrap_or(0))
}

/// Fetch the oldest enqueued readback if (or, blocking, once) its copy has
/// completed. Returns `None` when nothing is pending, or when non-blocking
/// and the oldest copy is still in flight.
pub fn readback_ring_fetch(
    In((entity, blocking)): In<(Entity, bool)>,
    world: &mut World,
) -> Result<Option<ReadbackData>> {
    let render_device = world.resource::<RenderDevice>().clone();
    {
        let Some(ring) = world.get::<ReadbackRing>(entity) else {
            return Ok(None);
        };
        if ring.pending.is_empty() {
            return Ok(None);
        }
    }

    // Pump the device so map_async callbacks can fire; block only if asked,
    // and then only until THIS copy's submission completes — never the whole
    // queue.
    loop {
        let (ready, submission) = {
            let ring = world.get::<ReadbackRing>(entity).unwrap();
            let front = ring.pending.front().unwrap();
            (
                front.ready.load(std::sync::atomic::Ordering::Acquire),
                front.submission.clone(),
            )
        };
        if ready {
            break;
        }
        if blocking {
            render_device
                .poll(PollType::Wait {
                    submission_index: Some(submission),
                    timeout: None,
                })
                .expect("Failed to poll device for async readback");
        } else {
            render_device
                .poll(PollType::Poll)
                .expect("Failed to poll device for async readback");
            let ready_now = {
                let ring = world.get::<ReadbackRing>(entity).unwrap();
                ring.pending
                    .front()
                    .unwrap()
                    .ready
                    .load(std::sync::atomic::Ordering::Acquire)
            };
            if !ready_now {
                return Ok(None);
            }
        }
    }

    let pr = world
        .get_mut::<ReadbackRing>(entity)
        .unwrap()
        .pending
        .pop_front()
        .unwrap();
    let data = pr
        .buffer
        .slice(..)
        .get_mapped_range()
        .expect("Failed to get mapped range for async readback")
        .to_vec();
    pr.buffer.unmap();

    let unpadded = if pr.padded_bytes_per_row != pr.bytes_per_row {
        data.chunks_exact(pr.padded_bytes_per_row)
            .take(pr.height as usize)
            .flat_map(|row| &row[..pr.bytes_per_row])
            .copied()
            .collect()
    } else {
        data
    };

    let buffer_size = pr.padded_bytes_per_row as u64 * pr.height as u64;
    world
        .get_mut::<ReadbackRing>(entity)
        .unwrap()
        .free
        .push((pr.buffer, buffer_size));

    Ok(Some(ReadbackData {
        bytes: unpadded,
        format: pr.format,
        width: pr.width,
        height: pr.height,
    }))
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
