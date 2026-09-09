//! GPU-resident frame capture into IOSurfaces (macOS only).
//!
//! The wgpu side of the hardware recording path: an IOSurface (backing a
//! CVPixelBuffer owned by the AVFoundation recorder in `processing_video`) is
//! wrapped as a Metal texture, that texture is wrapped as a wgpu texture via
//! wgpu-hal, and the canvas is blitted into it on the GPU. A completion
//! callback registered on the queue reports when the surface actually holds
//! the frame. No pixel ever crosses to the CPU.
//!
//! This module knows nothing about CoreVideo or AVFoundation — it sees only a
//! raw `IOSurfaceRef` pointer and its stable id. Everything is behind
//! `cfg(target_os = "macos")` at the module declaration.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::render_resource::Texture;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use objc2::runtime::ProtocolObject;
use objc2_io_surface::IOSurfaceRef;
use objc2_metal::{
    MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTextureDescriptor, MTLTextureType,
    MTLTextureUsage,
};
use processing_core::error::{ProcessingError, Result};

use crate::graphics::Graphics;

/// The pixel layout of every capture surface: BGRA8, with sRGB encode done by
/// the render hardware on write. Matches the recorder's
/// `kCVPixelFormatType_32BGRA` pool.
const CAPTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

/// GPU work the render side hands back once the blit into the surface has
/// completed on the queue.
pub type CaptureDone = Box<dyn FnOnce() + Send + 'static>;

/// Per-graphics cache of IOSurface texture wraps. The recorder's pool
/// recycles a handful of surfaces for the whole recording, so each one is
/// wrapped exactly once; the map is keyed by the surface's stable id.
#[derive(Component, Default)]
pub struct CaptureCache {
    textures: HashMap<u32, wgpu::Texture>,
    blitter: Option<wgpu::util::TextureBlitter>,
    width: u32,
    height: u32,
}

/// Wrap an IOSurface as a renderable wgpu texture on bevy's device.
///
/// SAFETY contract with the caller: `surface_ptr` is a live `IOSurfaceRef`
/// whose dimensions are exactly `width`×`height` and whose pixel format is
/// BGRA8. The Metal texture retains the surface, so the wrap stays valid in
/// the cache even across the caller releasing its own reference.
fn wrap_iosurface(
    device: &wgpu::Device,
    surface_ptr: usize,
    width: u32,
    height: u32,
) -> Result<wgpu::Texture> {
    let surface: &IOSurfaceRef = unsafe { &*(surface_ptr as *const IOSurfaceRef) };

    let hal_device = unsafe { device.as_hal::<wgpu::hal::api::Metal>() }.ok_or_else(|| {
        ProcessingError::InvalidArgument(
            "GPU capture requires the Metal backend".to_string(),
        )
    })?;

    let desc = unsafe {
        MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
            MTLPixelFormat::BGRA8Unorm_sRGB,
            width as usize,
            height as usize,
            false,
        )
    };
    desc.setUsage(MTLTextureUsage::RenderTarget);
    desc.setStorageMode(MTLStorageMode::Shared);

    let raw_device: &ProtocolObject<dyn MTLDevice> = hal_device.raw_device();
    let mtl_texture = raw_device
        .newTextureWithDescriptor_iosurface_plane(&desc, surface, 0)
        .ok_or_else(|| {
            ProcessingError::InvalidArgument("Metal refused to wrap the IOSurface".to_string())
        })?;

    let hal_texture = unsafe {
        wgpu::hal::metal::Device::texture_from_raw(
            mtl_texture,
            CAPTURE_FORMAT,
            MTLTextureType::Type2D,
            1,
            1,
            wgpu::hal::CopyExtent {
                width,
                height,
                depth: 1,
            },
            None,
        )
    };
    let texture = unsafe {
        device.create_texture_from_hal::<wgpu::hal::api::Metal>(
            hal_texture,
            &wgpu::TextureDescriptor {
                label: Some("gpu capture iosurface"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: CAPTURE_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            wgpu::wgt::TextureUses::UNINITIALIZED,
        )
    };
    Ok(texture)
}

/// Blit the canvas into the given IOSurface and register `on_done` to fire
/// once the copy has completed on the GPU. The blit converts whatever the
/// canvas format is (HDR included) into sRGB-encoded BGRA8 — the same plain
/// blit policy as the readback ring, so captures match the screen.
pub fn capture_into_surface(
    In((entity, texture, surface_ptr, surface_id, on_done)): In<(
        Entity,
        Texture,
        usize,
        u32,
        CaptureDone,
    )>,
    world: &mut World,
) -> Result<()> {
    let render_device = world.resource::<RenderDevice>().clone();
    let render_queue = world.resource::<RenderQueue>().clone();
    let size = world
        .get::<Graphics>(entity)
        .ok_or(ProcessingError::GraphicsNotFound)?
        .size;

    if world.get::<CaptureCache>(entity).is_none() {
        world.entity_mut(entity).insert(CaptureCache::default());
    }
    {
        let mut cache = world.get_mut::<CaptureCache>(entity).unwrap();
        if cache.width != size.width || cache.height != size.height {
            cache.textures.clear();
            cache.width = size.width;
            cache.height = size.height;
        }
        if !cache.textures.contains_key(&surface_id) {
            let wrapped =
                wrap_iosurface(render_device.wgpu_device(), surface_ptr, size.width, size.height)?;
            cache.textures.insert(surface_id, wrapped);
        }
        if cache.blitter.is_none() {
            cache.blitter = Some(wgpu::util::TextureBlitter::new(
                render_device.wgpu_device(),
                CAPTURE_FORMAT,
            ));
        }
    }

    let cache = world.get::<CaptureCache>(entity).unwrap();
    let dst = cache.textures.get(&surface_id).unwrap();
    let blitter = cache.blitter.as_ref().unwrap();

    let src_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let dst_view = dst.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = render_device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("gpu capture blit"),
        });
    blitter.copy(
        render_device.wgpu_device(),
        &mut encoder,
        &src_view,
        &dst_view,
    );
    render_queue.submit(std::iter::once(encoder.finish()));
    render_queue.on_submitted_work_done(on_done);
    Ok(())
}

/// Drop the capture cache (wrapped textures and blitter) for a graphics.
/// Called when a recording ends; the pool the surfaces came from is gone.
pub fn capture_reset(In(entity): In<Entity>, world: &mut World) -> Result<()> {
    if let Ok(mut e) = world.get_entity_mut(entity) {
        e.remove::<CaptureCache>();
    }
    Ok(())
}
