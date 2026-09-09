//! Offline video recording. Frame timestamps come from the frame index and
//! configured fps, never wall-clock time.
//!
//! The capture→encode queue is bounded ([`QUEUE_DEPTH`] frames) with a
//! blocking submit, so the render loop is paced by the encoder and memory
//! stays flat (at 8K RGBA a queued frame is ~130MB). Finalize drains at most
//! the queued frames plus the encoder's internal lookahead; progress is
//! logged so a slow software drain is distinguishable from a hang.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::Arc;
use std::thread;

use bevy::log::info;
use video_rs::ffmpeg;
use video_rs::ffmpeg::util::mathematics::rescale::TIME_BASE;

/// Bounded frame queue between the render thread and the encoder thread.
/// Submit blocks when full — the render loop runs at encode speed rather
/// than piling frames up in memory.
const QUEUE_DEPTH: usize = 3;

/// Pixel layout of the frames passed to [`VideoRecorder::record_frame`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecorderPixelFormat {
    /// 4 bytes per pixel, `[r, g, b, a]`. Alpha is discarded.
    Rgba8,
    /// 4 bytes per pixel, `[b, g, r, a]`. Alpha is discarded.
    Bgra8,
    /// 3 bytes per pixel, `[r, g, b]`.
    Rgb8,
}

impl RecorderPixelFormat {
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            RecorderPixelFormat::Rgba8 | RecorderPixelFormat::Bgra8 => 4,
            RecorderPixelFormat::Rgb8 => 3,
        }
    }

    fn av_pixel(self) -> ffmpeg::format::Pixel {
        match self {
            RecorderPixelFormat::Rgba8 => ffmpeg::format::Pixel::RGBA,
            RecorderPixelFormat::Bgra8 => ffmpeg::format::Pixel::BGRA,
            RecorderPixelFormat::Rgb8 => ffmpeg::format::Pixel::RGB24,
        }
    }
}

/// Which encoder produces the output stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecorderCodec {
    /// Software x264. Portable and quality-tunable (`crf`/`preset`), but
    /// slow at high resolutions (~1 fps at 8K on an M-series CPU).
    #[default]
    X264,
    /// Hardware H.264 via VideoToolbox (macOS). Many× realtime at 4K+;
    /// rate-controlled by `bitrate` instead of `crf`.
    H264VideoToolbox,
    /// Hardware HEVC via VideoToolbox (macOS). Same speed class as H.264
    /// hardware encode, better compression at high resolutions.
    HevcVideoToolbox,
}

impl RecorderCodec {
    fn ffmpeg_name(self) -> &'static str {
        match self {
            RecorderCodec::X264 => "libx264",
            RecorderCodec::H264VideoToolbox => "h264_videotoolbox",
            RecorderCodec::HevcVideoToolbox => "hevc_videotoolbox",
        }
    }

    fn is_videotoolbox(self) -> bool {
        matches!(
            self,
            RecorderCodec::H264VideoToolbox | RecorderCodec::HevcVideoToolbox
        )
    }

    /// Encoder-side pixel format. x264 needs planar 4:2:0 (software swscale
    /// conversion). VideoToolbox accepts BGRA directly and converts to 4:2:0
    /// in hardware — for a BGRA source that removes the CPU swscale pass
    /// entirely, which at high resolutions is otherwise the pipeline's
    /// slowest single-threaded stage (~350ms/frame at 3840×4320); other
    /// sources hand it NV12.
    fn target_pixel(self, source: RecorderPixelFormat) -> ffmpeg::format::Pixel {
        if self.is_videotoolbox() {
            match source {
                RecorderPixelFormat::Bgra8 => ffmpeg::format::Pixel::BGRA,
                _ => ffmpeg::format::Pixel::NV12,
            }
        } else {
            ffmpeg::format::Pixel::YUV420P
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoRecorderConfig {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub pixel_format: RecorderPixelFormat,
    pub codec: RecorderCodec,
    /// Use the encoder's realtime preset (faster encode, larger output).
    /// x264 only.
    pub realtime: bool,
    /// Keyframe interval in frames. `None` uses the encoder default.
    pub keyframe_interval: Option<u64>,
    /// Constant Rate Factor, 0 (lossless) to 51 (worst quality). `None` uses
    /// x264's default of 23; ~18 is visually lossless. x264 only — ignored
    /// (with a log line) by the VideoToolbox encoders.
    pub crf: Option<u8>,
    /// x264 speed/compression preset (`ultrafast` .. `veryslow`). Slower
    /// presets compress better at the same quality. `None` uses `medium`.
    pub preset: Option<String>,
    /// Target bitrate in bits per second. VideoToolbox only; `None` picks
    /// ~0.1 bits/pixel/frame from the resolution and fps.
    pub bitrate: Option<u64>,
    /// Maximum consecutive B frames. `None` leaves the encoder default (3 for
    /// x264).
    ///
    /// B frames are cheap to encode and can come out several times smaller
    /// than the P frames around them. On content the codec predicts badly --
    /// dense instanced geometry tumbling, say -- that size gap becomes a
    /// visible periodic pulse in quality, at the period of the GOP pattern.
    /// `Some(0)` turns them off, which removes the pulse without the bitrate
    /// increase that raising quality would cost.
    pub bframes: Option<u32>,
}

impl VideoRecorderConfig {
    pub fn new(width: u32, height: u32, fps: f64) -> Self {
        Self {
            width,
            height,
            fps,
            pixel_format: RecorderPixelFormat::Rgba8,
            codec: RecorderCodec::default(),
            realtime: false,
            keyframe_interval: None,
            crf: None,
            preset: None,
            bitrate: None,
            bframes: None,
        }
    }

    pub fn with_pixel_format(mut self, format: RecorderPixelFormat) -> Self {
        self.pixel_format = format;
        self
    }

    pub fn with_codec(mut self, codec: RecorderCodec) -> Self {
        self.codec = codec;
        self
    }

    pub fn with_realtime(mut self, realtime: bool) -> Self {
        self.realtime = realtime;
        self
    }

    pub fn with_keyframe_interval(mut self, interval: u64) -> Self {
        self.keyframe_interval = Some(interval);
        self
    }

    pub fn with_crf(mut self, crf: u8) -> Self {
        self.crf = Some(crf);
        self
    }

    pub fn with_preset(mut self, preset: impl Into<String>) -> Self {
        self.preset = Some(preset.into());
        self
    }

    pub fn with_bitrate(mut self, bits_per_second: u64) -> Self {
        self.bitrate = Some(bits_per_second);
        self
    }

    pub fn with_bframes(mut self, bframes: u32) -> Self {
        self.bframes = Some(bframes);
        self
    }

    /// Default VideoToolbox bitrate: ~0.1 bits per pixel per frame, clamped
    /// to a sane range. 3840×4320@60 ≈ 100 Mbps, 7680×4320@60 ≈ 200 Mbps.
    fn auto_bitrate(&self) -> u64 {
        let bps = self.width as f64 * self.height as f64 * self.fps * 0.1;
        (bps as u64).clamp(5_000_000, 800_000_000)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VideoRecordError {
    #[error("invalid recorder config: {0}")]
    InvalidConfig(String),
    #[error("encoder {name} is not available in the linked ffmpeg ({hint})")]
    CodecUnavailable { name: String, hint: String },
    #[error("failed to create {name} encoder at {width}x{height}: {source}{hint}")]
    Create {
        name: &'static str,
        width: u32,
        height: u32,
        source: ffmpeg::Error,
        hint: &'static str,
    },
    #[error("failed to encode frame {frame}: {source}")]
    Encode { frame: u64, source: ffmpeg::Error },
    #[error("failed to finalize video: {0}")]
    Finish(ffmpeg::Error),
    #[error(
        "frame has {got} bytes but {expected} were expected ({width}x{height} {format:?}) — \
         the recorder is sized to the frame it first captured, so this usually means a \
         DIFFERENT canvas (an offscreen/state buffer) was captured into the same recording; \
         record the primary canvas only"
    )]
    FrameSize {
        expected: usize,
        got: usize,
        width: u32,
        height: u32,
        format: RecorderPixelFormat,
    },
    #[error("no frames were recorded; the empty output file was removed")]
    NoFrames,
    #[error("encoder thread terminated unexpectedly")]
    WorkerDied,
}

/// The actual ffmpeg encode + mux pipeline: codec-selectable, unlike
/// video-rs's `Encoder` (which hardcodes libx264).
struct FrameEncoder {
    output: ffmpeg::format::context::Output,
    encoder: ffmpeg::codec::encoder::video::Encoder,
    stream_index: usize,
    encoder_time_base: ffmpeg::Rational,
    scaler: Option<ffmpeg::software::scaling::Context>,
}

// The scaler wraps a raw SwsContext pointer without a Send bound. The whole
// FrameEncoder is created on the caller thread (so creation errors surface
// synchronously) and then moved to the encoder thread, which is its sole
// user from then on — same justification as video-rs's own
// `unsafe impl Send for Encoder`.
unsafe impl Send for FrameEncoder {}

impl FrameEncoder {
    fn new(path: &Path, config: &VideoRecorderConfig) -> Result<Self, VideoRecordError> {
        let name = config.codec.ffmpeg_name();
        let codec = ffmpeg::encoder::find_by_name(name)
            .or_else(|| {
                // Fall back to any H264 encoder for the software path only.
                match config.codec {
                    RecorderCodec::X264 => ffmpeg::encoder::find(ffmpeg::codec::Id::H264),
                    _ => None,
                }
            })
            .ok_or_else(|| VideoRecordError::CodecUnavailable {
                name: name.to_string(),
                hint: if config.codec.is_videotoolbox() {
                    "VideoToolbox encoders need an ffmpeg built on macOS".to_string()
                } else {
                    "ffmpeg was built without libx264".to_string()
                },
            })?;

        let create_err = |source: ffmpeg::Error| VideoRecordError::Create {
            name,
            width: config.width,
            height: config.height,
            source,
            hint: if config.codec == RecorderCodec::H264VideoToolbox
                && (config.width > 4096 || config.height > 4096)
            {
                " (the hardware H.264 encoder rejects dimensions above 4096px — use hevc_videotoolbox)"
            } else {
                ""
            },
        };

        let mut output = ffmpeg::format::output(&path).map_err(create_err)?;
        let global_header = output
            .format()
            .flags()
            .contains(ffmpeg::format::Flags::GLOBAL_HEADER);
        let stream_index = {
            let stream = output
                .add_stream(codec)
                .map_err(create_err)?;
            stream.index()
        };

        let mut context = ffmpeg::codec::Context::new_with_codec(codec);
        if global_header {
            context.set_flags(ffmpeg::codec::Flags::GLOBAL_HEADER);
        }
        let mut video = context
            .encoder()
            .video()
            .map_err(create_err)?;
        video.set_width(config.width);
        video.set_height(config.height);
        video.set_format(config.codec.target_pixel(config.pixel_format));
        video.set_time_base(TIME_BASE);
        video.set_frame_rate(Some(ffmpeg::Rational::new(config.fps.round() as i32, 1)));
        if let Some(interval) = config.keyframe_interval {
            video.set_gop(interval as u32);
        }
        // Set on the codec context rather than as an x264 private option, so
        // it reaches the VideoToolbox encoders too.
        if let Some(bframes) = config.bframes {
            video.set_max_b_frames(bframes as usize);
        }

        let mut options = HashMap::new();
        if config.codec.is_videotoolbox() {
            let bitrate = config.bitrate.unwrap_or_else(|| config.auto_bitrate());
            video.set_bit_rate(bitrate as usize);
            if config.crf.is_some() {
                info!(
                    "recorder: crf is x264-only, ignored by {name}; \
                     rate is controlled by bitrate ({} Mbps)",
                    bitrate / 1_000_000
                );
            }
        } else {
            options.insert(
                "preset".to_string(),
                config
                    .preset
                    .clone()
                    .unwrap_or_else(|| "medium".to_string()),
            );
            if config.realtime {
                options.insert("tune".to_string(), "zerolatency".to_string());
            }
            if let Some(crf) = config.crf {
                options.insert("crf".to_string(), crf.to_string());
            }
        }
        let dict: ffmpeg::Dictionary = options.iter().map(|(k, v)| (&**k, &**v)).collect();

        let encoder = video.open_with(dict).map_err(create_err)?;
        // The encoder may adjust its time base on open; read it back.
        let encoder_time_base = unsafe { (*encoder.0.as_ptr()).time_base.into() };

        output
            .stream_mut(stream_index)
            .expect("stream was just added")
            .set_parameters(&encoder);
        output.write_header().map_err(create_err)?;

        // No scaler when the encoder takes the source format as-is (the
        // VideoToolbox BGRA path): frames are filled directly.
        let target = config.codec.target_pixel(config.pixel_format);
        let scaler = if target == config.pixel_format.av_pixel() {
            None
        } else {
            Some(
                ffmpeg::software::scaling::Context::get(
                    config.pixel_format.av_pixel(),
                    config.width,
                    config.height,
                    target,
                    config.width,
                    config.height,
                    ffmpeg::software::scaling::Flags::empty(),
                )
                .map_err(create_err)?,
            )
        };

        Ok(Self {
            output,
            encoder,
            stream_index,
            encoder_time_base,
            scaler,
        })
    }

    /// Encode one frame of tightly packed pixels in the configured source
    /// format, stamped at `frame_index / fps` seconds.
    fn encode(
        &mut self,
        pixels: &[u8],
        frame_index: u64,
        config: &VideoRecorderConfig,
    ) -> Result<(), ffmpeg::Error> {
        let src_pixel = config.pixel_format.av_pixel();
        let mut frame = match &mut self.scaler {
            None => {
                // Encoder consumes the source format directly.
                let mut frame =
                    ffmpeg::frame::Video::new(src_pixel, config.width, config.height);
                copy_packed_rows(&mut frame, pixels, config);
                frame
            }
            Some(scaler) => {
                let mut src = ffmpeg::frame::Video::new(src_pixel, config.width, config.height);
                copy_packed_rows(&mut src, pixels, config);
                let mut frame = ffmpeg::frame::Video::new(
                    config.codec.target_pixel(config.pixel_format),
                    config.width,
                    config.height,
                );
                scaler.run(&src, &mut frame)?;
                frame
            }
        };

        let tb = self.encoder_time_base;
        let pts = (frame_index as f64 / config.fps * tb.denominator() as f64
            / tb.numerator() as f64)
            .round() as i64;
        frame.set_pts(Some(pts));

        self.encoder.send_frame(&frame)?;
        self.drain_packets()
    }

    fn drain_packets(&mut self) -> Result<(), ffmpeg::Error> {
        let stream_time_base = self
            .output
            .stream(self.stream_index)
            .expect("output stream exists")
            .time_base();
        loop {
            let mut packet = ffmpeg::Packet::empty();
            match self.encoder.receive_packet(&mut packet) {
                Ok(()) => {
                    packet.set_stream(self.stream_index);
                    packet.set_position(-1);
                    packet.rescale_ts(self.encoder_time_base, stream_time_base);
                    packet.write_interleaved(&mut self.output)?;
                }
                Err(ffmpeg::Error::Other { errno }) if errno == ffmpeg::util::error::EAGAIN => {
                    return Ok(());
                }
                Err(ffmpeg::Error::Eof) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
    }

    /// Flush the encoder's internal lookahead and finalize the container.
    fn finish(mut self) -> Result<(), ffmpeg::Error> {
        self.encoder.send_eof()?;
        self.drain_packets()?;
        self.output.write_trailer()
    }
}

/// Copy tightly packed rows into a (possibly padded) ffmpeg frame.
fn copy_packed_rows(frame: &mut ffmpeg::frame::Video, pixels: &[u8], config: &VideoRecorderConfig) {
    let width = config.width as usize;
    let height = config.height as usize;
    let bpp = config.pixel_format.bytes_per_pixel();
    let src_stride = width * bpp;
    let dst_stride = frame.stride(0);
    let data = frame.data_mut(0);
    if src_stride == dst_stride {
        data[..src_stride * height].copy_from_slice(&pixels[..src_stride * height]);
    } else {
        for y in 0..height {
            data[y * dst_stride..y * dst_stride + src_stride]
                .copy_from_slice(&pixels[y * src_stride..(y + 1) * src_stride]);
        }
    }
}

/// Encodes pixel frames into a video file (container chosen by the path
/// extension). Dropping without [`finish`](Self::finish) finalizes
/// best-effort and swallows errors.
pub struct VideoRecorder {
    frame_tx: Option<SyncSender<Vec<u8>>>,
    handle: Option<thread::JoinHandle<Result<u64, VideoRecordError>>>,
    expected_len: usize,
    config: VideoRecorderConfig,
    frames_sent: u64,
    frames_encoded: Arc<AtomicU64>,
}

impl VideoRecorder {
    pub fn new(
        path: impl AsRef<Path>,
        config: VideoRecorderConfig,
    ) -> Result<Self, VideoRecordError> {
        if config.width == 0 || config.height == 0 {
            return Err(VideoRecordError::InvalidConfig(
                "width and height must be non-zero".to_string(),
            ));
        }
        // 1e6 = the encoder's microsecond timebase; higher fps collapses
        // consecutive frames onto duplicate timestamps.
        if !(config.fps.is_finite() && config.fps > 0.0 && config.fps <= 1_000_000.0) {
            return Err(VideoRecordError::InvalidConfig(format!(
                "fps must be a positive number no greater than 1000000, got {}",
                config.fps
            )));
        }
        if let Some(crf) = config.crf
            && crf > 51
        {
            return Err(VideoRecordError::InvalidConfig(format!(
                "crf must be between 0 (lossless) and 51 (worst), got {crf}"
            )));
        }

        static FFMPEG_INIT: Once = Once::new();
        FFMPEG_INIT.call_once(|| {
            let _ = video_rs::init();
        });

        let encoder = FrameEncoder::new(path.as_ref(), &config)?;
        info!(
            "recorder: started {}x{} @ {} fps, {} → {}",
            config.width,
            config.height,
            config.fps,
            config.codec.ffmpeg_name(),
            path.as_ref().display()
        );

        let (frame_tx, frame_rx) = sync_channel(QUEUE_DEPTH);
        let frames_encoded = Arc::new(AtomicU64::new(0));
        let worker_config = config.clone();
        let worker_encoded = Arc::clone(&frames_encoded);
        let dest = path.as_ref().to_path_buf();
        let handle = thread::Builder::new()
            .name("processing_video_encode".to_string())
            .spawn(move || encode_main(encoder, worker_config, frame_rx, worker_encoded, dest))
            .expect("failed to spawn video encoder thread");

        let expected_len =
            config.width as usize * config.height as usize * config.pixel_format.bytes_per_pixel();
        Ok(Self {
            frame_tx: Some(frame_tx),
            handle: Some(handle),
            expected_len,
            config,
            frames_sent: 0,
            frames_encoded,
        })
    }

    /// Queue one frame: tightly packed rows in the configured pixel format,
    /// top row first. Blocks while the encoder is [`QUEUE_DEPTH`] frames
    /// behind — the render loop is paced by encode speed.
    pub fn record_frame(&mut self, pixels: Vec<u8>) -> Result<(), VideoRecordError> {
        if pixels.len() != self.expected_len {
            return Err(VideoRecordError::FrameSize {
                expected: self.expected_len,
                got: pixels.len(),
                width: self.config.width,
                height: self.config.height,
                format: self.config.pixel_format,
            });
        }
        let tx = self.frame_tx.as_ref().ok_or(VideoRecordError::WorkerDied)?;
        if tx.send(pixels).is_err() {
            // The worker only exits early on an error; join to surface it.
            return Err(self
                .join_worker()
                .err()
                .unwrap_or(VideoRecordError::WorkerDied));
        }
        self.frames_sent += 1;
        Ok(())
    }

    pub fn frames_recorded(&self) -> u64 {
        self.frames_sent
    }

    pub fn width(&self) -> u32 {
        self.config.width
    }

    pub fn height(&self) -> u32 {
        self.config.height
    }

    pub fn fps(&self) -> f64 {
        self.config.fps
    }

    /// Flush, finalize the container, and return the number of frames encoded.
    pub fn finish(mut self) -> Result<u64, VideoRecordError> {
        self.join_worker()
    }

    fn join_worker(&mut self) -> Result<u64, VideoRecordError> {
        let pending = self
            .frames_sent
            .saturating_sub(self.frames_encoded.load(Ordering::Relaxed));
        info!(
            "recorder: finalizing — draining {pending} queued frames, \
             then flushing the encoder"
        );
        // Closing the channel signals the worker to flush and finish.
        drop(self.frame_tx.take());
        match self.handle.take() {
            Some(handle) => handle.join().map_err(|_| VideoRecordError::WorkerDied)?,
            None => Err(VideoRecordError::WorkerDied),
        }
    }
}

impl Drop for VideoRecorder {
    fn drop(&mut self) {
        if self.handle.is_some() {
            let _ = self.join_worker();
        }
    }
}

fn encode_main(
    mut encoder: FrameEncoder,
    config: VideoRecorderConfig,
    frame_rx: Receiver<Vec<u8>>,
    frames_encoded: Arc<AtomicU64>,
    dest: PathBuf,
) -> Result<u64, VideoRecordError> {
    let mut frame_index: u64 = 0;

    while let Ok(pixels) = frame_rx.recv() {
        encoder
            .encode(&pixels, frame_index, &config)
            .map_err(|source| VideoRecordError::Encode {
                frame: frame_index,
                source,
            })?;
        frame_index += 1;
        frames_encoded.store(frame_index, Ordering::Relaxed);
        if frame_index % 300 == 0 {
            info!("recorder: {frame_index} frames encoded");
        }
    }

    if frame_index == 0 {
        drop(encoder);
        let _ = std::fs::remove_file(&dest);
        return Err(VideoRecordError::NoFrames);
    }

    info!("recorder: queue drained, flushing encoder ({frame_index} frames)");
    encoder.finish().map_err(VideoRecordError::Finish)?;
    info!(
        "recorder: wrote {frame_index} frames to {}",
        dest.display()
    );
    Ok(frame_index)
}
