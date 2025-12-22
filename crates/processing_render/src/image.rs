//! An image in Processing is a 2D texture that can be used for rendering.
//!
//! It can be created from raw pixel data, loaded from disk, resized, and read back to CPU memory.
use std::path::PathBuf;

use bevy::{
    asset::{
        LoadState, RenderAssetUsages, handle_internal_asset_events, io::embedded::GetAssetServer,
    },
    ecs::{entity::EntityHashMap, system::RunSystemOnce},
    prelude::*,
    render::{
        ExtractSchedule, MainWorld,
        render_asset::{AssetExtractionSystems, RenderAssets},
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, MapMode,
            Origin3d, PollType, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo,
            Texture, TextureDimension, TextureFormat,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
    },
};
use half::f16;

use crate::error::{ProcessingError, Result};

pub struct ImagePlugin;

impl Plugin for ImagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ImageTextures>();

        let render_app = app.sub_app_mut(bevy::render::RenderApp);
        render_app.add_systems(ExtractSchedule, sync_textures.after(AssetExtractionSystems));
    }
}

// In Bevy, `Image` is a `RenderResource`, which means its descriptor is stored in the main world
// but its GPU texture is stored in the render world. To avoid tedious lookups or the need to
// explicitly reference the render world, we store a mapping of `PImage` entities to their
// corresponding GPU `Texture` in the main world. This is as bit hacky, but it simplifies the API.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct ImageTextures(EntityHashMap<Texture>);

#[derive(Component)]
pub struct Image {
    pub handle: Handle<bevy::image::Image>,
    readback_buffer: Buffer,
    pub texture_format: TextureFormat,
    pub size: Extent3d,
}

fn sync_textures(mut main_world: ResMut<MainWorld>, gpu_images: Res<RenderAssets<GpuImage>>) {
    main_world.resource_scope(|world, mut p_image_textures: Mut<ImageTextures>| {
        let mut p_images = world.query_filtered::<(Entity, &Image), Changed<Image>>();
        for (entity, p_image) in p_images.iter(world) {
            if let Some(gpu_image) = gpu_images.get(&p_image.handle) {
                p_image_textures.insert(entity, gpu_image.texture.clone());
            }
        }
    });
}

pub fn create(
    In((size, data, texture_format)): In<(Extent3d, Vec<u8>, TextureFormat)>,
    mut commands: Commands,
    mut images: ResMut<Assets<bevy::image::Image>>,
    render_device: Res<RenderDevice>,
) -> Entity {
    let image = bevy::image::Image::new(
        size,
        TextureDimension::D2,
        data,
        texture_format,
        RenderAssetUsages::all(),
    );

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

#[cfg(target_arch = "wasm32")]
pub fn from_handle(
    In(handle): In<Handle<bevy::image::Image>>,
    world: &mut World,
) -> Result<Entity> {
    let images = world.resource::<Assets<bevy::image::Image>>();
    let image = images.get(&handle).ok_or(ProcessingError::ImageNotFound)?;

    let size = image.texture_descriptor.size;
    let texture_format = image.texture_descriptor.format;

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
        })
        .id())
}

pub fn load(In(path): In<PathBuf>, world: &mut World) -> Result<Entity> {
    let handle: Handle<bevy::image::Image> = world.get_asset_server().load(path);
    while let LoadState::Loading = world.get_asset_server().load_state(&handle) {
        world.run_system_once(handle_internal_asset_events).unwrap();
    }
    let images = world.resource::<Assets<bevy::image::Image>>();
    let image = images.get(&handle).ok_or(ProcessingError::ImageNotFound)?;

    let size = image.texture_descriptor.size;
    let texture_format = image.texture_descriptor.format;

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
    In(entity): In<Entity>,
    p_images: Query<&Image>,
    p_image_textures: Res<ImageTextures>,
    mut images: ResMut<Assets<bevy::image::Image>>,
    render_device: Res<RenderDevice>,
    render_queue: ResMut<RenderQueue>,
) -> Result<Vec<LinearRgba>> {
    let p_image = p_images
        .get(entity)
        .map_err(|_| ProcessingError::ImageNotFound)?;
    let texture = p_image_textures
        .get(&entity)
        .ok_or(ProcessingError::ImageNotFound)?;

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
        .poll(PollType::Wait)
        .expect("Failed to poll device for map async");

    r.recv().expect("Failed to receive the map_async message");

    let data = buffer_slice.get_mapped_range().to_vec();

    let image = images
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
    In((entity, x, y, width, height, data, px_size)): In<(
        Entity,
        u32,
        u32,
        u32,
        u32,
        Vec<u8>,
        u32,
    )>,
    p_images: Query<&Image>,
    p_image_textures: Res<ImageTextures>,
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

    let texture = p_image_textures
        .get(&entity)
        .ok_or(ProcessingError::ImageNotFound)?;

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

    let p_image = world
        .get::<Image>(entity)
        .ok_or(ProcessingError::ImageNotFound)?;
    let px_size = pixel_size(p_image.texture_format)? as u32;
    let data = pixels_to_bytes(pixels, p_image.texture_format)?;

    Ok((data, px_size))
}

pub fn destroy(
    In(entity): In<Entity>,
    mut commands: Commands,
    mut p_images: Query<&mut Image>,
    mut images: ResMut<Assets<bevy::image::Image>>,
    mut p_image_textures: ResMut<ImageTextures>,
) -> Result<()> {
    let p_image = p_images
        .get_mut(entity)
        .map_err(|_| ProcessingError::ImageNotFound)?;

    images.remove(&p_image.handle);
    p_image_textures.remove(&entity);
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
