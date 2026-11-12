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
            material.base_color = match value {
                // plain numbers are colors as written, i.e. sRGB
                ShaderValue::Float4(c) => Color::srgba(c[0], c[1], c[2], c[3]),
                ShaderValue::Color(c) => *c,
                _ => {
                    return Err(ProcessingError::InvalidArgument(format!(
                        "'{name}' expects a color, got {value:?}"
                    )));
                }
            };
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
            material.emissive = match value {
                ShaderValue::Float4(c) => LinearRgba::new(c[0], c[1], c[2], c[3]),
                ShaderValue::Color(c) => c.to_linear(),
                _ => {
                    return Err(ProcessingError::InvalidArgument(format!(
                        "'{name}' expects a color, got {value:?}"
                    )));
                }
            };
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
