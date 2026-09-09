//! A graphics object is the core rendering context in Processing, responsible for managing the
//! draw state and recording draw commands to be executed each frame.
//!
//! In Bevy terms, a graphics object is represented as an entity with a camera component
//! configured to render to a specific surface (either a window or an offscreen image).
use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    camera::{
        CameraMainTextureUsages, CameraOutputMode, CameraProjection, ClearColorConfig, Hdr,
        ImageRenderTarget, MsaaWriteback, Projection, RenderTarget, visibility::RenderLayers,
    },
    core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass},
    core_pipeline::tonemapping::Tonemapping,
    ecs::query::QueryEntityError,
    pbr::contact_shadows::ContactShadows,
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
        BATCH_INDEX_STEP, NEAR_HEADROOM, RenderState,
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

/// Marks a canvas that holds SIMULATION STATE rather than a picture:
/// feedback patches, accumulators, reaction-diffusion fields. Such a canvas
/// is guaranteed post-process-free — bloom and tonemapping are never applied,
/// because both write back THROUGH the view target and would re-transform the
/// stored state every frame (tonemapping converges it to the tonemapper's
/// fixed point, which looks like a mysterious blur/wash bug). Draws into it
/// also write LINEAR color, so values round-trip exactly.
#[derive(Component)]
pub struct StateCanvas;

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
        // Depth is REVERSED (near -> 1, far -> 0), which an ortho expresses by
        // swapping near and far in the standard formula. Everything else in
        // the pipeline assumes it: the depth test is GreaterEqual, the depth
        // buffer clears to 0 = far, bevy's own OrthographicProjection swaps
        // for this reason, FrustumProjection below derives the same convention
        // by hand, and world_from_screen unprojects with `1.0 - depth`.
        //
        // Unswapped, this projection alone ran forward-z, and every depth
        // comparison in 2d came out backwards: a later draw LOST to an earlier
        // one, and the background/clear quad -- built at ndc_z ~= 0, the far
        // plane under reverse-z (create_ndc_background_quad) -- landed on the
        // NEAR plane, where it sorted last in Transparent3d and repainted over
        // every blended draw in the frame. That is what made blended draws
        // vanish inside offscreen canvases.
        //
        // NEAR_HEADROOM opens the near side for the per-batch z staircase.
        Mat4::orthographic_rh(
            0.0,
            self.width,
            self.height, // bottom = height
            0.0,         // top = 0
            self.far,
            self.near - NEAR_HEADROOM,
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
    In((width, height, surface_entity, texture_format, state)): In<(
        u32,
        u32,
        Entity,
        TextureFormat,
        bool,
    )>,
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
        // A state canvas takes LINEAR color: the host normalizes fill/stroke
        // to 0..1 and the engine would otherwise read those as sRGB, so
        // fill(0.5) would land at linear 0.21 — fatal when the "color" is a
        // reagent concentration rather than a picture.
        if state {
            crate::color::ColorMode {
                space: crate::color::ColorSpace::Linear,
                max: [1.0; 4],
            }
        } else {
            crate::color::ColorMode::default()
        },
        SurfaceSize(width, height),
        Graphics {
            readback_buffer,
            texture_format,
            size,
        },
    ));

    // Physical-lux contract: EV100 9.7 (Blender's default, and bevy's own
    // implicit default) unless overridden for calibration probes.
    entity_commands.insert(bevy::camera::Exposure {
        ev100: std::env::var("PROCESSING_EV100")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(9.7),
    });

    if state {
        // Float precision, zero post-processing: the canvas IS the data.
        entity_commands.insert((StateCanvas, Hdr));
    } else if is_hdr {
        // Bloom is OPT-IN: a canvas gets HDR + the tonemapper, but no bloom
        // until `bloom()` asks for it. Shipping `Bloom::NATURAL` here meant
        // every canvas had UN-THRESHOLDED bloom (intensity 0.15, prefilter
        // threshold 0.0) without asking — which contradicts the documented
        // API (`no_bloom` "disables bloom previously enabled with bloom")
        // and, in a dense scene, hazes the whole frame including areas with
        // nothing drawn in them. It also skips the bloom node entirely when
        // unused, which is free performance at show resolutions.
        entity_commands.insert((Hdr, Tonemapping::TonyMcMapface));
    }

    // TEMP(bevy-020 debug): force direct drawing to bisect the indirect-args
    // build for sorted instance batches.
    if std::env::var("PROCESSING_NO_INDIRECT").is_ok() {
        entity_commands.insert(bevy::render::view::NoIndirectDrawing);
    }

    let entity = entity_commands.id();

    Ok(entity)
}

/// Turn screen-space contact shadows on or off for this canvas' camera.
///
/// Contact shadows are the short dark line where two surfaces meet -- box on
/// box, box on ground. Shadow maps do the big forms; at cascade resolution
/// they cannot resolve a contact, which is what makes a dense pile of objects
/// read as one mass instead of many things.
///
/// `length` is in WORLD units, not pixels: it is how far the ray marches from
/// the contact, so it scales with the scene, not with the window. `thickness`
/// is likewise world units -- the depth buffer only carries 2.5D information,
/// so the effect has to assume some thickness for what it finds there.
/// `linear_steps` is the per-pixel raymarch step count, and the one to spend
/// down if the effect costs too much at delivery resolution.
///
/// A canvas is a camera here, so this applies to whichever canvas is passed:
/// the primary, a `create_offscreen` buffer, or a `create_window` canvas.
///
/// `ContactShadows` requires a depth prepass, which bevy attaches for us. We
/// take it away again on disable -- nothing else in the engine asks for a
/// prepass, so leaving one running would be paying for a full depth pass per
/// frame to feed an effect that is off.
pub fn set_contact_shadows(
    In((entity, enabled, linear_steps, thickness, length)): In<(Entity, bool, u32, f32, f32)>,
    mut commands: Commands,
    graphics: Query<(), With<Graphics>>,
) -> Result<()> {
    graphics
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    if enabled {
        commands.entity(entity).insert(ContactShadows {
            linear_steps,
            thickness,
            length,
        });
    } else {
        commands
            .entity(entity)
            .remove::<ContactShadows>()
            .remove::<DepthPrepass>();
    }
    Ok(())
}

/// Turn temporal antialiasing on or off for this canvas' camera.
///
/// Kept separate from `set_contact_shadows` deliberately, even though bevy's
/// own contact-shadows example pairs them: TAA rewrites how the WHOLE canvas
/// is antialiased, adds a motion-vector prepass, and jitters the projection
/// every frame. Turning that on as a side effect of asking for a lighting
/// detail would be a large, invisible change to a sketch's whole image.
///
/// TAA requires MSAA off -- the two are different answers to the same
/// question and bevy cannot run both -- so this switches MSAA off with it and
/// restores the default sample count on the way out.
pub fn set_temporal_aa(
    In((entity, enabled)): In<(Entity, bool)>,
    mut commands: Commands,
    graphics: Query<(), With<Graphics>>,
) -> Result<()> {
    graphics
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?;

    if enabled {
        commands
            .entity(entity)
            .insert((TemporalAntiAliasing::default(), Msaa::Off));
    } else {
        commands
            .entity(entity)
            .remove::<TemporalAntiAliasing>()
            .remove::<MotionVectorPrepass>()
            .insert(Msaa::default());
    }
    Ok(())
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
    state_canvases: Query<(), With<StateCanvas>>,
) -> Result<()> {
    use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};

    // Post-processing on a state canvas would corrupt the stored state; the
    // guarantee is the whole point of asking for one.
    if state_canvases.get(entity).is_ok() {
        warn!("bloom() ignored: this graphics is a state canvas (post-process-free by contract)");
        return Ok(());
    }

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
    if std::env::var("PROCESSING_TRACE_VT").is_ok() { eprintln!("ENGINE begin_draw ==== FRAME ===="); }
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

/// A canvas that has finished a frame and is waiting for the next
/// `app.update()` to put it on screen, instead of forcing an update of its
/// own. See [`arm_present`].
#[derive(Component)]
pub struct PendingPresent;

/// Arm `entity` to present on the next update rather than presenting now.
///
/// `present` costs a whole `app.update()` — around 3ms of fixed CPU work
/// regardless of what was drawn — so a sketch driving N windows through the
/// usual `endDraw`-per-canvas path pays N of them per frame and falls off a
/// cliff by a couple of dozen windows. The renderer never needed that: a
/// camera is active exactly when its graphics carries `Flush`
/// (`render::activate_cameras`), so any number of canvases can render and
/// present within a SINGLE update. Arming marks the canvas ready and leaves
/// the update to whoever runs one next — in practice the primary window's
/// `endDraw`, which Processing always calls at the end of every frame.
///
/// Only safe for canvases nothing samples back: a window surface is never an
/// image source, so no draw call can observe the delay. Offscreen buffers
/// keep presenting synchronously.
pub fn arm_present(app: &mut App, entity: Entity) -> Result<()> {
    let mut graphics = graphics_mut!(app, entity);
    graphics
        .get_mut::<Camera>()
        .ok_or(ProcessingError::GraphicsNotFound)?
        .output_mode = CameraOutputMode::Write {
        blend_state: None,
        clear_color: ClearColorConfig::None,
    };
    graphics.insert((Flush, PendingPresent));
    Ok(())
}

/// Stand every armed canvas back down after an update presented it.
fn disarm_pending(app: &mut App) {
    let world = app.world_mut();
    let armed: Vec<Entity> = world
        .query_filtered::<Entity, With<PendingPresent>>()
        .iter(world)
        .collect();
    for entity in armed {
        let Ok(mut graphics) = world.get_entity_mut(entity) else {
            continue;
        };
        if let Some(mut camera) = graphics.get_mut::<Camera>() {
            camera.output_mode = CameraOutputMode::Skip;
        }
        graphics.remove::<(Flush, PendingPresent)>();
    }
}

/// Whether `entity` has draw commands recorded since its last flush.
///
/// A canvas with an empty command buffer has already rendered everything it
/// was asked to, so flushing it again is a whole wasted `app.update()`. The
/// blit path (`image(pg, ...)`) flushes its source defensively, which at a
/// hundred-odd offscreen canvases per frame is the single largest cost in the
/// frame; this lets it skip the ones that are already clean.
pub fn has_pending(In(entity): In<Entity>, buffers: Query<&CommandBuffer>) -> Result<bool> {
    Ok(!buffers
        .get(entity)
        .map_err(|_| ProcessingError::GraphicsNotFound)?
        .commands
        .is_empty())
}

pub fn flush(app: &mut App, entity: Entity) -> Result<()> {
    if std::env::var("PROCESSING_TRACE_VT").is_ok() { eprintln!("ENGINE flush+update"); }
    graphics_mut!(app, entity).insert(Flush);
    app.update();
    graphics_mut!(app, entity).remove::<Flush>();
    // Whatever was armed for presentation rode along on that update.
    disarm_pending(app);
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
    In((entity, texture, lut_view)): In<(Entity, Texture, Option<wgpu::TextureView>)>,
    world: &mut World,
) -> Result<ReadbackData> {
    let render_device = world.resource::<RenderDevice>().clone();
    let render_queue = world.resource::<RenderQueue>().clone();
    let (source_format, size, readback_buffer) = {
        let graphics = world
            .get::<Graphics>(entity)
            .ok_or(ProcessingError::GraphicsNotFound)?;
        (
            graphics.texture_format,
            graphics.size,
            graphics.readback_buffer.clone(),
        )
    };

    // Same conversion policy as the async ring: 8-bit surfaces read back
    // as-is; HDR surfaces convert on the GPU to 8-bit sRGB, honoring the
    // camera's tonemapping — so q/save/loadPixels see what the screen shows.
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

    if world.get::<ReadbackRing>(entity).is_none() {
        world.entity_mut(entity).insert(ReadbackRing::default());
    }
    let copy_src = prepare_convert_source(
        world,
        entity,
        &render_device,
        &texture,
        needs_convert,
        size,
        &lut_view,
    );

    let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());
    if needs_convert {
        let src_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let ring = world.get::<ReadbackRing>(entity).unwrap();
        let convert = ring.convert.as_ref().unwrap();
        record_convert(&render_device, &mut encoder, convert, &src_view);
    }

    encoder.copy_texture_to_buffer(
        copy_src.as_image_copy(),
        TexelCopyBufferInfo {
            buffer: &readback_buffer,
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

    render_queue.submit(std::iter::once(encoder.finish()));

    // The readback buffer is sized for the (larger or equal) source format;
    // map only the region this copy wrote.
    let mapped_len = padded_bytes_per_row as u64 * size.height as u64;
    let buffer_slice = readback_buffer.slice(..mapped_len);

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

    readback_buffer.unmap();

    // strip row padding
    let unpadded = if padded_bytes_per_row != bytes_per_row {
        data.chunks_exact(padded_bytes_per_row)
            .take(size.height as usize)
            .flat_map(|row| &row[..bytes_per_row])
            .copied()
            .collect()
    } else {
        data
    };

    Ok(ReadbackData {
        bytes: unpadded,
        format,
        width: size.width,
        height: size.height,
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

/// A queued readback's mapping is still in flight.
const READBACK_PENDING: u8 = 0;
/// It mapped, and the bytes can be read.
const READBACK_READY: u8 = 1;
/// It will never map. wgpu calls the `map_async` callback with an error when
/// the device goes away underneath a copy that is still in flight, which is
/// the ordinary way a recording sketch exits: the window closes, the device is
/// torn down, and whatever was queued that frame aborts.
const READBACK_FAILED: u8 = 2;

struct PendingReadback {
    buffer: bevy::render::render_resource::Buffer,
    state: std::sync::Arc<std::sync::atomic::AtomicU8>,
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
/// non-8-bit) surfaces are converted into an 8-bit sRGB texture BEFORE the
/// copy, so the readback moves half the bytes and the CPU never touches a
/// float pixel. (A 3840×4320 Rgba16Float frame is 132MB and a per-pixel CPU
/// conversion of it costs ~350ms — it, not the copy, was the recording
/// bottleneck.)
///
/// The conversion honors the camera's tonemapping so captures match the
/// screen: `Tonemapping::None` (the show's calibrated mode) is a plain blit
/// — byte-identical to the previous behavior — while `TonyMcMapface` runs
/// bevy's own LUT in a tiny fullscreen pass. Without this, captures of lit
/// scenes read the raw HDR canvas and clamp everything bright to white.
enum ConvertMode {
    Blit(wgpu::util::TextureBlitter),
    Tonemap {
        pipeline: wgpu::RenderPipeline,
        bind_group_layout: wgpu::BindGroupLayout,
        lut_view: wgpu::TextureView,
        lut_sampler: wgpu::Sampler,
    },
}

struct ConvertPass {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    mode: ConvertMode,
    width: u32,
    height: u32,
}

/// Matches bevy's `sample_tony_mc_mapface_lut` (tonemapping_shared.wgsl);
/// exposure is already pre-applied during shading, so tonemapping is the
/// only remaining display transform. Output target is Rgba8UnormSrgb, so
/// the hardware performs the sRGB encode.
const TONEMAP_CONVERT_SHADER: &str = r#"
@group(0) @binding(0) var hdr_tex: texture_2d<f32>;
@group(0) @binding(1) var lut_tex: texture_3d<f32>;
@group(0) @binding(2) var lut_smp: sampler;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

const LUT_DIMS: f32 = 48.0;

fn tony_mc_mapface(stimulus: vec3<f32>) -> vec3<f32> {
    let uv = saturate(
        (stimulus / (stimulus + 1.0)) * ((LUT_DIMS - 1.0) / LUT_DIMS) + 0.5 / LUT_DIMS,
    );
    return textureSampleLevel(lut_tex, lut_smp, uv, 0.0).rgb;
}

@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let c = textureLoad(hdr_tex, vec2<i32>(pos.xy), 0);
    return vec4<f32>(tony_mc_mapface(max(c.rgb, vec3<f32>(0.0))), 1.0);
}
"#;

fn build_tonemap_convert(
    device: &wgpu::Device,
    lut_view: wgpu::TextureView,
) -> ConvertMode {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("readback tonemap convert"),
        source: wgpu::ShaderSource::Wgsl(TONEMAP_CONVERT_SHADER.into()),
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("readback tonemap convert"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D3,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("readback tonemap convert"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("readback tonemap convert"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });
    let lut_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("readback tonemap lut"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        ..Default::default()
    });
    ConvertMode::Tonemap {
        pipeline,
        bind_group_layout,
        lut_view,
        lut_sampler,
    }
}

/// Ensure the graphics' convert pass matches the current size and camera
/// tonemapping, and return the texture the readback copy should read from.
/// The capture must show what the camera setting shows: TonyMcMapface runs
/// the LUT pass; None (and, conservatively, any variant we don't replicate)
/// is a plain blit — the raw-linear behavior every calibrated-unlit sketch
/// depends on.
fn prepare_convert_source(
    world: &mut World,
    entity: Entity,
    render_device: &RenderDevice,
    texture: &Texture,
    needs_convert: bool,
    size: Extent3d,
    lut_view: &Option<wgpu::TextureView>,
) -> wgpu::Texture {
    if !needs_convert {
        // bevy's Texture derefs to the raw wgpu texture.
        return wgpu::Texture::clone(texture);
    }
    // The canvas texture is the camera's render TARGET: bevy's tonemapping
    // node has already been applied by the time anything lands in it, for
    // every Tonemapping mode. Running the LUT here again double-tonemaps —
    // tony(tony(x)) plateaus at tony(1.0) ≈ sRGB 169, which masked the whole
    // lit-exposure miscalibration (captures showed flat grey while the
    // window was hot white). Readback is therefore ALWAYS a plain blit.
    let want_tonemap = false;
    let _ = lut_view;

    let stale = match world.get::<ReadbackRing>(entity).unwrap().convert {
        Some(ref c) => {
            c.width != size.width
                || c.height != size.height
                || matches!(c.mode, ConvertMode::Tonemap { .. }) != want_tonemap
        }
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
        let mode = if want_tonemap {
            build_tonemap_convert(device, lut_view.clone().unwrap())
        } else {
            ConvertMode::Blit(wgpu::util::TextureBlitter::new(
                device,
                TextureFormat::Rgba8UnormSrgb,
            ))
        };
        world.get_mut::<ReadbackRing>(entity).unwrap().convert = Some(ConvertPass {
            texture: tex,
            view,
            mode,
            width: size.width,
            height: size.height,
        });
    }
    world
        .get::<ReadbackRing>(entity)
        .unwrap()
        .convert
        .as_ref()
        .unwrap()
        .texture
        .clone()
}

/// Record the convert (blit or tonemap) into `encoder`, writing the source
/// into the convert pass's sRGB8 target.
fn record_convert(
    render_device: &RenderDevice,
    encoder: &mut wgpu::CommandEncoder,
    convert: &ConvertPass,
    src_view: &wgpu::TextureView,
) {
    match &convert.mode {
        ConvertMode::Blit(blitter) => {
            blitter.copy(render_device.wgpu_device(), encoder, src_view, &convert.view);
        }
        ConvertMode::Tonemap {
            pipeline,
            bind_group_layout,
            lut_view,
            lut_sampler,
        } => {
            let bind_group = render_device
                .wgpu_device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("readback tonemap convert"),
                    layout: bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(src_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(lut_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(lut_sampler),
                        },
                    ],
                });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("readback tonemap convert"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &convert.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}

/// Resolve the GPU texture view of bevy's TonyMcMapface LUT, if loaded.
pub fn tonemap_lut_view(app: &mut App) -> Option<wgpu::TextureView> {
    use bevy::core_pipeline::tonemapping::TonemappingLuts;
    let lut_id = app
        .world()
        .get_resource::<TonemappingLuts>()?
        .tony_mc_mapface
        .id();
    let render_world = app.sub_app(RenderApp).world();
    let gpu_images =
        render_world.get_resource::<bevy::render::render_asset::RenderAssets<bevy::render::texture::GpuImage>>()?;
    let img = gpu_images.get(lut_id)?;
    Some((*img.texture_view).clone())
}

/// Per-graphics async readback state; created lazily on first enqueue.
#[derive(Component, Default)]
pub struct ReadbackRing {
    free: Vec<(bevy::render::render_resource::Buffer, u64)>,
    pending: std::collections::VecDeque<PendingReadback>,
    convert: Option<ConvertPass>,
}

pub fn readback_ring_enqueue(
    In((entity, texture, lut_view)): In<(Entity, Texture, Option<wgpu::TextureView>)>,
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

    let copy_src = prepare_convert_source(
        world,
        entity,
        &render_device,
        &texture,
        needs_convert,
        size,
        &lut_view,
    );

    let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());
    if needs_convert {
        let src_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let ring = world.get::<ReadbackRing>(entity).unwrap();
        let convert = ring.convert.as_ref().unwrap();
        record_convert(&render_device, &mut encoder, convert, &src_view);
    }
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

    let state = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(READBACK_PENDING));
    let state_cb = std::sync::Arc::clone(&state);
    buffer.slice(..).map_async(MapMode::Read, move |r| {
        // This runs on wgpu's callback thread, so a panic here takes the
        // process down from somewhere with no useful context. A mapping that
        // aborts is expected during teardown -- the device can be gone before
        // the last frame's copy lands -- so record the failure and let the
        // fetch side drop the readback.
        let next = match r {
            Ok(()) => READBACK_READY,
            Err(err) => {
                debug!("readback buffer mapping aborted, dropping the frame: {err}");
                READBACK_FAILED
            }
        };
        state_cb.store(next, std::sync::atomic::Ordering::Release);
    });

    world
        .get_mut::<ReadbackRing>(entity)
        .unwrap()
        .pending
        .push_back(PendingReadback {
            buffer,
            state,
            submission,
            padded_bytes_per_row,
            bytes_per_row,
            format,
            width: size.width,
            height: size.height,
        });
    Ok(())
}

/// Retire the oldest queued readback without reading it.
///
/// For a readback whose mapping or poll failed -- which is what happens when
/// the device is torn down with a copy still in flight. The buffer is dropped
/// rather than recycled onto the free list, since wgpu still regards it as
/// mapped and handing it back out would fail again on its next use.
fn drop_failed_readback(world: &mut World, entity: Entity) {
    if let Some(mut ring) = world.get_mut::<ReadbackRing>(entity) {
        ring.pending.pop_front();
    }
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
        let (state, submission) = {
            let ring = world.get::<ReadbackRing>(entity).unwrap();
            let front = ring.pending.front().unwrap();
            (
                front.state.load(std::sync::atomic::Ordering::Acquire),
                front.submission.clone(),
            )
        };
        if state == READBACK_READY {
            break;
        }
        // A mapping that aborted is never coming back, so retire the entry
        // rather than waiting on it forever. Its buffer is deliberately NOT
        // returned to the free list: it is still considered mapped by wgpu and
        // reusing it would be a second failure.
        if state == READBACK_FAILED {
            drop_failed_readback(world, entity);
            return Ok(None);
        }
        // Polling can itself fail once the device is gone, which is the same
        // teardown story -- drop the frame instead of taking the process with
        // it.
        let polled = if blocking {
            render_device.poll(PollType::Wait {
                submission_index: Some(submission),
                timeout: None,
            })
        } else {
            render_device.poll(PollType::Poll)
        };
        if let Err(err) = polled {
            debug!("polling for a readback failed, dropping the frame: {err}");
            drop_failed_readback(world, entity);
            return Ok(None);
        }
        if !blocking {
            let state_now = {
                let ring = world.get::<ReadbackRing>(entity).unwrap();
                ring.pending
                    .front()
                    .unwrap()
                    .state
                    .load(std::sync::atomic::Ordering::Acquire)
            };
            if state_now == READBACK_PENDING {
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
    let Ok(mapped) = pr.buffer.slice(..).get_mapped_range() else {
        debug!("readback buffer went away before it could be read, dropping the frame");
        return Ok(None);
    };
    let data = mapped.to_vec();
    drop(mapped);
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
