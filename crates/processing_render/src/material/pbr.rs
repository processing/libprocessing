use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;

use super::MaterialValue;
use crate::error::{ProcessingError, Result};

/// Set a property on a StandardMaterial by name.
pub fn set_property(
    material: &mut StandardMaterial,
    name: &str,
    value: &MaterialValue,
) -> Result<()> {
    match name {
        "base_color" | "color" => {
            let MaterialValue::Float4(c) = value else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Float4, got {value:?}"
                )));
            };
            material.base_color = Color::srgba(c[0], c[1], c[2], c[3]);
        }
        "metallic" => {
            let MaterialValue::Float(v) = value else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Float, got {value:?}"
                )));
            };
            material.metallic = *v;
        }
        "roughness" | "perceptual_roughness" => {
            let MaterialValue::Float(v) = value else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Float, got {value:?}"
                )));
            };
            material.perceptual_roughness = *v;
        }
        "reflectance" => {
            let MaterialValue::Float(v) = value else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Float, got {value:?}"
                )));
            };
            material.reflectance = *v;
        }
        "emissive" => {
            let MaterialValue::Float4(c) = value else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Float4, got {value:?}"
                )));
            };
            material.emissive = LinearRgba::new(c[0], c[1], c[2], c[3]);
        }
        "unlit" => {
            let MaterialValue::Float(v) = value else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Float, got {value:?}"
                )));
            };
            material.unlit = *v > 0.5;
        }
        "double_sided" => {
            let MaterialValue::Float(v) = value else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Float, got {value:?}"
                )));
            };
            material.double_sided = *v > 0.5;
        }
        "alpha_mode" => {
            let MaterialValue::Int(v) = value else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Int, got {value:?}"
                )));
            };
            material.alpha_mode = match v {
                0 => AlphaMode::Opaque,
                // TODO: allow configuring the alpha cutoff value
                1 => AlphaMode::Mask(0.5),
                2 => AlphaMode::Blend,
                3 => AlphaMode::Premultiplied,
                4 => AlphaMode::Add,
                5 => AlphaMode::Multiply,
                _ => {
                    return Err(ProcessingError::InvalidArgument(format!(
                        "unknown alpha_mode value: {v}"
                    )));
                }
            };
        }
        _ => {
            return Err(ProcessingError::UnknownMaterialProperty(name.to_string()));
        }
    }
    Ok(())
}
