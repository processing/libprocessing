use processing_video::{VideoRecorder, VideoRecorderConfig};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 48;
const FPS: f64 = 30.0;
const FRAMES: u64 = 30;

fn gradient_frame(index: u64) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(WIDTH as usize * HEIGHT as usize * 4);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let r = (x * 255 / WIDTH) as u8;
            let g = (y * 255 / HEIGHT) as u8;
            let b = ((index * 8) % 256) as u8;
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }
    pixels
}

#[test]
fn encode_decode_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.mp4");

    let config = VideoRecorderConfig::new(WIDTH, HEIGHT, FPS);
    let mut recorder = VideoRecorder::new(&path, config).unwrap();
    for i in 0..FRAMES {
        recorder.record_frame(gradient_frame(i)).unwrap();
    }
    assert_eq!(recorder.finish().unwrap(), FRAMES);

    let mut decoder = video_rs::Decoder::new(path.as_path()).unwrap();
    assert_eq!(decoder.size(), (WIDTH, HEIGHT));
    let time_base = decoder.time_base();
    let (tb_num, tb_den) = (time_base.numerator() as f64, time_base.denominator() as f64);

    let mut decoded = 0u64;
    let mut last_pts = 0.0f64;
    loop {
        match decoder.decode_raw() {
            Ok(frame) => {
                if let Some(pts) = frame.pts() {
                    last_pts = pts as f64 * tb_num / tb_den;
                }
                decoded += 1;
            }
            Err(video_rs::Error::DecodeExhausted | video_rs::Error::ReadExhausted) => break,
            Err(e) => panic!("decode error after {decoded} frames: {e}"),
        }
    }
    assert_eq!(decoded, FRAMES);
    let expected_last = (FRAMES - 1) as f64 / FPS;
    assert!(
        (last_pts - expected_last).abs() < 1.5 / FPS,
        "last pts {last_pts} should be near {expected_last}"
    );
}

#[test]
fn zero_frames_is_an_error_and_removes_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.mp4");
    let recorder = VideoRecorder::new(&path, VideoRecorderConfig::new(WIDTH, HEIGHT, FPS)).unwrap();
    let err = recorder.finish().unwrap_err();
    assert!(matches!(err, processing_video::VideoRecordError::NoFrames));
    assert!(!path.exists());
}

#[test]
fn crf_controls_output_size() {
    let dir = tempfile::tempdir().unwrap();
    let mut sizes = Vec::new();
    for (name, crf) in [("hi.mp4", 18u8), ("lo.mp4", 40u8)] {
        let path = dir.path().join(name);
        let config = VideoRecorderConfig::new(WIDTH, HEIGHT, FPS).with_crf(crf);
        let mut recorder = VideoRecorder::new(&path, config).unwrap();
        // Incompressible noise, so quantization dominates the file size.
        let mut rng: u32 = 1;
        for _ in 0..FRAMES {
            let mut pixels = Vec::with_capacity(WIDTH as usize * HEIGHT as usize * 4);
            for _ in 0..(WIDTH * HEIGHT) {
                rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                let [a, b, c, _] = rng.to_le_bytes();
                pixels.extend_from_slice(&[a, b, c, 255]);
            }
            recorder.record_frame(pixels).unwrap();
        }
        recorder.finish().unwrap();
        sizes.push(std::fs::metadata(&path).unwrap().len());
    }
    assert!(
        sizes[0] > sizes[1] * 2,
        "crf 18 ({}) should be much larger than crf 40 ({})",
        sizes[0],
        sizes[1]
    );
}

/// VideoToolbox hardware encode: macOS-only, so gate on the codec actually
/// being present in the linked ffmpeg rather than on the platform.
#[test]
fn videotoolbox_roundtrip() {
    use processing_video::RecorderCodec;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out_vt.mp4");

    let config = VideoRecorderConfig::new(WIDTH, HEIGHT, FPS)
        .with_codec(RecorderCodec::H264VideoToolbox)
        .with_bitrate(2_000_000);
    let mut recorder = match VideoRecorder::new(&path, config) {
        Ok(r) => r,
        Err(processing_video::VideoRecordError::CodecUnavailable { .. }) => {
            eprintln!("h264_videotoolbox not available; skipping");
            return;
        }
        Err(e) => panic!("recorder create failed: {e}"),
    };
    for i in 0..FRAMES {
        recorder.record_frame(gradient_frame(i)).unwrap();
    }
    assert_eq!(recorder.finish().unwrap(), FRAMES);

    let mut decoder = video_rs::Decoder::new(path.as_path()).unwrap();
    assert_eq!(decoder.size(), (WIDTH, HEIGHT));
    let mut decoded = 0u64;
    loop {
        match decoder.decode_raw() {
            Ok(_) => decoded += 1,
            Err(video_rs::Error::DecodeExhausted | video_rs::Error::ReadExhausted) => break,
            Err(e) => panic!("decode error after {decoded} frames: {e}"),
        }
    }
    assert_eq!(decoded, FRAMES);
}

/// Hardware-encoder session creation at Deep Space delivery resolutions.
/// Apple's hardware H.264 encoder rejects dimensions above 4096px (session
/// error -12903), which includes the 3840x4320 per-surface files — those are
/// allowed to fail with a Create error. HEVC must work at both delivery
/// sizes.
#[test]
fn videotoolbox_delivery_resolutions() {
    use processing_video::{RecorderCodec, VideoRecordError};

    let dir = tempfile::tempdir().unwrap();
    let cases = [
        (RecorderCodec::H264VideoToolbox, 3840u32, 4320u32),
        (RecorderCodec::HevcVideoToolbox, 3840, 4320),
        (RecorderCodec::HevcVideoToolbox, 7680, 4320),
    ];
    for (i, (codec, w, h)) in cases.into_iter().enumerate() {
        let path = dir.path().join(format!("delivery_{i}.mp4"));
        let config = VideoRecorderConfig::new(w, h, 60.0).with_codec(codec);
        let mut recorder = match VideoRecorder::new(&path, config) {
            Ok(r) => r,
            Err(VideoRecordError::CodecUnavailable { .. }) => {
                eprintln!("{codec:?} not available; skipping");
                continue;
            }
            Err(e @ VideoRecordError::Create { .. })
                if codec == RecorderCodec::H264VideoToolbox =>
            {
                // Hardware-dependent: Apple's H.264 encoder caps at 4096px.
                eprintln!("{codec:?} at {w}x{h}: {e}");
                continue;
            }
            Err(e) => panic!("{codec:?} at {w}x{h}: {e}"),
        };
        let frame = vec![128u8; w as usize * h as usize * 4];
        for _ in 0..3 {
            recorder.record_frame(frame.clone()).unwrap();
        }
        assert_eq!(recorder.finish().unwrap(), 3, "{codec:?} at {w}x{h}");
    }
}

/// The VideoToolbox BGRA fast path (no swscale — hardware colorspace
/// conversion) must produce a decodable file like the NV12 path.
#[test]
fn videotoolbox_bgra_direct_roundtrip() {
    use processing_video::{RecorderCodec, RecorderPixelFormat};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out_bgra.mp4");
    let config = VideoRecorderConfig::new(WIDTH, HEIGHT, FPS)
        .with_codec(RecorderCodec::H264VideoToolbox)
        .with_pixel_format(RecorderPixelFormat::Bgra8)
        .with_bitrate(2_000_000);
    let mut recorder = match VideoRecorder::new(&path, config) {
        Ok(r) => r,
        Err(processing_video::VideoRecordError::CodecUnavailable { .. }) => {
            eprintln!("h264_videotoolbox not available; skipping");
            return;
        }
        Err(e) => panic!("recorder create failed: {e}"),
    };
    for i in 0..FRAMES {
        recorder.record_frame(gradient_frame(i)).unwrap(); // bytes reinterpreted as BGRA — fine for a roundtrip
    }
    assert_eq!(recorder.finish().unwrap(), FRAMES);

    let mut decoder = video_rs::Decoder::new(path.as_path()).unwrap();
    assert_eq!(decoder.size(), (WIDTH, HEIGHT));
    let mut decoded = 0u64;
    loop {
        match decoder.decode_raw() {
            Ok(_) => decoded += 1,
            Err(video_rs::Error::DecodeExhausted | video_rs::Error::ReadExhausted) => break,
            Err(e) => panic!("decode error after {decoded} frames: {e}"),
        }
    }
    assert_eq!(decoded, FRAMES);
}
