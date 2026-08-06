use bevy::prelude::*;

use crate::shader_value::ShaderValue;
use processing_core::error::{ProcessingError, Result};

pub fn set_property(
    material: &mut StandardMaterial,
    name: &str,
    value: &ShaderValue,
    texture_handle: Option<Handle<Image>>,
) -> Result<()> {
    match name {
        "base_color" | "color" => {
            let ShaderValue::Float4(c) = value else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Float4, got {value:?}"
                )));
            };
            material.base_color = Color::srgba(c[0], c[1], c[2], c[3]);
        }
        "metallic" => {
            let ShaderValue::Float(v) = value else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Float, got {value:?}"
                )));
            };
            material.metallic = *v;
        }
        "roughness" | "perceptual_roughness" => {
            let ShaderValue::Float(v) = value else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Float, got {value:?}"
                )));
            };
            material.perceptual_roughness = *v;
        }
        "reflectance" => {
            let ShaderValue::Float(v) = value else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Float, got {value:?}"
                )));
            };
            material.reflectance = *v;
        }
        "emissive" => {
            let ShaderValue::Float4(c) = value else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Float4, got {value:?}"
                )));
            };
            material.emissive = LinearRgba::new(c[0], c[1], c[2], c[3]);
        }
        "base_color_texture" | "texture" => {
            let Some(handle) = texture_handle else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "'{name}' expects Texture, got {value:?}"
                )));
            };
            material.base_color_texture = Some(handle);
        }
        _ => {
            return Err(ProcessingError::UnknownShaderProperty(name.to_string()));
        }
    }
    Ok(())
}
