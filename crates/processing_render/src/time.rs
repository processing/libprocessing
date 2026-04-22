use bevy::prelude::*;
use bevy::time::Time;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct ProcessingFrame(pub u32);

pub fn frame_count(frame: Option<Res<ProcessingFrame>>) -> u32 {
    frame.map(|f| f.0).unwrap_or(0)
}

pub fn advance_frame_count(mut frame: ResMut<ProcessingFrame>) {
    frame.0 = frame.0.wrapping_add(1);
}

pub fn delta_secs(time: Option<Res<Time>>) -> f32 {
    time.map(|t| t.delta_secs()).unwrap_or(0.0)
}

pub fn elapsed_secs(time: Option<Res<Time>>) -> f32 {
    time.map(|t| t.elapsed_secs()).unwrap_or(0.0)
}
