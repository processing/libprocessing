use bevy::prelude::*;
use bevy_naga_reflect::dynamic_shader::DynamicShader;
use bevy_naga_reflect::reflect::ParameterCategory;

use crate::compute::{Buffer, Compute};
use crate::image::Image as PImage;
use crate::material::custom::{apply_reflect_field, shader_value_to_reflect};
use crate::render::filter::Filter;
use crate::shader_value::ShaderValue;
use processing_core::error::{ProcessingError, Result};

pub(crate) fn apply_shader_value(
    shader: &mut DynamicShader,
    name: &str,
    value: ShaderValue,
    p_buffers: &mut Query<&mut Buffer>,
    p_images: &Query<&PImage>,
) -> Result<()> {
    match value {
        ShaderValue::Buffer(buf_entity) => {
            let category = shader
                .reflection()
                .parameter(name)
                .map(|p| p.category())
                .ok_or_else(|| ProcessingError::UnknownShaderProperty(name.to_string()))?;
            let ParameterCategory::Storage { read_only } = category else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "property `{name}` expects {category:?}, got Buffer",
                )));
            };
            let mut buffer = p_buffers
                .get_mut(buf_entity)
                .map_err(|_| ProcessingError::BufferNotFound)?;
            shader.insert(name, buffer.handle.clone());
            if !read_only {
                buffer.bound_rw = true;
            }
            Ok(())
        }
        ShaderValue::Texture(img_entity) => {
            let category = shader
                .reflection()
                .parameter(name)
                .map(|p| p.category())
                .ok_or_else(|| ProcessingError::UnknownShaderProperty(name.to_string()))?;
            if !matches!(
                category,
                ParameterCategory::Texture | ParameterCategory::StorageTexture
            ) {
                return Err(ProcessingError::InvalidArgument(format!(
                    "property `{name}` expects {category:?}, got Texture",
                )));
            }
            let image = p_images
                .get(img_entity)
                .map_err(|_| ProcessingError::ImageNotFound)?;
            shader.insert(name, image.handle.clone());
            Ok(())
        }
        v => {
            let reflect_value = shader_value_to_reflect(&v)?;
            apply_reflect_field(shader, name, &*reflect_value)
        }
    }
}

pub fn set_property(
    In((entity, name, value)): In<(Entity, String, ShaderValue)>,
    mut computes: Query<&mut Compute>,
    mut filters: Query<&mut Filter>,
    mut p_buffers: Query<&mut Buffer>,
    p_images: Query<&PImage>,
) -> Result<bool> {
    if let Ok(mut compute) = computes.get_mut(entity) {
        apply_shader_value(&mut compute.shader, &name, value, &mut p_buffers, &p_images)?;
        return Ok(true);
    }
    if let Ok(mut filter) = filters.get_mut(entity) {
        apply_shader_value(&mut filter.shader, &name, value, &mut p_buffers, &p_images)?;
        return Ok(true);
    }
    Ok(false)
}
