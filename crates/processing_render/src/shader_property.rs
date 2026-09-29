use bevy::prelude::*;
use bevy::reflect::ReflectMut;
use bevy_naga_reflect::dynamic_shader::DynamicShader;
use bevy_naga_reflect::reflect::ParameterCategory;

use crate::compute::{Buffer, Compute, MeshBindingRef};
use crate::image::Image as PImage;
use crate::material::custom::{apply_field_coerced, apply_reflect_field, shader_value_to_reflect};
use crate::particles::grid::Grid;
use crate::render::filter::Filter;
use crate::shader_value::ShaderValue;
use processing_core::error::{ProcessingError, Result};

fn require_read_only_storage(shader: &DynamicShader, name: &str, kind: &str) -> Result<()> {
    let category = shader
        .reflection()
        .parameter(name)
        .map(|p| p.category())
        .ok_or_else(|| ProcessingError::UnknownShaderProperty(name.to_string()))?;
    let ParameterCategory::Storage { read_only } = category else {
        return Err(ProcessingError::InvalidArgument(format!(
            "property `{name}` expects {category:?}, got {kind}",
        )));
    };
    if !read_only {
        return Err(ProcessingError::InvalidArgument(format!(
            "property `{name}` is read-write; {kind} buffers can only bind as read-only",
        )));
    }
    Ok(())
}

fn bind_compute_mesh(compute: &mut Compute, name: String, value: ShaderValue) -> Result<()> {
    match value {
        ShaderValue::MeshAttribute(geom, attribute) => {
            require_read_only_storage(&compute.shader, &name, "mesh attribute")?;
            compute
                .mesh_bindings
                .insert(name, MeshBindingRef::Attribute { geom, attribute });
        }
        ShaderValue::MeshIndex(geom) => {
            require_read_only_storage(&compute.shader, &name, "mesh index")?;
            compute
                .mesh_bindings
                .insert(name, MeshBindingRef::Index { geom });
        }
        _ => unreachable!("bind_compute_mesh only handles MeshAttribute/MeshIndex"),
    }
    Ok(())
}

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
                ParameterCategory::Texture
                    | ParameterCategory::StorageTexture
                    | ParameterCategory::Sampler
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

/// Binds `{name}_{suffix}` buffers and the fields of the `{name}` uniform struct.
fn bind_struct(
    shader: &mut DynamicShader,
    name: &str,
    type_name: &str,
    buffers: &[(&str, Entity)],
    values: &[(&str, ShaderValue)],
    p_buffers: &mut Query<&mut Buffer>,
    p_images: &Query<&PImage>,
) -> Result<()> {
    for (suffix, buffer) in buffers {
        let binding = format!("{name}_{suffix}");
        apply_shader_value(
            shader,
            &binding,
            ShaderValue::Buffer(*buffer),
            p_buffers,
            p_images,
        )?;
    }
    let wrong_type = || {
        ProcessingError::InvalidArgument(format!(
            "`{name}` must be a `particles::{type_name}` uniform"
        ))
    };
    let param = shader
        .field_mut(name)
        .ok_or_else(|| ProcessingError::UnknownShaderProperty(name.to_string()))?;
    let ReflectMut::Struct(fields) = param.reflect_mut() else {
        return Err(wrong_type());
    };
    for (field, value) in values {
        let target = fields.field_mut(field).ok_or_else(wrong_type)?;
        apply_field_coerced(target, &*shader_value_to_reflect(value)?);
    }
    Ok(())
}

pub fn set_property(
    In((entity, name, value)): In<(Entity, String, ShaderValue)>,
    mut computes: Query<&mut Compute>,
    mut filters: Query<&mut Filter>,
    mut p_buffers: Query<&mut Buffer>,
    p_images: Query<&PImage>,
    grids: Query<&Grid>,
) -> Result<bool> {
    let shader = if let Ok(compute) = computes.get_mut(entity) {
        if let ShaderValue::MeshAttribute(..) | ShaderValue::MeshIndex(..) = value {
            bind_compute_mesh(compute.into_inner(), name, value)?;
            return Ok(true);
        }
        &mut compute.into_inner().shader
    } else if let Ok(filter) = filters.get_mut(entity) {
        &mut filter.into_inner().shader
    } else {
        return Ok(false);
    };
    match value {
        ShaderValue::Grid(grid) => {
            let grid = grids.get(grid).map_err(|_| ProcessingError::GridNotFound)?;
            bind_struct(
                shader,
                &name,
                "Grid",
                &[("offsets", grid.offsets), ("sorted", grid.sorted)],
                &[
                    ("origin", ShaderValue::Float3(grid.params.min)),
                    ("cell_size", ShaderValue::Float(grid.params.cell_size)),
                    ("dims", ShaderValue::UInt3(grid.params.dims)),
                ],
                &mut p_buffers,
                &p_images,
            )?;
        }
        other => apply_shader_value(shader, &name, other, &mut p_buffers, &p_images)?,
    }
    Ok(true)
}
