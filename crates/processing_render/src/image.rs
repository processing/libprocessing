//! An image in Processing is a 2D texture that can be used for rendering.
//!
//! It can be created from raw pixel data, loaded from disk, resized, and read back to CPU memory.
use std::path::PathBuf;

use bevy::color::Srgba;
use bevy::{
    asset::{
        AssetPath, LoadState, RenderAssetUsages, handle_internal_asset_events,
        io::{AssetSourceId, embedded::GetAssetServer},
    },
    ecs::system::RunSystemOnce,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::{
        ExtractSchedule, MainWorld, Render, RenderApp, RenderSystems,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, MapMode,
            Origin3d, PollType, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo,
            Texture, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
        view::ViewTarget,
    },
};
use half::f16;

use processing_core::config::{Config, ConfigKey};
use processing_core::error::{ProcessingError, Result};

pub struct ImagePlugin;

impl Plugin for ImagePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<BackgroundBlit>::default());
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<BlitterCache>();
            render_app.add_systems(Render, blit_backgrounds.in_set(RenderSystems::Prepare));
            render_app.add_systems(ExtractSchedule, upload_pending);
        }
    }
}

/// The image a `background(img)` paints over this camera's target before its next pass
#[derive(Component, Clone, Default, ExtractComponent)]
pub struct BackgroundBlit(pub Option<Handle<bevy::image::Image>>);

/// `background(img)` applies to one pass only
pub fn clear_background_blits(mut blits: Query<&mut BackgroundBlit>) {
    for mut blit in &mut blits {
        if blit.0.is_some() {
            blit.0 = None;
        }
    }
}

/// Clears each view target with a pending background image, then alpha-blends the image over
/// it like Processing's `set(0, 0, img)`; the pass loads the result
fn blit_backgrounds(
    views: Query<(&ViewTarget, &BackgroundBlit)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    mut cache: ResMut<BlitterCache>,
) {
    let device = render_device.wgpu_device();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("processing_background_blit"),
    });
    let mut pending = false;
    for (target, blit) in &views {
        let Some(handle) = &blit.0 else { continue };
        pending = true;
        let dst: &wgpu::TextureView = target.main_texture_view();
        clear_view(&mut encoder, dst);
        // an image still loading leaves the clear alone
        if let Some(src) = gpu_images.get(handle) {
            cache.copy(
                device,
                &mut encoder,
                &src.texture_view,
                dst,
                target.main_texture_format(),
                true,
            );
        }
    }
    if pending {
        render_queue.submit(std::iter::once(encoder.finish()));
    }
}

fn clear_view(encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("processing_background_clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}

/// Where a blit reads its source pixels: another image, or a graphics render
/// target (via its render-world `ViewTarget`).
pub enum BlitSource {
    Image(Handle<bevy::image::Image>),
    Graphics(Entity),
}

/// Caches one `wgpu::util::TextureBlitter` per destination format and blend. Lives
/// in the render world; blitters are cheap to reuse and expensive to rebuild.
#[derive(Resource, Default)]
pub struct BlitterCache {
    blitters: std::collections::HashMap<(TextureFormat, bool), wgpu::util::TextureBlitter>,
}

impl BlitterCache {
    /// Draws `src` over all of `dst`, which has `format`; `blend` alpha-blends instead of replacing
    pub fn copy(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        src: &wgpu::TextureView,
        dst: &wgpu::TextureView,
        format: TextureFormat,
        blend: bool,
    ) {
        let blitter = self.blitters.entry((format, blend)).or_insert_with(|| {
            let builder = wgpu::util::TextureBlitterBuilder::new(device, format);
            if blend {
                builder
                    .sample_type(wgpu::FilterMode::Linear)
                    .blend_state(wgpu::BlendState::ALPHA_BLENDING)
                    .build()
            } else {
                builder.build()
            }
        });
        blitter.copy(device, encoder, src, dst);
    }
}

/// Blit `source` into the destination image (a sampling copy: handles differing
/// size and format, unlike a raw `copy_texture_to_texture`). Runs in the render
/// world; the destination image's texture is used as the render target.
pub fn blit(
    In((dst_handle, source)): In<(Handle<bevy::image::Image>, BlitSource)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    view_targets: Query<(
        &bevy::render::sync_world::MainEntity,
        &bevy::render::view::ViewTarget,
    )>,
    mut cache: ResMut<BlitterCache>,
) -> Result<()> {
    let dst = gpu_images
        .get(&dst_handle)
        .ok_or(ProcessingError::ImageNotFound)?;
    let dst_view: &wgpu::TextureView = &dst.texture_view;
    let format = dst.texture_descriptor.format;

    let src_view: &wgpu::TextureView = match &source {
        BlitSource::Image(handle) => {
            &gpu_images
                .get(handle)
                .ok_or(ProcessingError::ImageNotFound)?
                .texture_view
        }
        BlitSource::Graphics(entity) => view_targets
            .iter()
            .find(|(main, _)| ***main == *entity)
            .map(|(_, vt)| vt.main_texture_view())
            .ok_or(ProcessingError::GraphicsNotFound)?,
    };

    let device = render_device.wgpu_device();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("processing_blit"),
    });
    cache.copy(device, &mut encoder, src_view, dst_view, format, false);
    render_queue.submit(std::iter::once(encoder.finish()));
    Ok(())
}

#[derive(Component)]
#[relationship(relationship_target = LinkedImageTarget)]
pub struct LinkedImage(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = LinkedImage)]
pub struct LinkedImageTarget(Entity);

#[derive(Component)]
pub struct Image {
    pub handle: Handle<bevy::image::Image>,
    readback_buffer: Buffer,
    pub texture_format: TextureFormat,
    pub size: Extent3d,
    /// Any pixel has alpha below one, so draws sampling it must blend
    pub translucent: bool,
    pub coherence: Coherence,
    /// Bound as a storage texture, so any compute dispatch may write it
    pub bound_rw: bool,
}

/// Which copy of the pixels is newer; the other is brought up to date on demand
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coherence {
    /// CPU writes inside the rect are not in the texture yet
    CpuAhead(URect),
    InSync,
    /// The GPU wrote the texture, so the CPU copy is stale or absent
    GpuAhead,
}

impl Coherence {
    fn of_asset(image: &bevy::image::Image) -> Self {
        if image.data.is_some() {
            Self::InSync
        } else {
            Self::GpuAhead
        }
    }
}

impl Image {
    fn cpu_wrote(&mut self, rect: URect) {
        self.coherence = match self.coherence {
            Coherence::CpuAhead(pending) => Coherence::CpuAhead(pending.union(rect)),
            _ => Coherence::CpuAhead(rect),
        };
    }
}

/// Whether tightly packed `data` in `format` holds any pixel with alpha below one
pub fn has_translucent_pixels(data: &[u8], format: TextureFormat, width: u32, height: u32) -> bool {
    let Ok(px_size) = pixel_size(format) else {
        return true;
    };
    if px_size == 4 {
        return data.as_chunks::<4>().0.iter().any(|p| p[3] < u8::MAX);
    }
    match bytes_to_pixels(data, format, width, height, width as usize * px_size) {
        Ok(pixels) => pixels.iter().any(|p| p.alpha < 1.0),
        Err(_) => true,
    }
}

/// `translucent` for a CPU-side image; unknown contents count as translucent
fn image_translucent(image: &bevy::image::Image) -> bool {
    let size = image.texture_descriptor.size;
    image.data.as_deref().is_none_or(|data| {
        has_translucent_pixels(
            data,
            image.texture_descriptor.format,
            size.width,
            size.height,
        )
    })
}

pub fn create(
    In((size, data, texture_format)): In<(Extent3d, Vec<u8>, TextureFormat)>,
    mut commands: Commands,
    mut images: ResMut<Assets<bevy::image::Image>>,
    render_device: Res<RenderDevice>,
) -> Entity {
    let translucent = has_translucent_pixels(&data, texture_format, size.width, size.height);
    let mut image = bevy::image::Image::new(
        size,
        TextureDimension::D2,
        data,
        texture_format,
        RenderAssetUsages::all(),
    );
    // we need to mark this as a render attachment so it can be used as a render target for
    // drawing and readback
    image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;

    let handle = images.add(image);
    let readback_buffer = create_readback_buffer(
        &render_device,
        size.width,
        size.height,
        texture_format,
        "Image Readback Buffer",
    )
    .expect("Failed to create readback buffer");

    commands
        .spawn((Image {
            handle: handle.clone(),
            readback_buffer,
            texture_format,
            size,
            translucent,
            coherence: Coherence::InSync,
            bound_rw: false,
        },))
        .id()
}

pub fn load_start(world: &mut World, path: PathBuf) -> Handle<bevy::image::Image> {
    world.get_asset_server().load(path)
}

pub fn is_loaded(world: &World, handle: &Handle<bevy::image::Image>) -> bool {
    matches!(
        world.get_asset_server().load_state(handle),
        LoadState::Loaded
    )
}

pub fn from_handle(
    In(handle): In<Handle<bevy::image::Image>>,
    world: &mut World,
) -> Result<Entity> {
    let images = world.resource::<Assets<bevy::image::Image>>();
    let image = images.get(&handle).ok_or(ProcessingError::ImageNotFound)?;

    let size = image.texture_descriptor.size;
    let texture_format = image.texture_descriptor.format;
    let translucent = image_translucent(image);
    let coherence = Coherence::of_asset(image);

    let render_device = world.resource::<RenderDevice>();
    let readback_buffer = create_readback_buffer(
        render_device,
        size.width,
        size.height,
        texture_format,
        "Image Readback Buffer",
    )?;

    Ok(world
        .spawn(Image {
            handle: handle.clone(),
            readback_buffer,
            texture_format,
            size,
            translucent,
            coherence,
            bound_rw: false,
        })
        .id())
}

pub fn load(In(path): In<PathBuf>, world: &mut World) -> Result<Entity> {
    let config = world.resource_mut::<Config>();
    let path: AssetPath = match config.get(ConfigKey::AssetRootPath) {
        Some(_) => {
            AssetPath::from_path_buf(path).with_source(AssetSourceId::from("assets_directory"))
        }
        None => AssetPath::from_path_buf(path),
    };

    let handle: Handle<bevy::image::Image> = world.get_asset_server().load(path);

    while let LoadState::Loading = world.get_asset_server().load_state(&handle) {
        world.run_system_once(handle_internal_asset_events).unwrap();
    }
    let images = world.resource::<Assets<bevy::image::Image>>();
    let image = images.get(&handle).ok_or(ProcessingError::ImageNotFound)?;

    let size = image.texture_descriptor.size;
    let texture_format = image.texture_descriptor.format;
    let translucent = image_translucent(image);
    let coherence = Coherence::of_asset(image);

    let render_device = world.resource::<RenderDevice>();
    let readback_buffer = create_readback_buffer(
        render_device,
        size.width,
        size.height,
        texture_format,
        "Image Readback Buffer",
    )?;

    Ok(world
        .spawn(Image {
            handle: handle.clone(),
            readback_buffer,
            texture_format,
            size,
            translucent,
            coherence,
            bound_rw: false,
        })
        .id())
}

pub fn resize(
    In((entity, new_size)): In<(Entity, Extent3d)>,
    mut p_images: Query<&mut Image>,
    mut images: ResMut<Assets<bevy::image::Image>>,
    render_device: Res<RenderDevice>,
) -> Result<()> {
    let mut p_image = p_images
        .get_mut(entity)
        .map_err(|_| ProcessingError::ImageNotFound)?;

    images
        .get_mut(&p_image.handle)
        .ok_or(ProcessingError::ImageNotFound)?
        .resize_in_place(new_size);
    p_image.coherence = Coherence::InSync;

    p_image.readback_buffer = create_readback_buffer(
        &render_device,
        new_size.width,
        new_size.height,
        p_image.texture_format,
        "Image Readback Buffer",
    )?;
    p_image.size = new_size;

    Ok(())
}

pub fn readback(
    In((entity, texture)): In<(Entity, Texture)>,
    mut p_images: Query<&mut Image>,
    mut images: ResMut<Assets<bevy::image::Image>>,
    render_device: Res<RenderDevice>,
    render_queue: ResMut<RenderQueue>,
) -> Result<()> {
    let mut p_image = p_images
        .get_mut(entity)
        .map_err(|_| ProcessingError::ImageNotFound)?;

    let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());

    let px_size = pixel_size(p_image.texture_format)?;
    let padded_bytes_per_row =
        RenderDevice::align_copy_bytes_per_row(p_image.size.width as usize * px_size);

    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        TexelCopyBufferInfo {
            buffer: &p_image.readback_buffer,
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
        p_image.size,
    );

    render_queue.submit(std::iter::once(encoder.finish()));

    let buffer_slice = p_image.readback_buffer.slice(..);

    let (s, r) = crossbeam_channel::bounded(1);

    buffer_slice.map_async(MapMode::Read, move |r| match r {
        Ok(r) => s.send(r).expect("Failed to send map update"),
        Err(err) => panic!("Failed to map buffer {err}"),
    });

    render_device
        .poll(PollType::wait_indefinitely())
        .expect("Failed to poll device for map async");

    r.recv().expect("Failed to receive the map_async message");

    // drop the row padding so the stored copy matches the asset's tight layout
    let bytes_per_row = p_image.size.width as usize * px_size;
    let data: Vec<u8> = buffer_slice
        .get_mapped_range()
        .chunks_exact(padded_bytes_per_row)
        .take(p_image.size.height as usize)
        .flat_map(|row| row[..bytes_per_row].iter().copied())
        .collect();
    p_image.readback_buffer.unmap();

    // untracked: the GPU already holds these pixels, so no re-upload is wanted
    images
        .get_mut_untracked(&p_image.handle)
        .ok_or(ProcessingError::ImageNotFound)?
        .data = Some(data);
    p_image.coherence = Coherence::InSync;
    Ok(())
}

/// Pixels from the asset's CPU copy; `sync_cpu` first if the GPU may be ahead
pub fn cpu_pixels(world: &World, entity: Entity) -> Result<Vec<LinearRgba>> {
    let p_image = world
        .get::<Image>(entity)
        .ok_or(ProcessingError::ImageNotFound)?;
    let image = world
        .resource::<Assets<bevy::image::Image>>()
        .get(&p_image.handle)
        .ok_or(ProcessingError::ImageNotFound)?;
    let data = image
        .data
        .as_deref()
        .ok_or(ProcessingError::ImageNotFound)?;
    let px_size = pixel_size(p_image.texture_format)?;
    let stride = p_image.size.width as usize * px_size;
    check_len(data.len(), stride, p_image.size, p_image.texture_format)?;
    bytes_to_pixels(
        data,
        p_image.texture_format,
        p_image.size.width,
        p_image.size.height,
        stride,
    )
}

/// A CPU buffer that disagrees with the image size would make row math index out of bounds
fn check_len(len: usize, stride: usize, size: Extent3d, format: TextureFormat) -> Result<()> {
    let expected = stride * size.height as usize;
    if len != expected {
        return Err(ProcessingError::InvalidArgument(format!(
            "image data is {len} bytes, expected {expected} for {}x{} {format:?}",
            size.width, size.height
        )));
    }
    Ok(())
}

/// One pixel from the asset's CPU copy; `None` outside the image
pub fn cpu_pixel(world: &World, entity: Entity, x: i64, y: i64) -> Result<Option<LinearRgba>> {
    let p_image = world
        .get::<Image>(entity)
        .ok_or(ProcessingError::ImageNotFound)?;
    let size = p_image.size;
    if x < 0 || y < 0 || x >= size.width as i64 || y >= size.height as i64 {
        return Ok(None);
    }
    let data = world
        .resource::<Assets<bevy::image::Image>>()
        .get(&p_image.handle)
        .and_then(|image| image.data.as_deref())
        .ok_or(ProcessingError::ImageNotFound)?;
    let px_size = pixel_size(p_image.texture_format)?;
    let stride = size.width as usize * px_size;
    check_len(data.len(), stride, size, p_image.texture_format)?;
    let at = y as usize * stride + x as usize * px_size;
    decode_pixel(&data[at..at + px_size], p_image.texture_format).map(Some)
}

/// Replaces the asset's CPU pixels with `data` in the texture format
pub fn cpu_write(world: &mut World, entity: Entity, data: Vec<u8>) -> Result<()> {
    let (handle, texture_format, size) = {
        let p_image = world
            .get::<Image>(entity)
            .ok_or(ProcessingError::ImageNotFound)?;
        (p_image.handle.clone(), p_image.texture_format, p_image.size)
    };
    let stride = size.width as usize * pixel_size(texture_format)?;
    check_len(data.len(), stride, size, texture_format)?;
    // untracked: only the dirty rect is uploaded, not the whole asset
    world
        .resource_mut::<Assets<bevy::image::Image>>()
        .get_mut_untracked(&handle)
        .ok_or(ProcessingError::ImageNotFound)?
        .data = Some(data);
    let full = URect::new(0, 0, size.width, size.height);
    world
        .get_mut::<Image>(entity)
        .ok_or(ProcessingError::ImageNotFound)?
        .coherence = Coherence::CpuAhead(full);
    Ok(())
}

/// Writes a `width`×`height` block of `pixels` at (`x`, `y`) into the asset's CPU pixels,
/// clipping to the image like Processing's `set`; `sync_cpu` first if the GPU may be ahead
pub fn cpu_write_region(
    world: &mut World,
    entity: Entity,
    x: i64,
    y: i64,
    width: u32,
    height: u32,
    pixels: &[LinearRgba],
) -> Result<()> {
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
    let (handle, texture_format, size) = {
        let p_image = world
            .get::<Image>(entity)
            .ok_or(ProcessingError::ImageNotFound)?;
        (p_image.handle.clone(), p_image.texture_format, p_image.size)
    };
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + width as i64).min(size.width as i64);
    let y1 = (y + height as i64).min(size.height as i64);
    if x1 <= x0 || y1 <= y0 {
        return Ok(());
    }
    let px_size = pixel_size(texture_format)?;
    let bytes = pixels_to_bytes(pixels, texture_format)?;
    let stride = size.width as usize * px_size;
    let mut images = world.resource_mut::<Assets<bevy::image::Image>>();
    // untracked: only the dirty rect is uploaded, not the whole asset
    let data = images
        .get_mut_untracked(&handle)
        .and_then(|image| image.data.as_mut())
        .ok_or(ProcessingError::ImageNotFound)?;
    check_len(data.len(), stride, size, texture_format)?;
    let run = (x1 - x0) as usize * px_size;
    for dy in y0..y1 {
        let src = ((dy - y) as usize * width as usize + (x0 - x) as usize) * px_size;
        let dst = dy as usize * stride + x0 as usize * px_size;
        data[dst..dst + run].copy_from_slice(&bytes[src..src + run]);
    }
    world
        .get_mut::<Image>(entity)
        .ok_or(ProcessingError::ImageNotFound)?
        .cpu_wrote(URect::new(x0 as u32, y0 as u32, x1 as u32, y1 as u32));
    Ok(())
}

/// Writes `rect` of the asset's CPU pixels into the texture
fn upload_rect(
    queue: &RenderQueue,
    texture: &Texture,
    image: &bevy::image::Image,
    rect: URect,
) -> Result<()> {
    let Some(data) = image.data.as_deref() else {
        return Ok(());
    };
    let size = image.texture_descriptor.size;
    let format = image.texture_descriptor.format;
    write_rect(queue, texture, data, size, format, rect)
}

/// Writes `rect` of tightly packed `data`, which covers a texture of `size`, into the texture
pub(crate) fn write_rect(
    queue: &RenderQueue,
    texture: &Texture,
    data: &[u8],
    size: Extent3d,
    format: TextureFormat,
    rect: URect,
) -> Result<()> {
    let px_size = pixel_size(format)?;
    let stride = size.width as usize * px_size;
    check_len(data.len(), stride, size, format)?;
    let start = rect.min.y as usize * stride + rect.min.x as usize * px_size;
    let len = (rect.height() as usize - 1) * stride + rect.width() as usize * px_size;
    queue.write_texture(
        TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: Origin3d {
                x: rect.min.x,
                y: rect.min.y,
                z: 0,
            },
            aspect: Default::default(),
        },
        &data[start..start + len],
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(stride as u32),
            rows_per_image: None,
        },
        Extent3d {
            width: rect.width(),
            height: rect.height(),
            depth_or_array_layers: 1,
        },
    );
    Ok(())
}

/// Uploads every pending CPU write once per frame, before anything renders
fn upload_pending(
    mut main_world: ResMut<MainWorld>,
    mut state: Local<Option<QueryState<(Entity, &'static Image)>>>,
    queue: Res<RenderQueue>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    let state = state.get_or_insert_with(|| main_world.query());
    let pending: Vec<_> = state
        .iter(&main_world)
        .filter_map(|(entity, p_image)| match p_image.coherence {
            Coherence::CpuAhead(rect) => Some((entity, p_image.handle.clone(), rect)),
            _ => None,
        })
        .collect();
    for (entity, handle, rect) in pending {
        // an unprepared image uploads its whole CPU copy when Bevy first prepares it
        let Some(gpu_image) = gpu_images.get(&handle) else {
            continue;
        };
        let Some(image) = main_world
            .resource::<Assets<bevy::image::Image>>()
            .get(&handle)
        else {
            continue;
        };
        if let Err(err) = upload_rect(&queue, &gpu_image.texture, image, rect) {
            error!("image upload failed: {err}");
        }
        if let Some(mut p_image) = main_world.get_mut::<Image>(entity) {
            p_image.coherence = Coherence::InSync;
        }
    }
}

/// Uploads an image's pending CPU writes now, for GPU work that runs outside a frame
pub fn sync_gpu(app: &mut App, entity: Entity) -> Result<()> {
    let p_image = app
        .world()
        .get::<Image>(entity)
        .ok_or(ProcessingError::ImageNotFound)?;
    let Coherence::CpuAhead(rect) = p_image.coherence else {
        return Ok(());
    };
    let handle = p_image.handle.clone();
    let gpu_images = app
        .sub_app(RenderApp)
        .world()
        .resource::<RenderAssets<GpuImage>>();
    // an unprepared image uploads its whole CPU copy when Bevy first prepares it
    let Some(gpu_image) = gpu_images.get(&handle) else {
        return Ok(());
    };
    let world = app.world();
    let image = world
        .resource::<Assets<bevy::image::Image>>()
        .get(&handle)
        .ok_or(ProcessingError::ImageNotFound)?;
    upload_rect(
        world.resource::<RenderQueue>(),
        &gpu_image.texture,
        image,
        rect,
    )?;
    app.world_mut()
        .get_mut::<Image>(entity)
        .ok_or(ProcessingError::ImageNotFound)?
        .coherence = Coherence::InSync;
    Ok(())
}

/// `sync_gpu` for every image
pub fn flush_uploads(app: &mut App) -> Result<()> {
    let world = app.world_mut();
    let pending: Vec<Entity> = world
        .query::<(Entity, &Image)>()
        .iter(world)
        .filter(|(_, p_image)| matches!(p_image.coherence, Coherence::CpuAhead(_)))
        .map(|(entity, _)| entity)
        .collect();
    for entity in pending {
        sync_gpu(app, entity)?;
    }
    Ok(())
}

/// Reads GPU-written pixels back into the CPU copy, so CPU reads and writes see them
pub fn sync_cpu(app: &mut App, entity: Entity) -> Result<()> {
    let world = app.world();
    let p_image = world
        .get::<Image>(entity)
        .ok_or(ProcessingError::ImageNotFound)?;
    let has_cpu = world
        .resource::<Assets<bevy::image::Image>>()
        .get(&p_image.handle)
        .is_some_and(|image| image.data.is_some());
    if has_cpu && p_image.coherence != Coherence::GpuAhead {
        return Ok(());
    }
    let texture = gpu_texture(app, entity)?;
    app.world_mut()
        .run_system_cached_with(readback, (entity, texture))
        .unwrap()
}

/// After a compute dispatch, every storage-bound image may hold new pixels
pub fn invalidate_rw(mut p_images: Query<&mut Image>) {
    for mut p_image in &mut p_images {
        if p_image.bound_rw && p_image.coherence != Coherence::GpuAhead {
            p_image.coherence = Coherence::GpuAhead;
        }
    }
}

/// Marks an image whose texture the GPU just wrote
pub fn gpu_wrote(app: &mut App, entity: Entity) -> Result<()> {
    app.world_mut()
        .get_mut::<Image>(entity)
        .ok_or(ProcessingError::ImageNotFound)?
        .coherence = Coherence::GpuAhead;
    Ok(())
}

/// The texture for a GPU reader, with every CPU write uploaded
pub fn texture_for_read(app: &mut App, entity: Entity) -> Result<Texture> {
    sync_gpu(app, entity)?;
    gpu_texture(app, entity)
}

/// The texture for a GPU writer; the CPU copy is stale from here on
pub fn texture_for_write(app: &mut App, entity: Entity) -> Result<Texture> {
    let texture = texture_for_read(app, entity)?;
    gpu_wrote(app, entity)?;
    Ok(texture)
}

fn gpu_texture(app: &mut App, entity: Entity) -> Result<Texture> {
    // a new image has no texture until a frame prepares it
    if gpu_image(app, entity).is_err() {
        app.update();
    }
    Ok(gpu_image(app, entity)?.texture.clone())
}

/// Get the pixel dimensions `(width, height)` of an image.
pub fn dimensions(In(entity): In<Entity>, p_images: Query<&Image>) -> Result<(u32, u32)> {
    let p_image = p_images
        .get(entity)
        .map_err(|_| ProcessingError::ImageNotFound)?;
    Ok((p_image.size.width, p_image.size.height))
}

pub fn destroy(
    In(entity): In<Entity>,
    mut commands: Commands,
    p_images: Query<&Image>,
) -> Result<()> {
    p_images
        .get(entity)
        .map_err(|_| ProcessingError::ImageNotFound)?;

    commands.entity(entity).despawn();
    Ok(())
}

/// Get the size in bytes of a single pixel for the given texture format.
pub fn pixel_size(format: TextureFormat) -> Result<usize> {
    match format {
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => Ok(4),
        TextureFormat::Rgba16Float => Ok(8),
        TextureFormat::Rgba32Float => Ok(16),
        _ => Err(ProcessingError::UnsupportedTextureFormat),
    }
}

/// Convert LinearRgba pixels to raw bytes in the specified texture format.
pub fn pixels_to_bytes(pixels: &[LinearRgba], format: TextureFormat) -> Result<Vec<u8>> {
    match format {
        TextureFormat::Rgba8Unorm => Ok(pixels.iter().flat_map(|p| p.to_u8_array()).collect()),
        TextureFormat::Rgba8UnormSrgb => Ok(pixels
            .iter()
            .flat_map(|p| Srgba::from(*p).to_u8_array())
            .collect()),
        TextureFormat::Rgba16Float => Ok(pixels
            .iter()
            .flat_map(|p| {
                let [r, g, b, a] = p.to_f32_array();
                [
                    f16::from_f32(r).to_bits().to_le_bytes(),
                    f16::from_f32(g).to_bits().to_le_bytes(),
                    f16::from_f32(b).to_bits().to_le_bytes(),
                    f16::from_f32(a).to_bits().to_le_bytes(),
                ]
                .into_iter()
                .flatten()
            })
            .collect()),
        TextureFormat::Rgba32Float => Ok(pixels
            .iter()
            .flat_map(|p| {
                let [r, g, b, a] = p.to_f32_array();
                [
                    r.to_le_bytes(),
                    g.to_le_bytes(),
                    b.to_le_bytes(),
                    a.to_le_bytes(),
                ]
                .into_iter()
                .flatten()
            })
            .collect()),
        _ => Err(ProcessingError::UnsupportedTextureFormat),
    }
}

/// Convert raw bytes to LinearRgba pixels based on the texture format.
/// Handles row padding, data should come from a GPU texture readback with proper alignment.
pub fn bytes_to_pixels(
    data: &[u8],
    format: TextureFormat,
    width: u32,
    height: u32,
    padded_bytes_per_row: usize,
) -> Result<Vec<LinearRgba>> {
    let px_size = pixel_size(format)?;
    let bytes_per_row = width as usize * px_size;
    data.chunks_exact(padded_bytes_per_row)
        .take(height as usize)
        .flat_map(|row| row[..bytes_per_row].chunks_exact(px_size))
        .map(|chunk| decode_pixel(chunk, format))
        .collect()
}

pub(crate) fn decode_pixel(c: &[u8], format: TextureFormat) -> Result<LinearRgba> {
    Ok(match format {
        TextureFormat::Rgba8Unorm => LinearRgba::from_u8_array([c[0], c[1], c[2], c[3]]),
        TextureFormat::Rgba8UnormSrgb => Srgba::from_u8_array([c[0], c[1], c[2], c[3]]).into(),
        TextureFormat::Rgba16Float => {
            let ch = |i: usize| f16::from_bits(u16::from_le_bytes([c[i], c[i + 1]])).to_f32();
            LinearRgba::from_f32_array([ch(0), ch(2), ch(4), ch(6)])
        }
        TextureFormat::Rgba32Float => {
            let ch = |i: usize| f32::from_le_bytes([c[i], c[i + 1], c[i + 2], c[i + 3]]);
            LinearRgba::from_f32_array([ch(0), ch(4), ch(8), ch(12)])
        }
        _ => return Err(ProcessingError::UnsupportedTextureFormat),
    })
}

/// Create a readback buffer for the given texture dimensions and format.
pub fn create_readback_buffer(
    render_device: &RenderDevice,
    width: u32,
    height: u32,
    format: TextureFormat,
    label: &str,
) -> Result<Buffer> {
    let px_size = pixel_size(format)?;
    let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(width as usize * px_size);
    let buffer_size = padded_bytes_per_row as u64 * height as u64;

    Ok(render_device.create_buffer(&BufferDescriptor {
        label: Some(label),
        size: buffer_size,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    }))
}

pub fn set_sampler(
    In((entity, filter, wrap_x, wrap_y)): In<(Entity, u8, u8, u8)>,
    p_images: Query<&Image>,
    mut images: ResMut<Assets<bevy::image::Image>>,
) -> Result<()> {
    let p_image = p_images
        .get(entity)
        .map_err(|_| ProcessingError::ImageNotFound)?;

    let mut image = images
        .get_mut(&p_image.handle)
        .ok_or(ProcessingError::ImageNotFound)?;

    let filter_mode = match filter {
        0 => ImageFilterMode::Linear,
        1 => ImageFilterMode::Nearest,
        _ => {
            return Err(ProcessingError::InvalidArgument(format!(
                "unknown filter mode: {filter}"
            )));
        }
    };

    let addr_mode = |v: u8| match v {
        0 => Ok(ImageAddressMode::ClampToEdge),
        1 => Ok(ImageAddressMode::Repeat),
        2 => Ok(ImageAddressMode::MirrorRepeat),
        _ => Err(ProcessingError::InvalidArgument(format!(
            "unknown wrap mode: {v}"
        ))),
    };

    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: filter_mode,
        min_filter: filter_mode,
        mipmap_filter: filter_mode,
        address_mode_u: addr_mode(wrap_x)?,
        address_mode_v: addr_mode(wrap_y)?,
        ..Default::default()
    });

    Ok(())
}

fn gpu_image(app: &mut App, entity: Entity) -> Result<&GpuImage> {
    let handle = app
        .world()
        .get::<Image>(entity)
        .ok_or(ProcessingError::ImageNotFound)?
        .handle
        .clone();
    let render_world = app.sub_app(RenderApp).world();
    let gpu_images = render_world.resource::<RenderAssets<GpuImage>>();
    let gpu_image = gpu_images
        .get(&handle)
        .ok_or(ProcessingError::ImageNotFound)?;
    Ok(gpu_image)
}
