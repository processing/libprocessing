//! Deterministic screenshot capture for visual regression CI.
use std::path::PathBuf;
use std::time::Duration;

use bevy::{
    camera::RenderTarget, platform::time::Instant, prelude::*,
    render::render_resource::TextureFormat, time::TimeUpdateStrategy,
};
use processing_core::app_mut;
use processing_core::error::{ProcessingError, Result};

pub const SCREENSHOT_ENV: &str = "PROCESSING_CI_SCREENSHOT";
pub const FRAME_ENV: &str = "PROCESSING_CI_FRAME";
pub const DEFAULT_FRAME: u32 = 10;
pub const FIXED_TIMESTEP: Duration = Duration::from_micros(16_667);

#[derive(Resource, Debug)]
pub struct CiCapture {
    path: PathBuf,
    frame: u32,
    epoch: Instant,
    target: Option<Entity>,
    frames_ended: u32,
    done: bool,
}

pub fn enabled() -> bool {
    std::env::var_os(SCREENSHOT_ENV).is_some_and(|p| !p.is_empty())
}

impl CiCapture {
    fn from_env() -> Result<Option<Self>> {
        let Some(path) = std::env::var_os(SCREENSHOT_ENV).filter(|p| !p.is_empty()) else {
            return Ok(None);
        };
        let frame = match std::env::var(FRAME_ENV) {
            Ok(s) => s.parse::<u32>().ok().filter(|n| *n > 0).ok_or_else(|| {
                ProcessingError::InvalidArgument(format!(
                    "{FRAME_ENV} must be a positive integer, got {s:?}"
                ))
            })?,
            Err(_) => DEFAULT_FRAME,
        };
        Ok(Some(Self {
            path: path.into(),
            frame,
            epoch: Instant::now(),
            target: None,
            frames_ended: 0,
            done: false,
        }))
    }
}

pub struct CiPlugin;

impl Plugin for CiPlugin {
    fn build(&self, app: &mut App) {
        match CiCapture::from_env() {
            Ok(Some(capture)) => {
                info!(
                    "CI capture enabled: frame {} -> {}",
                    capture.frame,
                    capture.path.display()
                );
                let epoch = capture.epoch;
                app.insert_resource(capture)
                    .insert_resource(TimeUpdateStrategy::ManualInstant(epoch));
            }
            Ok(None) => {}
            Err(e) => panic!("{e}"),
        }
    }
}

pub(crate) fn after_end_draw(app: &mut App, entity: Entity) -> Result<()> {
    let world = app.world_mut();
    let Some(capture) = world.get_resource::<CiCapture>() else {
        return Ok(());
    };
    if capture.done {
        return Ok(());
    }
    match capture.target {
        Some(target) if target != entity => return Ok(()),
        Some(_) => {}
        None => {
            let is_window = matches!(
                world.get::<RenderTarget>(entity),
                Some(RenderTarget::Window(_))
            );
            if !is_window {
                return Ok(());
            }
            world.resource_mut::<CiCapture>().target = Some(entity);
        }
    }

    let mut capture = world.resource_mut::<CiCapture>();
    capture.frames_ended += 1;
    let now = capture.epoch + FIXED_TIMESTEP * capture.frames_ended;
    let reached = capture.frames_ended >= capture.frame;
    world.insert_resource(TimeUpdateStrategy::ManualInstant(now));
    if !reached {
        return Ok(());
    }
    let mut capture = world.resource_mut::<CiCapture>();
    let path = capture.path.clone();
    capture.done = true;

    let (width, height, rgba) = readback_srgba8(app, entity)?;
    write_png(&path, width, height, &rgba)?;
    info!("CI capture written to {}", path.display());
    Ok(())
}

pub fn done() -> bool {
    app_mut(|app| {
        Ok(app
            .world()
            .get_resource::<CiCapture>()
            .is_some_and(|c| c.done))
    })
    .unwrap_or(false)
}

pub(crate) fn readback_srgba8(app: &mut App, entity: Entity) -> Result<(u32, u32, Vec<u8>)> {
    crate::graphics::flush(app, entity)?;
    let vt = crate::graphics::view_target(app, entity)?;
    let texture = vt.main_texture().clone();
    let raw = app
        .world_mut()
        .run_system_cached_with(crate::graphics::readback_raw, (entity, texture))
        .unwrap()?;
    let rgba = match raw.format {
        TextureFormat::Rgba8UnormSrgb => raw.bytes,
        format => {
            let px_size = crate::image::pixel_size(format)?;
            crate::image::bytes_to_pixels(
                &raw.bytes,
                format,
                raw.width,
                raw.height,
                raw.width as usize * px_size,
            )?
            .iter()
            .flat_map(|pixel| Srgba::from(*pixel).to_u8_array())
            .collect()
        }
    };
    Ok((raw.width, raw.height, rgba))
}

pub(crate) fn write_png(
    path: &std::path::Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<()> {
    let io_err = |e: std::io::Error| {
        ProcessingError::InvalidArgument(format!("write {}: {e}", path.display()))
    };
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(io_err)?;
    }
    let file = std::fs::File::create(path).map_err(io_err)?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let png_err =
        |e: png::EncodingError| ProcessingError::InvalidArgument(format!("PNG encode: {e}"));
    encoder
        .write_header()
        .map_err(png_err)?
        .write_image_data(rgba)
        .map_err(png_err)
}
