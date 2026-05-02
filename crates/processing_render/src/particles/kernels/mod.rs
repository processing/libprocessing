//! Built-in compute kernels for [`Particles`](super::Particles), embedded as
//! assets and dispatched via `particles_apply`.

use bevy::asset::embedded_asset;
use bevy::prelude::*;

pub struct ParticlesKernelsPlugin;

impl Plugin for ParticlesKernelsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "noise.wgsl");
        embedded_asset!(app, "transform.wgsl");
    }
}

pub const NOISE_PATH: &str = "embedded://processing_render/particles/kernels/noise.wgsl";
pub const TRANSFORM_PATH: &str =
    "embedded://processing_render/particles/kernels/transform.wgsl";
