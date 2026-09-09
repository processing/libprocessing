//! GPU-resident recording glue (macOS only).
//!
//! Pairs `processing_render`'s IOSurface capture (wgpu blit, GPU completion
//! callback) with `processing_video`'s AVAssetWriter recorder (hardware
//! encode + mux). Selected automatically by `processing_record_capture` when
//! a VideoToolbox codec is requested; set `PROCESSING_RECORD_CPU=1` to force
//! the readback + ffmpeg path instead.

use bevy::prelude::Entity;
use processing::prelude::{
    error::ProcessingError, graphics_capture_into_iosurface, graphics_capture_reset,
    graphics_device_poll, graphics_surface_size,
};
use processing_video::macos::{AvfCodec, AvfRecorder, AvfRecorderConfig};

fn record_err(e: impl std::fmt::Display) -> ProcessingError {
    ProcessingError::InvalidArgument(format!("video record: {e}"))
}

/// One in-progress GPU recording. The AVF recorder is created on the first
/// captured frame so its dimensions match the graphics being recorded.
pub struct GpuRecorder {
    path: String,
    fps: f64,
    codec: AvfCodec,
    bitrate: Option<u64>,
    bframes: Option<u32>,
    recorder: Option<AvfRecorder>,
}

impl GpuRecorder {
    pub fn new(
        path: String,
        fps: f64,
        codec: AvfCodec,
        bitrate: Option<u64>,
        bframes: Option<u32>,
    ) -> Self {
        Self {
            path,
            fps,
            codec,
            bitrate,
            bframes,
            recorder: None,
        }
    }

    /// Capture the current canvas as one frame: check a pixel buffer out of
    /// the recorder's pool, blit into its IOSurface on the GPU, and hand the
    /// buffer to the encoder when the GPU signals completion. Blocks only
    /// when `MAX_IN_FLIGHT` frames are still in the pipeline.
    pub fn capture(&mut self, entity: Entity) -> Result<(), ProcessingError> {
        let timing = std::env::var_os("PROCESSING_RECORD_TIMING").is_some();
        let t0 = std::time::Instant::now();
        let (width, height) = graphics_surface_size(entity)?;
        let recorder = match &mut self.recorder {
            Some(r) => {
                if r.width() != width || r.height() != height {
                    return Err(record_err(format!(
                        "canvas resized mid-recording ({}x{} -> {width}x{height})",
                        r.width(),
                        r.height(),
                    )));
                }
                r
            }
            None => {
                let config = AvfRecorderConfig {
                    width,
                    height,
                    fps: self.fps,
                    codec: self.codec,
                    bitrate: self.bitrate,
                    bframes: self.bframes,
                };
                self.recorder
                    .insert(AvfRecorder::new(&self.path, config).map_err(record_err)?)
            }
        };

        let frame = recorder
            .acquire_frame(&mut || {
                let _ = graphics_device_poll();
            })
            .map_err(record_err)?;
        let t1 = std::time::Instant::now();
        let ptr = frame.iosurface_ptr();
        let id = frame.iosurface_id();
        let completer = recorder.completer();
        graphics_capture_into_iosurface(
            entity,
            ptr,
            id,
            Box::new(move || completer.complete(frame)),
        )?;
        if timing {
            eprintln!(
                "capture(gpu): acquire={:?} blit-submit={:?}",
                t1 - t0,
                t1.elapsed(),
            );
        }
        Ok(())
    }

    /// Finalize the file and return the number of frames encoded. `entity` is
    /// the recorded graphics (0 bits if no frame was ever captured), used to
    /// drop the render side's texture cache.
    pub fn finish(self, entity_bits: u64) -> Result<u64, ProcessingError> {
        if entity_bits != 0 {
            // Best-effort: the graphics may already be gone at exit.
            let _ = graphics_capture_reset(Entity::from_bits(entity_bits));
        }
        match self.recorder {
            Some(recorder) => {
                let frames = recorder
                    .finish(&mut || {
                        let _ = graphics_device_poll();
                    })
                    .map_err(record_err)?;
                if frames == 0 {
                    return Err(record_err("no frames were captured"));
                }
                Ok(frames)
            }
            None => Err(ProcessingError::InvalidArgument(
                "no frames were captured between processing_record_begin and \
                 processing_record_end"
                    .to_string(),
            )),
        }
    }
}
