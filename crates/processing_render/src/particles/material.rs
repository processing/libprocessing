//! Per-particle albedo on top of `StandardMaterial`. The `unlit` flag on the
//! base material toggles between lit and unlit; `apply_pbr_lighting`
//! short-circuits when set.

use std::ops::Deref;

use bevy::asset::embedded_asset;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::{render_resource::AsBindGroup, storage::ShaderBuffer};
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

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct ParticlesExtension {
    #[storage(100, read_only)]
    pub colors: Handle<ShaderBuffer>,
}

impl MaterialExtension for ParticlesExtension {
    fn fragment_shader() -> ShaderRef {
        "embedded://processing_render/particles/particles.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "embedded://processing_render/particles/particles.wgsl".into()
    }
}

/// Promote `UntypedMaterial(handle)` to `MeshMaterial3d<ParticlesMaterial>`
/// where the handle's type matches. Sibling of `add_processing_materials`.
pub fn add_particles_materials(
    mut commands: Commands,
    meshes: Query<(Entity, &UntypedMaterial)>,
) {
    for (entity, handle) in meshes.iter() {
        let handle = handle.deref().clone();
        if let Ok(handle) = handle.try_typed::<ParticlesMaterial>() {
            commands
                .entity(entity)
                .insert(MeshMaterial3d::<ParticlesMaterial>(handle));
        }
    }
}
