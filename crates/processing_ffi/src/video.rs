//! Video playback and offline recording.

use std::ffi::c_char;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use bevy::color::{ColorToPacked, Srgba};
use bevy::prelude::Entity;
use bevy::render::render_resource::TextureFormat;
use processing::prelude::{
    error::ProcessingError, graphics, graphics_readback_enqueue, graphics_readback_fetch,
    graphics_readback_pending, image,
};
use processing_video::{
    PlaybackMode, RecorderCodec, RecorderPixelFormat, VideoRecorder, VideoRecorderConfig,
    video_create, video_destroy, video_image, video_is_loaded, video_load, video_position,
    video_resolution, video_seek, video_set_mode, video_set_paused, video_set_speed,
};

use crate::{cstr_to_str, error};

// ── Playback ────────────────────────────────────────────────────────────────

/// Load a video file and create a playback entity for it. The path resolves
/// like other assets (relative to the configured asset root). Decoding starts
/// in the background; poll `processing_video_is_loaded` before querying
/// resolution or drawing frames. Returns the video entity, or 0 on error.
///
/// SAFETY:
/// - Init has been called.
/// - path is a valid, NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_video_create(path: *const c_char) -> u64 {
    error::clear_error();
    error::check(|| {
        let path = unsafe { cstr_to_str(path) }?;
        let handle = video_load(path)?;
        let entity = video_create(handle)?;
        Ok(entity.to_bits())
    })
    .unwrap_or(0)
}

/// Whether the video's first frame has been decoded and its output exists.
#[unsafe(no_mangle)]
pub extern "C" fn processing_video_is_loaded(video_id: u64) -> bool {
    error::clear_error();
    error::check(|| video_is_loaded(Entity::from_bits(video_id))).unwrap_or(false)
}

/// Image entity backed by the video's output texture, creating it on first
/// call. The image tracks playback; draw it like any other image. Returns 0
/// on error (including the video not yet being loaded).
#[unsafe(no_mangle)]
pub extern "C" fn processing_video_image(video_id: u64) -> u64 {
    error::clear_error();
    error::check(|| Ok(video_image(Entity::from_bits(video_id))?.to_bits())).unwrap_or(0)
}

/// Video frame width in pixels, 0 until loaded.
#[unsafe(no_mangle)]
pub extern "C" fn processing_video_width(video_id: u64) -> u32 {
    error::clear_error();
    error::check(|| Ok(video_resolution(Entity::from_bits(video_id))?.0)).unwrap_or(0)
}

/// Video frame height in pixels, 0 until loaded.
#[unsafe(no_mangle)]
pub extern "C" fn processing_video_height(video_id: u64) -> u32 {
    error::clear_error();
    error::check(|| Ok(video_resolution(Entity::from_bits(video_id))?.1)).unwrap_or(0)
}

/// Current playback position in seconds.
#[unsafe(no_mangle)]
pub extern "C" fn processing_video_position(video_id: u64) -> f64 {
    error::clear_error();
    error::check(|| video_position(Entity::from_bits(video_id))).unwrap_or(0.0)
}

/// Seek to a position in seconds.
#[unsafe(no_mangle)]
pub extern "C" fn processing_video_seek(video_id: u64, seconds: f64) {
    error::clear_error();
    error::check(|| video_seek(Entity::from_bits(video_id), seconds));
}

/// Pause or resume playback.
#[unsafe(no_mangle)]
pub extern "C" fn processing_video_set_paused(video_id: u64, paused: bool) {
    error::clear_error();
    error::check(|| video_set_paused(Entity::from_bits(video_id), paused));
}

/// Set the playback rate (1.0 = normal speed).
#[unsafe(no_mangle)]
pub extern "C" fn processing_video_set_speed(video_id: u64, speed: f32) {
    error::clear_error();
    error::check(|| video_set_speed(Entity::from_bits(video_id), speed));
}

/// Choose whether playback loops or stops at the end.
#[unsafe(no_mangle)]
pub extern "C" fn processing_video_set_loop(video_id: u64, looping: bool) {
    error::clear_error();
    error::check(|| {
        let mode = if looping {
            PlaybackMode::Loop
        } else {
            PlaybackMode::Once
        };
        video_set_mode(Entity::from_bits(video_id), mode)
    });
}

/// Destroy the video player and its linked image.
#[unsafe(no_mangle)]
pub extern "C" fn processing_video_destroy(video_id: u64) {
    error::clear_error();
    error::check(|| video_destroy(Entity::from_bits(video_id)));
}

// ── Recording ───────────────────────────────────────────────────────────────

const X264_PRESETS: [&str; 10] = [
    "ultrafast",
    "superfast",
    "veryfast",
    "faster",
    "fast",
    "medium",
    "slow",
    "slower",
    "veryslow",
    "placebo",
];

enum RecState {
    /// The recorder is created on the first captured frame so its dimensions
    /// match the actual readback.
    Pending {
        path: String,
        fps: f64,
        crf: Option<u8>,
        preset: Option<String>,
        codec: RecorderCodec,
        bitrate: Option<u64>,
    },
    Active(VideoRecorder),
}

static RECORDER: Mutex<Option<RecState>> = Mutex::new(None);

/// The graphics whose frames the active recording captures — set on the
/// first capture so the end/exit paths can drain in-flight async readbacks.
static ACTIVE_GRAPHICS: AtomicU64 = AtomicU64::new(0);

fn record_err(e: impl std::fmt::Display) -> ProcessingError {
    ProcessingError::InvalidArgument(format!("video record: {e}"))
}

/// Convert a raw readback into encoder input. 8-bit sRGB surfaces (RGBA or
/// BGRA) pass through untouched — swscale converts to the encoder's 4:2:0
/// layout far faster than a per-pixel CPU loop; anything else takes the slow
/// LinearRgba conversion path.
fn recorder_pixels(
    raw: graphics::ReadbackData,
) -> Result<(Vec<u8>, RecorderPixelFormat), ProcessingError> {
    match raw.format {
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm => {
            Ok((raw.bytes, RecorderPixelFormat::Rgba8))
        }
        TextureFormat::Bgra8UnormSrgb | TextureFormat::Bgra8Unorm => {
            Ok((raw.bytes, RecorderPixelFormat::Bgra8))
        }
        _ => {
            let px = image::pixel_size(raw.format)?;
            let pixels = image::bytes_to_pixels(
                &raw.bytes,
                raw.format,
                raw.width,
                raw.height,
                raw.width as usize * px,
            )?;
            Ok((
                pixels
                    .iter()
                    .flat_map(|pixel| Srgba::from(*pixel).to_u8_array())
                    .collect(),
                RecorderPixelFormat::Rgba8,
            ))
        }
    }
}

/// Feed one fetched frame to the recorder, creating it on the first frame so
/// its dimensions and pixel format match the actual readback.
fn feed_frame(state: &mut RecState, raw: graphics::ReadbackData) -> Result<(), ProcessingError> {
    let (width, height) = (raw.width, raw.height);
    let (pixels, pixel_format) = recorder_pixels(raw)?;
    if let RecState::Pending {
        path,
        fps,
        crf,
        preset,
        codec,
        bitrate,
    } = state
    {
        let mut config =
            VideoRecorderConfig::new(width, height, *fps).with_pixel_format(pixel_format);
        config.crf = *crf;
        config.preset = preset.clone();
        config.codec = *codec;
        config.bitrate = *bitrate;
        let recorder = VideoRecorder::new(&*path, config).map_err(record_err)?;
        *state = RecState::Active(recorder);
    }
    let RecState::Active(recorder) = state else {
        unreachable!("state was just made Active");
    };
    recorder.record_frame(pixels).map_err(record_err)
}

/// Blocking-drain every in-flight async readback into the recorder.
fn drain_ring(state: &mut RecState) -> Result<(), ProcessingError> {
    let bits = ACTIVE_GRAPHICS.load(Ordering::Relaxed);
    if bits == 0 {
        return Ok(());
    }
    let entity = Entity::from_bits(bits);
    while graphics_readback_pending(entity)? > 0 {
        match graphics_readback_fetch(entity, true)? {
            Some(raw) => feed_frame(state, raw)?,
            None => break,
        }
    }
    Ok(())
}

/// Start recording captured frames into a video file (container chosen by
/// the path extension). Frame timestamps come from the frame index and
/// `fps`, never wall-clock time. `crf` is 0 (lossless) to 51 (worst); pass a
/// negative value for the encoder default (23). `preset` is an x264 speed
/// preset (`ultrafast` .. `placebo`); pass null or an empty string for
/// `medium`. `codec` selects the encoder: 0 = software x264, 1 = hardware
/// H.264 via VideoToolbox (macOS), 2 = hardware HEVC via VideoToolbox.
/// `crf`/`preset` apply to x264 only; the VideoToolbox encoders are
/// rate-controlled by `bitrate_bps` (bits per second; pass 0 or negative for
/// an automatic choice of ~0.1 bits/pixel/frame). Frames are queued with
/// `processing_record_capture` and the file is finalized by
/// `processing_record_end`.
///
/// SAFETY:
/// - Init has been called.
/// - path is a valid, NUL-terminated C string.
/// - preset is null or a valid, NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn processing_record_begin(
    path: *const c_char,
    fps: f64,
    crf: i32,
    preset: *const c_char,
    codec: u8,
    bitrate_bps: i64,
) {
    error::clear_error();
    error::check(|| {
        let path = unsafe { cstr_to_str(path) }?.to_string();
        let preset = if preset.is_null() {
            None
        } else {
            match unsafe { cstr_to_str(preset) }? {
                "" => None,
                p => Some(p.to_string()),
            }
        };
        if let Some(ref preset) = preset
            && !X264_PRESETS.contains(&preset.as_str())
        {
            return Err(ProcessingError::InvalidArgument(format!(
                "unknown preset {preset:?}; expected one of {X264_PRESETS:?}"
            )));
        }
        if !(fps.is_finite() && fps > 0.0 && fps <= 1_000_000.0) {
            return Err(ProcessingError::InvalidArgument(format!(
                "fps must be a positive number no greater than 1000000, got {fps}"
            )));
        }
        let crf = match crf {
            i32::MIN..0 => None,
            0..=51 => Some(crf as u8),
            _ => {
                return Err(ProcessingError::InvalidArgument(format!(
                    "crf must be between 0 (lossless) and 51 (worst quality), got {crf}"
                )));
            }
        };
        let codec = match codec {
            0 => RecorderCodec::X264,
            1 => RecorderCodec::H264VideoToolbox,
            2 => RecorderCodec::HevcVideoToolbox,
            other => {
                return Err(ProcessingError::InvalidArgument(format!(
                    "unknown codec {other}; expected 0 (x264), 1 (h264_videotoolbox), \
                     or 2 (hevc_videotoolbox)"
                )));
            }
        };
        let bitrate = (bitrate_bps > 0).then_some(bitrate_bps as u64);
        let mut slot = RECORDER.lock().unwrap();
        if slot.is_some() {
            return Err(ProcessingError::InvalidArgument(
                "already recording; call processing_record_end first".to_string(),
            ));
        }
        *slot = Some(RecState::Pending {
            path,
            fps,
            crf,
            preset,
            codec,
            bitrate,
        });
        Ok(())
    });
}

/// Capture the graphics canvas as one video frame. A no-op when not
/// recording. The copy is submitted asynchronously into a bounded ring so
/// the render loop overlaps the next frame with this frame's readback;
/// completed frames are drained to the encoder in order. The recorder is
/// created on the first completed frame, sized to the readback; a failed
/// capture stops the recording and reports the error.
#[unsafe(no_mangle)]
pub extern "C" fn processing_record_capture(graphics_id: u64) {
    error::clear_error();
    error::check(|| {
        let mut slot = RECORDER.lock().unwrap();
        let Some(state) = slot.as_mut() else {
            return Ok(());
        };
        ACTIVE_GRAPHICS.store(graphics_id, Ordering::Relaxed);
        let entity = Entity::from_bits(graphics_id);
        let result = (|| {
            let timing = std::env::var_os("PROCESSING_RECORD_TIMING").is_some();
            let t0 = std::time::Instant::now();
            graphics_readback_enqueue(entity)?;
            let t1 = std::time::Instant::now();
            let mut fetched = 0u32;
            while let Some(raw) = graphics_readback_fetch(entity, false)? {
                feed_frame(state, raw)?;
                fetched += 1;
            }
            let t2 = std::time::Instant::now();
            // Ring at capacity: block on the oldest copy so the ring never
            // overflows — render stays paced by readback+encode throughput.
            let mut blocked = false;
            if graphics_readback_pending(entity)? >= graphics::READBACK_RING_DEPTH {
                if let Some(raw) = graphics_readback_fetch(entity, true)? {
                    feed_frame(state, raw)?;
                    blocked = true;
                }
            }
            if timing {
                eprintln!(
                    "capture: enqueue={:?} fetch={:?}({fetched}) block={:?}({blocked})",
                    t1 - t0,
                    t2 - t1,
                    t2.elapsed(),
                );
            }
            Ok(())
        })();
        if result.is_err() {
            *slot = None;
            ACTIVE_GRAPHICS.store(0, Ordering::Relaxed);
        }
        result
    });
}

/// Whether a recording is in progress.
#[unsafe(no_mangle)]
pub extern "C" fn processing_record_active() -> bool {
    error::clear_error();
    RECORDER.lock().unwrap().is_some()
}

/// Finish the recording, finalize the file, and return the number of frames
/// encoded (0 on error).
#[unsafe(no_mangle)]
pub extern "C" fn processing_record_end() -> u64 {
    error::clear_error();
    error::check(|| {
        let taken = RECORDER.lock().unwrap().take();
        match taken {
            None => Err(ProcessingError::InvalidArgument(
                "not recording; call processing_record_begin first".to_string(),
            )),
            Some(mut state) => {
                let drained = drain_ring(&mut state);
                ACTIVE_GRAPHICS.store(0, Ordering::Relaxed);
                drained?;
                match state {
                    RecState::Pending { .. } => Err(ProcessingError::InvalidArgument(
                        "no frames were captured between processing_record_begin and \
                         processing_record_end"
                            .to_string(),
                    )),
                    RecState::Active(recorder) => recorder.finish().map_err(record_err),
                }
            }
        }
    })
    .unwrap_or(0)
}

/// Closing the window mid-recording still leaves a playable file.
pub(crate) fn finish_on_exit() {
    if let Ok(mut slot) = RECORDER.lock()
        && let Some(mut state) = slot.take()
    {
        let _ = drain_ring(&mut state);
        if let RecState::Active(recorder) = state {
            let _ = recorder.finish();
        }
    }
    ACTIVE_GRAPHICS.store(0, Ordering::Relaxed);
}
