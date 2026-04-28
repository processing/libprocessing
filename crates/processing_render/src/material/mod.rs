pub mod custom;
pub mod pbr;

use crate::compute;
use crate::render::material::UntypedMaterial;
use crate::shader_value::ShaderValue;
use bevy::material::descriptor::RenderPipelineDescriptor;
use bevy::material::specialize::SpecializedMeshPipelineError;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, BlendState};
use bevy::shader::ShaderRef;
use bevy_naga_reflect::reflect::ParameterCategory;
use processing_core::error::{self, ProcessingError};

pub struct ProcessingMaterialPlugin;

impl Plugin for ProcessingMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy::pbr::MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, ProcessingMaterial>,
        >::default());

        let world = app.world_mut();
        let handle = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                unlit: true,
                cull_mode: None,
                base_color: Color::WHITE,
                ..default()
            });
        let entity = world.spawn(UntypedMaterial(handle.untyped())).id();
        world.insert_resource(DefaultMaterial(entity));
    }
}

#[derive(Resource)]
pub struct DefaultMaterial(pub Entity);

pub fn create_pbr(
    mut commands: Commands,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ProcessingMaterial>>>,
) -> Entity {
    let handle = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            unlit: false,
            cull_mode: None,
            ..default()
        },
        extension: ProcessingMaterial { blend_state: None },
    });
    commands.spawn(UntypedMaterial(handle.untyped())).id()
}

pub fn set_property(
    In((entity, name, value)): In<(Entity, String, ShaderValue)>,
    material_handles: Query<&UntypedMaterial>,
    mut extended_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ProcessingMaterial>>>,
    mut custom_materials: ResMut<Assets<custom::CustomMaterial>>,
    mut p_buffers: Query<&mut compute::Buffer>,
) -> error::Result<()> {
    let untyped = material_handles
        .get(entity)
        .map_err(|_| ProcessingError::MaterialNotFound)?;

    if let Ok(handle) = untyped
        .0
        .clone()
        .try_typed::<ExtendedMaterial<StandardMaterial, ProcessingMaterial>>()
    {
        let mut extended = extended_materials
            .get_mut(&handle)
            .ok_or(ProcessingError::MaterialNotFound)?;
        return pbr::set_property(&mut extended.base, &name, &value);
    }

    if let Ok(handle) = untyped.0.clone().try_typed::<custom::CustomMaterial>() {
        let mut mat = custom_materials
            .get_mut(&handle)
            .ok_or(ProcessingError::MaterialNotFound)?;

        if let ShaderValue::Buffer(buf_entity) = &value {
            let mut buffer = p_buffers
                .get_mut(*buf_entity)
                .map_err(|_| ProcessingError::BufferNotFound)?;

            let category = mat
                .shader
                .reflection()
                .parameter(&name)
                .map(|p| p.category())
                .ok_or_else(|| ProcessingError::UnknownShaderProperty(name.clone()))?;

            let ParameterCategory::Storage { read_only } = category else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "property `{name}` expects {category:?}, got Buffer"
                )));
            };
            mat.shader.insert(&name, buffer.handle.clone());
            if !read_only {
                buffer.bound_rw = true;
            }
            return Ok(());
        }

        return custom::set_property(&mut mat, &name, &value);
    }

    Err(ProcessingError::MaterialNotFound)
}

pub fn destroy(
    In(entity): In<Entity>,
    mut commands: Commands,
    material_handles: Query<&UntypedMaterial>,
    mut extended_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ProcessingMaterial>>>,
    mut custom_materials: ResMut<Assets<custom::CustomMaterial>>,
) -> error::Result<()> {
    let untyped = material_handles
        .get(entity)
        .map_err(|_| ProcessingError::MaterialNotFound)?;
    if let Ok(handle) = untyped
        .0
        .clone()
        .try_typed::<ExtendedMaterial<StandardMaterial, ProcessingMaterial>>()
    {
        extended_materials.remove(&handle);
    }
    if let Ok(handle) = untyped.0.clone().try_typed::<custom::CustomMaterial>() {
        custom_materials.remove(&handle);
    }
    commands.entity(entity).despawn();
    Ok(())
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
#[bind_group_data(ProcessingMaterialKey)]
pub struct ProcessingMaterial {
    pub blend_state: Option<BlendState>,
}

#[repr(C)]
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct ProcessingMaterialKey {
    blend_state: Option<BlendState>,
}

impl From<&ProcessingMaterial> for ProcessingMaterialKey {
    fn from(mat: &ProcessingMaterial) -> Self {
        ProcessingMaterialKey {
            blend_state: mat.blend_state,
        }
    }
}

impl MaterialExtension for ProcessingMaterial {
    fn vertex_shader() -> ShaderRef {
        <StandardMaterial as Material>::vertex_shader()
    }

    fn fragment_shader() -> ShaderRef {
        <StandardMaterial as Material>::fragment_shader()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> std::result::Result<(), SpecializedMeshPipelineError> {
        if let Some(blend_state) = key.bind_group_data.blend_state {
            // this should never be null but we have to check it anyway
            if let Some(fragment_state) = &mut descriptor.fragment {
                fragment_state.targets.iter_mut().for_each(|target| {
                    if let Some(target) = target {
                        target.blend = Some(blend_state);
                    }
                });
            }
        }
        Ok(())
    }
}
