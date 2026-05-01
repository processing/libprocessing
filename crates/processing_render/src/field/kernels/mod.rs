//! Built-in compute kernels for [`Field`](super::Field). Each kernel is a small WGSL
//! shader packaged with libprocessing as an embedded asset. Use them via `field_apply`
//! after configuring parameters via `compute_set`.

use bevy::asset::embedded_asset;
use bevy::prelude::*;

pub struct FieldKernelsPlugin;

impl Plugin for FieldKernelsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "noise.wgsl");
    }
}

pub const NOISE_PATH: &str = "embedded://processing_render/field/kernels/noise.wgsl";
