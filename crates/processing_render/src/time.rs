use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::time::Time;

pub fn frame_count(count: Option<Res<FrameCount>>) -> u32 {
    count.map(|c| c.0).unwrap_or(0)
}

pub fn delta_secs(time: Option<Res<Time>>) -> f32 {
    time.map(|t| t.delta_secs()).unwrap_or(0.0)
}

pub fn elapsed_secs(time: Option<Res<Time>>) -> f32 {
    time.map(|t| t.elapsed_secs()).unwrap_or(0.0)
}
