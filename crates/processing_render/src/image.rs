//! An image in Processing is a 2D texture that can be used for rendering.
//!
//! It can be created from raw pixel data, loaded from disk, resized, and read back to CPU memory.
use std::path::PathBuf;

use bevy::{
    asset::{
        AssetPath, LoadState, RenderAssetUsages, handle_internal_asset_events,
        io::{AssetSourceId, embedded::GetAssetServer},
    },
    ecs::system::RunSystemOnce,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
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
}

/// Whether tightly packed `data` in `format` holds any pixel with alpha below one
pub fn has_translucent_pixels(data: &[u8], format: TextureFormat, width: u32, height: u32) -> bool {
    let Ok(px_size) = pixel_size(format) else {
        return true;
    };
    if px_size == 4 {
        return data.chunks_exact(4).any(|p| p[3] < u8::MAX);
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
    p_images: Query<&Image>,
    mut images: ResMut<Assets<bevy::image::Image>>,
    render_device: Res<RenderDevice>,
    render_queue: ResMut<RenderQueue>,
) -> Result<Vec<LinearRgba>> {
    let p_image = p_images
        .get(entity)
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

    let data = buffer_slice.get_mapped_range().to_vec();

    let mut image = images
        .get_mut(&p_image.handle)
        .ok_or(ProcessingError::ImageNotFound)?;
    image.data = Some(data.clone());

    p_image.readback_buffer.unmap();

    bytes_to_pixels(
        &data,
        p_image.texture_format,
        p_image.size.width,
        p_image.size.height,
        padded_bytes_per_row,
    )
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
    p_images: Query<&Image>,
    render_queue: Res<RenderQueue>,
) -> Result<()> {
    let p_image = p_images
        .get(entity)
        .map_err(|_| ProcessingError::ImageNotFound)?;

    if x + width > p_image.size.width || y + height > p_image.size.height {
        return Err(ProcessingError::InvalidArgument(format!(
            "Region ({}, {}, {}, {}) exceeds image bounds ({}, {})",
            x, y, width, height, p_image.size.width, p_image.size.height
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

    let p_image = world
        .get::<Image>(entity)
        .ok_or(ProcessingError::ImageNotFound)?;
    let px_size = pixel_size(p_image.texture_format)? as u32;
    let data = pixels_to_bytes(pixels, p_image.texture_format)?;

    Ok((data, px_size))
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
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => {
            Ok(pixels.iter().flat_map(|p| p.to_u8_array()).collect())
        }
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

    let pixels = match format {
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => data
            .chunks_exact(padded_bytes_per_row)
            .take(height as usize)
            .flat_map(|row| {
                row[..bytes_per_row].chunks_exact(px_size).map(|chunk| {
                    LinearRgba::from_u8_array([chunk[0], chunk[1], chunk[2], chunk[3]])
                })
            })
            .collect(),
        TextureFormat::Rgba16Float => data
            .chunks_exact(padded_bytes_per_row)
            .take(height as usize)
            .flat_map(|row| {
                row[..bytes_per_row].chunks_exact(px_size).map(|chunk| {
                    let r = f16::from_bits(u16::from_le_bytes([chunk[0], chunk[1]])).to_f32();
                    let g = f16::from_bits(u16::from_le_bytes([chunk[2], chunk[3]])).to_f32();
                    let b = f16::from_bits(u16::from_le_bytes([chunk[4], chunk[5]])).to_f32();
                    let a = f16::from_bits(u16::from_le_bytes([chunk[6], chunk[7]])).to_f32();
                    LinearRgba::from_f32_array([r, g, b, a])
                })
            })
            .collect(),
        TextureFormat::Rgba32Float => data
            .chunks_exact(padded_bytes_per_row)
            .take(height as usize)
            .flat_map(|row| {
                row[..bytes_per_row].chunks_exact(px_size).map(|chunk| {
                    let r = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    let g = f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                    let b = f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);
                    let a = f32::from_le_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]);
                    LinearRgba::from_f32_array([r, g, b, a])
                })
            })
            .collect(),
        // TODO: Handle more formats as needed
        _ => return Err(ProcessingError::UnsupportedTextureFormat),
    };

    Ok(pixels)
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

pub fn gpu_image(app: &mut App, entity: Entity) -> Result<&GpuImage> {
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
