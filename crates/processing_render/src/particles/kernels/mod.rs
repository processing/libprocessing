//! Built-in compute kernels for [`Particles`](super::Particles). Each kernel is
//! a small WGSL shader packaged with libprocessing as an embedded asset. Use
//! them via `particles_apply` after configuring parameters via `compute_set`.

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
