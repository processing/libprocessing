use std::ops::Deref;

use bevy::asset::embedded_asset;
use bevy::material::specialize::SpecializedMeshPipelineError;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
    MaterialPlugin,
};
use bevy::prelude::*;
use bevy::render::{
    mesh::MeshVertexBufferLayoutRef,
    render_resource::{AsBindGroup, RenderPipelineDescriptor},
    storage::ShaderBuffer,
};
use bevy::shader::ShaderRef;

use crate::render::material::UntypedMaterial;

pub struct ParticlesMaterialPlugin;

impl Plugin for ParticlesMaterialPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "particles.wgsl");
        app.add_plugins(MaterialPlugin::<ParticlesMaterial>::default());
    }
}

pub type ParticlesMaterial = ExtendedMaterial<StandardMaterial, ParticlesExtension>;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ParticlesExtensionKey {
    pub has_albedo: bool,
    pub has_emissive: bool,
}

impl From<&ParticlesExtension> for ParticlesExtensionKey {
    fn from(ext: &ParticlesExtension) -> Self {
        Self {
            has_albedo: ext.colors.is_some(),
            has_emissive: ext.emissive_colors.is_some(),
        }
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[bind_group_data(ParticlesExtensionKey)]
pub struct ParticlesExtension {
    #[storage(100, read_only)]
    pub colors: Option<Handle<ShaderBuffer>>,
    #[storage(101, read_only)]
    pub emissive_colors: Option<Handle<ShaderBuffer>>,
}

impl MaterialExtension for ParticlesExtension {
    fn fragment_shader() -> ShaderRef {
        "embedded://processing_render/particles/particles.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "embedded://processing_render/particles/particles.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(ref mut fragment) = descriptor.fragment {
            if key.bind_group_data.has_albedo {
                fragment.shader_defs.push("HAS_COLORS".into());
            }
            if key.bind_group_data.has_emissive {
                fragment.shader_defs.push("HAS_EMISSIVE_COLORS".into());
            }
        }
        Ok(())
    }
}

pub fn add_particles_materials(mut commands: Commands, meshes: Query<(Entity, &UntypedMaterial)>) {
    for (entity, handle) in meshes.iter() {
        let handle = handle.deref().clone();
        if let Ok(handle) = handle.try_typed::<ParticlesMaterial>() {
            commands
                .entity(entity)
                .insert(MeshMaterial3d::<ParticlesMaterial>(handle));
        }
    }
}
