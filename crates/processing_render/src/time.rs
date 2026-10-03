use bevy::prelude::*;
use bevy::time::Time;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct ProcessingFrame(pub u32);

pub fn frame_count(frame: Option<Res<ProcessingFrame>>) -> u32 {
    frame.map(|f| f.0).unwrap_or(0)
}

pub fn set_frame_count(In(n): In<u32>, mut frame: ResMut<ProcessingFrame>) {
    frame.0 = n;
}

/// Frames per second, smoothed like Processing's `frameRate`: starts at 60 and
/// moves 10% toward the instantaneous rate each frame.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ProcessingFrameRate(pub f32);

impl Default for ProcessingFrameRate {
    fn default() -> Self {
        Self(60.0)
    }
}

pub fn advance_frame_count(
    mut frame: ResMut<ProcessingFrame>,
    mut rate: ResMut<ProcessingFrameRate>,
    time: Option<Res<Time>>,
) {
    frame.0 = frame.0.wrapping_add(1);
    if let Some(dt) = time.map(|t| t.delta_secs()).filter(|dt| *dt > 0.0) {
        rate.0 = rate.0 * 0.9 + 0.1 / dt;
    }
}

pub fn frame_rate(rate: Res<ProcessingFrameRate>) -> f32 {
    rate.0
}

pub fn delta_secs(time: Option<Res<Time>>) -> f32 {
    time.map(|t| t.delta_secs()).unwrap_or(0.0)
}

pub fn elapsed_secs(time: Option<Res<Time>>) -> f32 {
    time.map(|t| t.elapsed_secs()).unwrap_or(0.0)
}
