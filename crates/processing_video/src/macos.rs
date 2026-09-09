//! GPU-resident video recording via AVFoundation (macOS only).
//!
//! The zero-copy counterpart to [`crate::record`]: instead of reading frames
//! back to the CPU and feeding them through ffmpeg, the render side blits each
//! frame straight into an IOSurface-backed `CVPixelBuffer` and the buffer is
//! handed to an `AVAssetWriter`, which drives the hardware (media engine)
//! encoder and muxes the container. The CPU never touches a pixel.
//!
//! This module owns everything AVFoundation/CoreVideo. It knows nothing about
//! wgpu; the render side receives only a raw `IOSurfaceRef` pointer from
//! [`AvfFrame::iosurface_ptr`] and reports GPU completion through
//! [`AvfCompleter::complete`], which may be called from any thread.
//!
//! Per-frame flow (all bookkeeping, no pixel work):
//!   acquire_frame() ── pool buffer, bounded by MAX_IN_FLIGHT ──▶ GPU blit
//!   ──▶ AvfCompleter::complete(frame) on GPU completion ──▶ writer thread
//!   reorders by frame index and appends with pts = index / fps.

use std::collections::BTreeMap;
use std::ffi::c_void;
use std::path::Path;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use objc2::AnyThread;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_av_foundation::{
    AVAssetWriter, AVAssetWriterInput, AVAssetWriterInputPixelBufferAdaptor, AVAssetWriterStatus,
    AVFileType, AVFileTypeAppleM4V, AVFileTypeMPEG4, AVFileTypeQuickTimeMovie, AVMediaTypeVideo,
    AVVideoAllowFrameReorderingKey, AVVideoAverageBitRateKey, AVVideoCodecKey,
    AVVideoCodecTypeH264, AVVideoCodecTypeHEVC, AVVideoColorPrimaries_ITU_R_709_2,
    AVVideoColorPrimariesKey, AVVideoColorPropertiesKey, AVVideoCompressionPropertiesKey,
    AVVideoExpectedSourceFrameRateKey, AVVideoHeightKey, AVVideoTransferFunction_ITU_R_709_2,
    AVVideoTransferFunctionKey, AVVideoWidthKey, AVVideoYCbCrMatrix_ITU_R_709_2,
    AVVideoYCbCrMatrixKey,
};
use objc2_core_foundation::{CFRetained, CFString};
use objc2_core_media::{CMTime, CMTimeFlags};
use objc2_core_video::{
    CVAttachmentMode, CVPixelBuffer, CVPixelBufferGetIOSurface, CVPixelBufferPool,
    kCVImageBufferColorPrimaries_ITU_R_709_2, kCVImageBufferColorPrimariesKey,
    kCVImageBufferTransferFunction_ITU_R_709_2, kCVImageBufferTransferFunctionKey,
    kCVImageBufferYCbCrMatrix_ITU_R_709_2, kCVImageBufferYCbCrMatrixKey, kCVPixelBufferHeightKey,
    kCVPixelBufferIOSurfacePropertiesKey, kCVPixelBufferMetalCompatibilityKey,
    kCVPixelBufferPixelFormatTypeKey, kCVPixelBufferWidthKey, kCVPixelFormatType_32BGRA,
    kCVReturnSuccess,
};
use objc2_foundation::{NSCopying, NSDictionary, NSNumber, NSString, NSURL};
use objc2_io_surface::IOSurfaceRef;

/// Frames allowed between `acquire_frame` and their append completing.
/// Bounds pool memory and how far the render loop can run ahead of the
/// encoder — the same role as `READBACK_RING_DEPTH` on the CPU path.
pub const MAX_IN_FLIGHT: usize = 3;

/// How long to wait on a stalled encoder before giving up. The media engine
/// encodes far faster than any sketch renders, so hitting this means the
/// writer thread died or the device was torn down mid-frame.
const STALL_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, thiserror::Error)]
#[error("avf record: {0}")]
pub struct AvfRecordError(pub String);

type Result<T> = std::result::Result<T, AvfRecordError>;

fn err(msg: impl Into<String>) -> AvfRecordError {
    AvfRecordError(msg.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvfCodec {
    H264,
    Hevc,
}

#[derive(Debug, Clone)]
pub struct AvfRecorderConfig {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub codec: AvfCodec,
    /// Target bitrate in bits per second. `None` picks ~0.1 bits/pixel/frame,
    /// matching the ffmpeg VideoToolbox path.
    pub bitrate: Option<u64>,
    /// `Some(0)` disables frame reordering (no B frames). Other values are
    /// left to the encoder — AVFoundation exposes only the on/off switch.
    pub bframes: Option<u32>,
}

impl AvfRecorderConfig {
    fn bitrate_bps(&self) -> f64 {
        match self.bitrate {
            Some(b) => b as f64,
            None => (self.width as f64 * self.height as f64 * self.fps * 0.1).max(1_000_000.0),
        }
    }
}

/// CF objects are thread-safe refcounted; the pixel data is only ever touched
/// by the GPU and the encoder, so moving the handles across threads is fine.
struct SendCf<T>(CFRetained<T>);
unsafe impl<T> Send for SendCf<T> {}

/// The AVFoundation objects live on whatever thread uses them; Apple's
/// contract is "don't call concurrently", which the writer thread upholds by
/// being their only user after `startWriting`.
struct WriterObjects {
    writer: Retained<AVAssetWriter>,
    input: Retained<AVAssetWriterInput>,
    adaptor: Retained<AVAssetWriterInputPixelBufferAdaptor>,
}
unsafe impl Send for WriterObjects {}

/// Decrements the in-flight count when dropped — after a successful append,
/// or when a frame is abandoned (device teardown drops the GPU callback).
struct InFlightPermit(Arc<(Mutex<usize>, Condvar)>);

impl Drop for InFlightPermit {
    fn drop(&mut self) {
        let (count, cv) = &*self.0;
        *count.lock().unwrap() -= 1;
        cv.notify_all();
    }
}

/// One frame's pixel buffer, checked out of the pool. Blit into its IOSurface
/// on the GPU, then pass it to [`AvfCompleter::complete`] once the GPU work is
/// done. Dropping it without completing simply skips the frame.
pub struct AvfFrame {
    buffer: SendCf<CVPixelBuffer>,
    surface: SendCf<IOSurfaceRef>,
    index: u64,
    _permit: InFlightPermit,
}

impl AvfFrame {
    /// Raw `IOSurfaceRef` pointer for the render side to wrap as a texture.
    /// Valid for the lifetime of this frame.
    pub fn iosurface_ptr(&self) -> *mut c_void {
        CFRetained::as_ptr(&self.surface.0).as_ptr().cast()
    }

    /// Stable identity of the underlying IOSurface, for caching texture wraps
    /// across the pool's recycling.
    pub fn iosurface_id(&self) -> u32 {
        self.surface.0.id()
    }

    pub fn index(&self) -> u64 {
        self.index
    }
}

/// Cloneable handle for reporting GPU completion of a frame; safe to call
/// from any thread (wgpu's completion callback thread, in practice).
#[derive(Clone)]
pub struct AvfCompleter {
    tx: Sender<AvfFrame>,
}

impl AvfCompleter {
    /// Queue the frame for appending. Ignores a finished/failed recorder —
    /// the frame is dropped and its permit released either way.
    pub fn complete(&self, frame: AvfFrame) {
        let _ = self.tx.send(frame);
    }
}

/// Hardware video recorder writing through AVAssetWriter. Frames are appended
/// in index order with timestamps from the frame index and configured fps,
/// never wall-clock time.
pub struct AvfRecorder {
    tx: Option<Sender<AvfFrame>>,
    handle: Option<thread::JoinHandle<Result<u64>>>,
    pool: SendCf<CVPixelBufferPool>,
    in_flight: Arc<(Mutex<usize>, Condvar)>,
    died: Arc<AtomicBool>,
    next_index: u64,
    config: AvfRecorderConfig,
}

/// Resolve one of AVFoundation's weak-linked string constants.
fn avk(v: Option<&'static NSString>, name: &str) -> Result<&'static NSString> {
    v.ok_or_else(|| err(format!("AVFoundation constant {name} unavailable")))
}

/// The CV pixel-buffer keys are CFStrings; the AV settings dictionaries want
/// NSStrings. Toll-free bridged, so the cast is the sanctioned conversion.
fn cf_as_ns(s: &CFString) -> &NSString {
    let ptr: *const CFString = s;
    unsafe { &*ptr.cast::<NSString>() }
}

fn any(obj: Retained<NSNumber>) -> Retained<AnyObject> {
    unsafe { Retained::cast_unchecked(obj) }
}

fn any_str(obj: &NSString) -> Retained<AnyObject> {
    unsafe { Retained::cast_unchecked(obj.copy()) }
}

/// pts timescale: fps × 1000 keeps fractional rates (59.94 → 59940) exact
/// enough that no two frames collapse onto one timestamp.
fn timescale(fps: f64) -> i32 {
    (fps * 1000.0).round().clamp(1.0, i32::MAX as f64) as i32
}

fn pts(index: u64, ts: i32) -> CMTime {
    CMTime {
        value: index as i64 * 1000,
        timescale: ts,
        flags: CMTimeFlags::Valid,
        epoch: 0,
    }
}

fn file_type(path: &Path) -> Result<&'static AVFileType> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    let ft = match ext.as_str() {
        "mp4" => unsafe { AVFileTypeMPEG4 },
        "mov" => unsafe { AVFileTypeQuickTimeMovie },
        "m4v" => unsafe { AVFileTypeAppleM4V },
        other => {
            return Err(err(format!(
                "unsupported container .{other} for the hardware recorder \
                 (expected .mp4, .mov, or .m4v)"
            )));
        }
    };
    ft.ok_or_else(|| err("AVFoundation file type constant unavailable"))
}

fn writer_error(writer: &AVAssetWriter, context: &str) -> AvfRecordError {
    let detail = unsafe { writer.error() }
        .map(|e| e.localizedDescription().to_string())
        .unwrap_or_else(|| "no error detail".to_string());
    err(format!("{context}: {detail}"))
}

impl AvfRecorder {
    pub fn new(path: impl AsRef<Path>, config: AvfRecorderConfig) -> Result<Self> {
        let path = path.as_ref();
        if config.width == 0 || config.height == 0 {
            return Err(err("width and height must be non-zero"));
        }
        if !(config.fps.is_finite() && config.fps > 0.0 && config.fps <= 1_000_000.0) {
            return Err(err(format!(
                "fps must be a positive number, got {}",
                config.fps
            )));
        }
        let file_type = file_type(path)?;
        // AVAssetWriter refuses to overwrite; ffmpeg's behavior (and ours so
        // far) is to replace the file.
        if path.exists() {
            std::fs::remove_file(path)
                .map_err(|e| err(format!("cannot replace {}: {e}", path.display())))?;
        }
        let path_str = path
            .to_str()
            .ok_or_else(|| err("output path is not valid UTF-8"))?;
        let url = NSURL::fileURLWithPath(&NSString::from_str(path_str));

        let writer = unsafe {
            AVAssetWriter::initWithURL_fileType_error(AVAssetWriter::alloc(), &url, file_type)
        }
        .map_err(|e| err(format!("creating AVAssetWriter: {}", e.localizedDescription())))?;

        let codec_name = match config.codec {
            AvfCodec::H264 => avk(unsafe { AVVideoCodecTypeH264 }, "H264")?,
            AvfCodec::Hevc => avk(unsafe { AVVideoCodecTypeHEVC }, "HEVC")?,
        };

        // Tag the track 709 across the board: the canvas bytes are
        // sRGB-encoded, and 709-in/709-out means the RGB→YCbCr conversion is
        // a pure matrix with no transfer-function resampling — the same
        // treatment the ffmpeg path gives them, but with the file explicitly
        // tagged instead of left for players to guess.
        let color_props = NSDictionary::<NSString, AnyObject>::from_retained_objects(
            &[
                avk(unsafe { AVVideoColorPrimariesKey }, "ColorPrimariesKey")?,
                avk(unsafe { AVVideoTransferFunctionKey }, "TransferFunctionKey")?,
                avk(unsafe { AVVideoYCbCrMatrixKey }, "YCbCrMatrixKey")?,
            ],
            &[
                any_str(avk(
                    unsafe { AVVideoColorPrimaries_ITU_R_709_2 },
                    "Primaries709",
                )?),
                any_str(avk(
                    unsafe { AVVideoTransferFunction_ITU_R_709_2 },
                    "Transfer709",
                )?),
                any_str(avk(unsafe { AVVideoYCbCrMatrix_ITU_R_709_2 }, "Matrix709")?),
            ],
        );

        let compression = NSDictionary::<NSString, AnyObject>::from_retained_objects(
            &[
                avk(unsafe { AVVideoAverageBitRateKey }, "AverageBitRateKey")?,
                avk(
                    unsafe { AVVideoExpectedSourceFrameRateKey },
                    "ExpectedSourceFrameRateKey",
                )?,
                avk(
                    unsafe { AVVideoAllowFrameReorderingKey },
                    "AllowFrameReorderingKey",
                )?,
            ],
            &[
                any(NSNumber::numberWithDouble(config.bitrate_bps())),
                any(NSNumber::numberWithDouble(config.fps)),
                any(NSNumber::numberWithBool(config.bframes != Some(0))),
            ],
        );

        let settings = NSDictionary::<NSString, AnyObject>::from_retained_objects(
            &[
                avk(unsafe { AVVideoCodecKey }, "CodecKey")?,
                avk(unsafe { AVVideoWidthKey }, "WidthKey")?,
                avk(unsafe { AVVideoHeightKey }, "HeightKey")?,
                avk(unsafe { AVVideoColorPropertiesKey }, "ColorPropertiesKey")?,
                avk(
                    unsafe { AVVideoCompressionPropertiesKey },
                    "CompressionPropertiesKey",
                )?,
            ],
            &[
                any_str(codec_name),
                any(NSNumber::numberWithUnsignedInt(config.width)),
                any(NSNumber::numberWithUnsignedInt(config.height)),
                unsafe { Retained::cast_unchecked(color_props) },
                unsafe { Retained::cast_unchecked(compression) },
            ],
        );

        let media_type = unsafe { AVMediaTypeVideo }
            .ok_or_else(|| err("AVMediaTypeVideo constant unavailable"))?;
        let input = unsafe {
            AVAssetWriterInput::initWithMediaType_outputSettings(
                AVAssetWriterInput::alloc(),
                media_type,
                Some(&settings),
            )
        };
        // Offline source: readyForMoreMediaData throttles us instead of
        // dropping frames.
        unsafe { input.setExpectsMediaDataInRealTime(false) };
        if !unsafe { writer.canAddInput(&input) } {
            return Err(err("AVAssetWriter rejected the video input"));
        }
        unsafe { writer.addInput(&input) };

        // The adaptor's own pool vends the IOSurface-backed, Metal-compatible
        // BGRA buffers the GPU blits into.
        let empty = NSDictionary::<NSString, AnyObject>::new();
        let attrs = NSDictionary::<NSString, AnyObject>::from_retained_objects(
            &[
                cf_as_ns(unsafe { kCVPixelBufferPixelFormatTypeKey }),
                cf_as_ns(unsafe { kCVPixelBufferWidthKey }),
                cf_as_ns(unsafe { kCVPixelBufferHeightKey }),
                cf_as_ns(unsafe { kCVPixelBufferIOSurfacePropertiesKey }),
                cf_as_ns(unsafe { kCVPixelBufferMetalCompatibilityKey }),
            ],
            &[
                any(NSNumber::numberWithUnsignedInt(kCVPixelFormatType_32BGRA)),
                any(NSNumber::numberWithUnsignedInt(config.width)),
                any(NSNumber::numberWithUnsignedInt(config.height)),
                unsafe { Retained::cast_unchecked(empty) },
                any(NSNumber::numberWithBool(true)),
            ],
        );
        let adaptor = unsafe {
            AVAssetWriterInputPixelBufferAdaptor::initWithAssetWriterInput_sourcePixelBufferAttributes(
                AVAssetWriterInputPixelBufferAdaptor::alloc(),
                &input,
                Some(&attrs),
            )
        };

        if !unsafe { writer.startWriting() } {
            return Err(writer_error(&writer, "startWriting failed"));
        }
        unsafe { writer.startSessionAtSourceTime(pts(0, timescale(config.fps))) };

        let pool = unsafe { adaptor.pixelBufferPool() }
            .ok_or_else(|| err("adaptor vended no pixel buffer pool"))?;
        let pool = SendCf(unsafe {
            CFRetained::retain(NonNull::new_unchecked(Retained::as_ptr(&pool) as *mut _))
        });

        let (tx, rx) = channel();
        let in_flight = Arc::new((Mutex::new(0usize), Condvar::new()));
        let died = Arc::new(AtomicBool::new(false));
        let objects = WriterObjects {
            writer,
            input,
            adaptor,
        };
        let ts = timescale(config.fps);
        let died_w = Arc::clone(&died);
        let handle = thread::Builder::new()
            .name("processing_avf_encode".to_string())
            .spawn(move || {
                let r = writer_main(objects, rx, ts);
                if r.is_err() {
                    died_w.store(true, Ordering::Release);
                }
                r
            })
            .expect("failed to spawn AVF writer thread");

        Ok(Self {
            tx: Some(tx),
            handle: Some(handle),
            pool,
            in_flight,
            died,
            next_index: 0,
            config,
        })
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

    /// Frames checked out so far (the eventual frame count, minus any frames
    /// abandoned at teardown).
    pub fn frames_acquired(&self) -> u64 {
        self.next_index
    }

    /// Handle for reporting GPU completion; clone one per frame into the
    /// completion callback.
    pub fn completer(&self) -> AvfCompleter {
        AvfCompleter {
            tx: self.tx.as_ref().expect("recorder already finished").clone(),
        }
    }

    /// Check out the next frame's pixel buffer. Blocks while `MAX_IN_FLIGHT`
    /// frames are still working their way through the GPU and encoder — this
    /// is what paces the render loop, exactly like the readback ring did,
    /// except the bound is now encode throughput rather than copy throughput.
    ///
    /// `pump` is called between waits and must make GPU completion callbacks
    /// deliverable (a non-blocking wgpu device poll): the completions this
    /// wait depends on only fire while the device is being pumped, and the
    /// caller is the thread that normally pumps it by rendering.
    pub fn acquire_frame(&mut self, pump: &mut dyn FnMut()) -> Result<AvfFrame> {
        if self.died.load(Ordering::Acquire) {
            return Err(self.take_error());
        }
        let died = {
            let (count, cv) = &*self.in_flight;
            let deadline = Instant::now() + STALL_TIMEOUT;
            let mut count = count.lock().unwrap();
            loop {
                if self.died.load(Ordering::Acquire) {
                    break true;
                }
                if *count < MAX_IN_FLIGHT {
                    *count += 1;
                    break false;
                }
                if Instant::now() >= deadline {
                    return Err(err("encoder stalled: no frame completed in 10s"));
                }
                drop(count);
                pump();
                let (guard, _) = {
                    let (m, _) = &*self.in_flight;
                    let guard = m.lock().unwrap();
                    cv.wait_timeout(guard, Duration::from_millis(1)).unwrap()
                };
                count = guard;
            }
        };
        if died {
            return Err(self.take_error());
        }
        let permit = InFlightPermit(Arc::clone(&self.in_flight));

        let mut raw: *mut CVPixelBuffer = std::ptr::null_mut();
        let ret = unsafe {
            CVPixelBufferPool::create_pixel_buffer(
                None,
                &self.pool.0,
                NonNull::new_unchecked(&mut raw),
            )
        };
        if ret != kCVReturnSuccess || raw.is_null() {
            return Err(err(format!("CVPixelBufferPoolCreatePixelBuffer: {ret}")));
        }
        // Create rule: the pool hands back +1.
        let buffer = unsafe { CFRetained::from_raw(NonNull::new_unchecked(raw)) };

        for (key, value) in [
            (unsafe { kCVImageBufferColorPrimariesKey }, unsafe {
                kCVImageBufferColorPrimaries_ITU_R_709_2
            }),
            (unsafe { kCVImageBufferTransferFunctionKey }, unsafe {
                kCVImageBufferTransferFunction_ITU_R_709_2
            }),
            (unsafe { kCVImageBufferYCbCrMatrixKey }, unsafe {
                kCVImageBufferYCbCrMatrix_ITU_R_709_2
            }),
        ] {
            unsafe {
                buffer.set_attachment(key, value, CVAttachmentMode::ShouldPropagate);
            }
        }

        let surface = CVPixelBufferGetIOSurface(Some(&buffer))
            .ok_or_else(|| err("pool pixel buffer has no IOSurface backing"))?;

        let index = self.next_index;
        self.next_index += 1;
        Ok(AvfFrame {
            buffer: SendCf(buffer),
            surface: SendCf(surface),
            index,
            _permit: permit,
        })
    }

    fn take_error(&mut self) -> AvfRecordError {
        self.tx.take();
        match self.handle.take().map(|h| h.join()) {
            Some(Ok(Err(e))) => e,
            Some(Err(_)) => err("writer thread panicked"),
            _ => err("writer thread terminated unexpectedly"),
        }
    }

    /// Wait for in-flight frames, finalize the file, and return the number of
    /// frames written. `pump` as in [`Self::acquire_frame`].
    pub fn finish(mut self, pump: &mut dyn FnMut()) -> Result<u64> {
        // Wait for the GPU completions of everything checked out; permits are
        // released even for frames whose callbacks are dropped at teardown.
        {
            let (count, cv) = &*self.in_flight;
            let deadline = Instant::now() + STALL_TIMEOUT;
            loop {
                {
                    let guard = count.lock().unwrap();
                    if *guard == 0 {
                        break;
                    }
                    if Instant::now() >= deadline {
                        break; // proceed; the writer appends what it has
                    }
                    let _ = cv.wait_timeout(guard, Duration::from_millis(1)).unwrap();
                }
                pump();
            }
        }
        drop(self.tx.take());
        match self.handle.take() {
            Some(handle) => handle.join().map_err(|_| err("writer thread panicked"))?,
            None => Err(err("writer thread terminated unexpectedly")),
        }
    }
}

impl Drop for AvfRecorder {
    fn drop(&mut self) {
        // Abnormal teardown (an error aborted the recording): close our end
        // and DETACH. Joining here can deadlock — completion closures still
        // hold channel senders, and they only resolve while the dropping
        // thread pumps the device. The writer thread finalizes a playable
        // partial file on its own once the last sender goes away.
        drop(self.tx.take());
        drop(self.handle.take());
    }
}

fn append(objects: &WriterObjects, frame: AvfFrame, ts: i32) -> Result<()> {
    // Offline pacing: wait for the input to drain rather than dropping.
    let deadline = Instant::now() + STALL_TIMEOUT;
    while !unsafe { objects.input.isReadyForMoreMediaData() } {
        if unsafe { objects.writer.status() } == AVAssetWriterStatus::Failed {
            return Err(writer_error(&objects.writer, "append"));
        }
        if Instant::now() >= deadline {
            return Err(err("AVAssetWriterInput never became ready"));
        }
        thread::sleep(Duration::from_micros(500));
    }
    let ok = unsafe {
        objects
            .adaptor
            .appendPixelBuffer_withPresentationTime(&frame.buffer.0, pts(frame.index, ts))
    };
    if !ok {
        return Err(writer_error(&objects.writer, "appendPixelBuffer failed"));
    }
    Ok(())
}

fn writer_main(objects: WriterObjects, rx: Receiver<AvfFrame>, ts: i32) -> Result<u64> {
    let mut written: u64 = 0;
    // GPU completion callbacks arrive in submission order in practice, but
    // wgpu doesn't promise it; reorder by frame index to be sure.
    let mut queue: BTreeMap<u64, AvfFrame> = BTreeMap::new();
    let mut next: u64 = 0;

    while let Ok(frame) = rx.recv() {
        queue.insert(frame.index, frame);
        while let Some(frame) = queue.remove(&next) {
            append(&objects, frame, ts)?;
            next += 1;
            written += 1;
        }
    }
    // Channel closed: everything still queued is appendable, in order. Gaps
    // are frames abandoned at teardown; their pts slots simply go unused.
    for (_, frame) in queue {
        append(&objects, frame, ts)?;
        written += 1;
    }

    unsafe { objects.input.markAsFinished() };
    if written == 0 {
        // Nothing was appended; cancel instead of finalizing an empty file.
        unsafe { objects.writer.cancelWriting() };
        return Ok(0);
    }
    let (done_tx, done_rx) = channel::<()>();
    let block = block2::RcBlock::new(move || {
        let _ = done_tx.send(());
    });
    unsafe { objects.writer.finishWritingWithCompletionHandler(&block) };
    done_rx
        .recv_timeout(Duration::from_secs(60))
        .map_err(|_| err("finishWriting did not complete"))?;
    if unsafe { objects.writer.status() } != AVAssetWriterStatus::Completed {
        return Err(writer_error(&objects.writer, "finishWriting"));
    }
    Ok(written)
}
