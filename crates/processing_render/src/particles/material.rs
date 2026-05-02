//! `ParticlesMaterial` — `ExtendedMaterial<StandardMaterial, ParticlesExtension>`
//! whose per-particle color comes from a storage buffer indexed by the
//! per-instance tag (set to slot index by the pack pass).
//!
//! Lit vs unlit is just the `unlit` flag on the base `StandardMaterial`;
//! `apply_pbr_lighting` short-circuits to `base_color * particle_colors[tag]`
//! when `unlit = true`, so a single extension serves both cases.

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

/// PBR material extended with a per-particle color buffer. Set the base
/// `StandardMaterial`'s `unlit` flag to switch between lit and unlit behavior;
/// the rest of the material works identically either way.
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

/// Sibling of `add_processing_materials` / `add_custom_materials`. Promotes
/// `UntypedMaterial(handle)` entities whose handle is a [`ParticlesMaterial`]
/// to having the typed `MeshMaterial3d<ParticlesMaterial>` component required
/// by the render pipeline.
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
