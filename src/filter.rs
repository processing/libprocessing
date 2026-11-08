use bevy::prelude::Entity;
use processing_core::error::Result;
use processing_render::shader_value::ShaderValue;
use processing_render::{
    filter_blur, filter_dilate, filter_erode, filter_gray, filter_invert, filter_opaque,
    filter_posterize, filter_set, filter_threshold,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Filter {
    Invert,
    Gray,
    /// Cutoff 0-1.
    Threshold(f32),
    /// Levels 2-255.
    Posterize(u32),
    /// Radius in pixels.
    Blur(f32),
    Opaque,
    Erode,
    Dilate,
}

impl Filter {
    pub(crate) fn entity(self) -> Result<Entity> {
        let (entity, param) = match self {
            Filter::Invert => (filter_invert()?, None),
            Filter::Gray => (filter_gray()?, None),
            Filter::Threshold(cutoff) => (
                filter_threshold()?,
                Some(("cutoff", ShaderValue::Float(cutoff))),
            ),
            Filter::Posterize(levels) => (
                filter_posterize()?,
                Some(("levels", ShaderValue::UInt(levels))),
            ),
            Filter::Blur(radius) => (filter_blur()?, Some(("radius", ShaderValue::Float(radius)))),
            Filter::Opaque => (filter_opaque()?, None),
            Filter::Erode => (filter_erode()?, None),
            Filter::Dilate => (filter_dilate()?, None),
        };
        if let Some((name, value)) = param {
            filter_set(entity, name, value)?;
        }
        Ok(entity)
    }
}
