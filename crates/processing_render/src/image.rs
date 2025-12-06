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
            PollType, TexelCopyBufferInfo, TexelCopyBufferLayout, Texture, TextureDimension,
            TextureFormat,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
    },
};
use half::f16;

use crate::error::{ProcessingError, Result};

pub struct PImagePlugin;

impl Plugin for PImagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PImageTextures>();

        let render_app = app.sub_app_mut(bevy::render::RenderApp);
        render_app.add_systems(ExtractSchedule, sync_textures.after(AssetExtractionSystems));
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
struct PImageTextures(EntityHashMap<Texture>);

#[derive(Component)]
pub struct PImage {
    pub handle: Handle<Image>,
    readback_buffer: Buffer,
    pixel_size: usize,
    texture_format: TextureFormat,
    size: Extent3d,
}

fn sync_textures(mut main_world: ResMut<MainWorld>, gpu_images: Res<RenderAssets<GpuImage>>) {
    main_world.resource_scope(|world, mut p_image_textures: Mut<PImageTextures>| {
        let mut p_images = world.query_filtered::<(Entity, &PImage), Changed<PImage>>();
        for (entity, p_image) in p_images.iter(world) {
            if let Some(gpu_image) = gpu_images.get(&p_image.handle) {
                p_image_textures.insert(entity, gpu_image.texture.clone());
            }
        }
    });
}

pub fn create(
    world: &mut World,
    size: Extent3d,
    data: Vec<u8>,
    texture_format: TextureFormat,
) -> Entity {
    fn new_inner(
        In((size, data, texture_format)): In<(Extent3d, Vec<u8>, TextureFormat)>,
        mut commands: Commands,
        mut images: ResMut<Assets<Image>>,
        render_device: Res<RenderDevice>,
    ) -> Entity {
        let image = Image::new(
            size,
            TextureDimension::D2,
            data,
            texture_format,
            RenderAssetUsages::all(),
        );

        let handle = images.add(image);

        let pixel_size = match texture_format {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => 4usize,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::Rgba32Float => 16,
            _ => panic!("Unsupported texture format for readback"),
        };
        let readback_buffer_size = size.width * size.height * pixel_size as u32;
        let readback_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("PImage Readback Buffer"),
            size: readback_buffer_size as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        commands
            .spawn(PImage {
                handle: handle.clone(),
                readback_buffer,
                pixel_size,
                texture_format,
                size,
            })
            .id()
    }

    world
        .run_system_cached_with(new_inner, (size, data, texture_format))
        .expect("Failed to run new PImage system")
}

pub fn load_start(world: &mut World, path: PathBuf) -> Handle<Image> {
    world.get_asset_server().load(path)
}

pub fn is_loaded(world: &World, handle: &Handle<Image>) -> bool {
    matches!(
        world.get_asset_server().load_state(handle),
        LoadState::Loaded
    )
}

#[cfg(target_arch = "wasm32")]
pub fn from_handle(world: &mut World, handle: Handle<Image>) -> Result<Entity> {
    fn from_handle_inner(In(handle): In<Handle<Image>>, world: &mut World) -> Result<Entity> {
        let images = world.resource::<Assets<Image>>();
        let image = images.get(&handle).ok_or(ProcessingError::ImageNotFound)?;

        let size = image.texture_descriptor.size;
        let texture_format = image.texture_descriptor.format;
        let pixel_size = match texture_format {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => 4usize,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::Rgba32Float => 16,
            _ => panic!("Unsupported texture format for readback"),
        };
        let readback_buffer_size = size.width * size.height * pixel_size as u32;

        let render_device = world.resource::<RenderDevice>();
        let readback_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("PImage Readback Buffer"),
            size: readback_buffer_size as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Ok(world
            .spawn(PImage {
                handle: handle.clone(),
                readback_buffer,
                pixel_size,
                texture_format,
                size,
            })
            .id())
    }

    world
        .run_system_cached_with(from_handle_inner, handle)
        .expect("Failed to run from_handle system")
}

pub fn load(world: &mut World, path: PathBuf) -> Result<Entity> {
    fn load_inner(In(path): In<PathBuf>, world: &mut World) -> Result<Entity> {
        let handle = world.get_asset_server().load(path);
        while let LoadState::Loading = world.get_asset_server().load_state(&handle) {
            world
                .run_system_once(handle_internal_asset_events)
                .expect("Failed to run internal asset events system");
        }
        let images = world.resource::<Assets<Image>>();
        let image = images.get(&handle).ok_or(ProcessingError::ImageNotFound)?;

        let size = image.texture_descriptor.size;
        let texture_format = image.texture_descriptor.format;
        let pixel_size = match texture_format {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => 4usize,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::Rgba32Float => 16,
            _ => panic!("Unsupported texture format for readback"),
        };
        let readback_buffer_size = size.width * size.height * pixel_size as u32;

        let render_device = world.resource::<RenderDevice>();
        let readback_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("PImage Readback Buffer"),
            size: readback_buffer_size as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Ok(world
            .spawn(PImage {
                handle: handle.clone(),
                readback_buffer,
                pixel_size,
                texture_format,
                size,
            })
            .id())
    }

    world
        .run_system_cached_with(load_inner, path.to_path_buf())
        .expect("Failed to run load system")
}

pub fn resize(world: &mut World, entity: Entity, new_size: Extent3d) -> Result<()> {
    fn resize_inner(
        In((entity, new_size)): In<(Entity, Extent3d)>,
        mut p_images: Query<&mut PImage>,
        mut images: ResMut<Assets<Image>>,
        render_device: Res<RenderDevice>,
    ) -> Result<()> {
        let mut image = p_images
            .get_mut(entity)
            .map_err(|_| ProcessingError::ImageNotFound)?;

        images
            .get_mut(&image.handle)
            .ok_or(ProcessingError::ImageNotFound)?
            .resize_in_place(new_size);

        let size = new_size.width as u64 * new_size.height as u64 * image.pixel_size as u64;
        image.readback_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("PImage Readback Buffer"),
            size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Ok(())
    }

    world
        .run_system_cached_with(resize_inner, (entity, new_size))
        .expect("Failed to run resize system")
}

pub fn load_pixels(world: &mut World, entity: Entity) -> Result<Vec<LinearRgba>> {
    fn readback_inner(
        In(entity): In<Entity>,
        p_images: Query<&PImage>,
        p_image_textures: Res<PImageTextures>,
        mut images: ResMut<Assets<Image>>,
        render_device: Res<RenderDevice>,
        render_queue: ResMut<RenderQueue>,
    ) -> Result<Vec<LinearRgba>> {
        let p_image = p_images
            .get(entity)
            .map_err(|_| ProcessingError::ImageNotFound)?;
        let texture = p_image_textures
            .get(&entity)
            .ok_or(ProcessingError::ImageNotFound)?;

        let mut encoder =
            render_device.create_command_encoder(&CommandEncoderDescriptor::default());

        let block_dimensions = p_image.texture_format.block_dimensions();
        let block_size = p_image.texture_format.block_copy_size(None).unwrap();

        let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
            (p_image.size.width as usize / block_dimensions.0 as usize) * block_size as usize,
        );

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

        let data = match p_image.texture_format {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => data
                .chunks_exact(p_image.pixel_size)
                .map(|chunk| LinearRgba::from_u8_array([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect(),
            TextureFormat::Rgba16Float => data
                .chunks_exact(p_image.pixel_size)
                .map(|chunk| {
                    let r = f16::from_bits(u16::from_le_bytes([chunk[0], chunk[1]])).to_f32();
                    let g = f16::from_bits(u16::from_le_bytes([chunk[2], chunk[3]])).to_f32();
                    let b = f16::from_bits(u16::from_le_bytes([chunk[4], chunk[5]])).to_f32();
                    let a = f16::from_bits(u16::from_le_bytes([chunk[6], chunk[7]])).to_f32();
                    LinearRgba::from_f32_array([r, g, b, a])
                })
                .collect(),
            TextureFormat::Rgba32Float => data
                .chunks_exact(p_image.pixel_size)
                .map(|chunk| {
                    let r = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    let g = f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                    let b = f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);
                    let a = f32::from_le_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]);
                    LinearRgba::from_f32_array([r, g, b, a])
                })
                .collect(),
            _ => return Err(ProcessingError::UnsupportedTextureFormat),
        };

        Ok(data)
    }

    world
        .run_system_cached_with(readback_inner, entity)
        .expect("Failed to run readback system")
}

pub fn destroy(world: &mut World, entity: Entity) -> Result<()> {
    fn destroy_inner(
        In(entity): In<Entity>,
        mut p_images: Query<&mut PImage>,
        mut images: ResMut<Assets<Image>>,
        mut p_image_textures: ResMut<PImageTextures>,
    ) -> Result<()> {
        let p_image = p_images
            .get_mut(entity)
            .map_err(|_| ProcessingError::ImageNotFound)?;

        images.remove(&p_image.handle);
        p_image_textures.remove(&entity);

        Ok(())
    }

    world
        .run_system_cached_with(destroy_inner, entity)
        .expect("Failed to run destroy system")
}
