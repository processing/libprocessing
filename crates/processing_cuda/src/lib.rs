#![cfg(feature = "cuda")]

use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::render_resource::{Texture, TextureFormat};
use bevy::render::renderer::RenderDevice;
use bevy_cuda::{CudaBuffer, CudaContext};
use processing_core::app_mut;
use processing_core::error::{ProcessingError, Result};
use processing_render::graphics::view_target;
use processing_render::image::{Image, gpu_image, pixel_size};

#[derive(Component)]
pub struct CudaImageBuffer {
    pub buffer: CudaBuffer,
    pub width: u32,
    pub height: u32,
    pub texture_format: TextureFormat,
}

pub struct CudaPlugin;

impl Plugin for CudaPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app(RenderApp);
        let render_device = render_app.world().resource::<RenderDevice>();
        let wgpu_device = render_device.wgpu_device();
        match CudaContext::new(wgpu_device, 0) {
            Ok(ctx) => {
                app.insert_resource(ctx);
            }
            Err(e) => {
                warn!("CUDA not available, GPU interop disabled: {e}");
            }
        }
    }
}

fn cuda_ctx(world: &World) -> Result<&CudaContext> {
    world
        .get_resource::<CudaContext>()
        .ok_or(ProcessingError::CudaError("CUDA not available".into()))
}

fn resolve_texture(app: &mut App, entity: Entity) -> Result<(Texture, TextureFormat, u32, u32)> {
    if app.world().get::<Image>(entity).is_some() {
        let texture = gpu_image(app, entity)?.texture.clone();
        let p_image = app.world().get::<Image>(entity).unwrap();
        return Ok((
            texture,
            p_image.texture_format,
            p_image.size.width,
            p_image.size.height,
        ));
    }
    if let Ok(vt) = view_target(app, entity) {
        let texture = vt.main_texture().clone();
        let fmt = vt.main_texture_format();
        let size = texture.size();
        return Ok((texture, fmt, size.width, size.height));
    }
    Err(ProcessingError::ImageNotFound)
}

pub fn cuda_export(entity: Entity) -> Result<()> {
    app_mut(|app| {
        let (texture, texture_format, width, height) = resolve_texture(app, entity)?;

        let px_size = pixel_size(texture_format)?;
        let buffer_size = (width as u64) * (height as u64) * (px_size as u64);

        let existing = app.world().get::<CudaImageBuffer>(entity);
        let needs_alloc = existing.is_none_or(|buf| buf.buffer.size() != buffer_size);

        if needs_alloc {
            let cuda_ctx = cuda_ctx(app.world())?;
            let buffer = cuda_ctx
                .create_buffer(buffer_size)
                .map_err(|e| ProcessingError::CudaError(format!("Buffer creation failed: {e}")))?;
            app.world_mut().entity_mut(entity).insert(CudaImageBuffer {
                buffer,
                width,
                height,
                texture_format,
            });
        }

        let world = app.world();
        let cuda_buf = world.get::<CudaImageBuffer>(entity).unwrap();
        let cuda_ctx = cuda_ctx(world)?;

        cuda_ctx
            .copy_texture_to_buffer(&texture, &cuda_buf.buffer, width, height, texture_format)
            .map_err(|e| {
                ProcessingError::CudaError(format!("Texture-to-buffer copy failed: {e}"))
            })?;

        Ok(())
    })
}

pub fn cuda_import(entity: Entity, src_device_ptr: u64, byte_size: u64) -> Result<()> {
    app_mut(|app| {
        let (texture, texture_format, width, height) = resolve_texture(app, entity)?;

        let existing = app.world().get::<CudaImageBuffer>(entity);
        let needs_alloc = existing.is_none_or(|buf| buf.buffer.size() != byte_size);

        if needs_alloc {
            let cuda_ctx = cuda_ctx(app.world())?;
            let buffer = cuda_ctx
                .create_buffer(byte_size)
                .map_err(|e| ProcessingError::CudaError(format!("Buffer creation failed: {e}")))?;
            app.world_mut().entity_mut(entity).insert(CudaImageBuffer {
                buffer,
                width,
                height,
                texture_format,
            });
        }

        let world = app.world();
        let cuda_buf = world.get::<CudaImageBuffer>(entity).unwrap();
        let cuda_ctx = cuda_ctx(world)?;

        // wait for work (i.e. python) to be done with the buffer before we read from it
        cuda_ctx
            .synchronize()
            .map_err(|e| ProcessingError::CudaError(format!("synchronize failed: {e}")))?;

        cuda_buf
            .buffer
            .copy_from_device_ptr(src_device_ptr, byte_size)
            .map_err(|e| ProcessingError::CudaError(format!("memcpy_dtod failed: {e}")))?;

        cuda_ctx
            .copy_buffer_to_texture(&cuda_buf.buffer, &texture, width, height, texture_format)
            .map_err(|e| {
                ProcessingError::CudaError(format!("Buffer-to-texture copy failed: {e}"))
            })?;

        Ok(())
    })
}

pub fn cuda_write_back(entity: Entity) -> Result<()> {
    app_mut(|app| {
        let (texture, _, _, _) = resolve_texture(app, entity)?;

        let cuda_buf = app
            .world()
            .get::<CudaImageBuffer>(entity)
            .ok_or(ProcessingError::ImageNotFound)?;

        let cuda_ctx = cuda_ctx(app.world())?;

        cuda_ctx
            .copy_buffer_to_texture(
                &cuda_buf.buffer,
                &texture,
                cuda_buf.width,
                cuda_buf.height,
                cuda_buf.texture_format,
            )
            .map_err(|e| {
                ProcessingError::CudaError(format!("Buffer-to-texture copy failed: {e}"))
            })?;

        Ok(())
    })
}

pub struct CudaBufferInfo {
    pub device_ptr: u64,
    pub width: u32,
    pub height: u32,
    pub texture_format: TextureFormat,
}

pub fn cuda_buffer(entity: Entity) -> Result<CudaBufferInfo> {
    app_mut(|app| {
        let cuda_buf = app
            .world()
            .get::<CudaImageBuffer>(entity)
            .ok_or(ProcessingError::ImageNotFound)?;
        Ok(CudaBufferInfo {
            device_ptr: cuda_buf.buffer.device_ptr(),
            width: cuda_buf.width,
            height: cuda_buf.height,
            texture_format: cuda_buf.texture_format,
        })
    })
}

pub fn typestr_for_format(format: TextureFormat) -> Result<&'static str> {
    match format {
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => Ok("|u1"),
        TextureFormat::Rgba16Float => Ok("<f2"),
        TextureFormat::Rgba32Float => Ok("<f4"),
        _ => Err(ProcessingError::UnsupportedTextureFormat),
    }
}

pub fn elem_size_for_typestr(typestr: &str) -> Result<usize> {
    match typestr {
        "|u1" => Ok(1),
        "<f2" => Ok(2),
        "<f4" => Ok(4),
        _ => Err(ProcessingError::CudaError(format!(
            "unsupported typestr: {typestr}"
        ))),
    }
}
