//! `FieldColorMaterial` — unlit material that reads a per-particle color from a
//! storage buffer indexed by the per-instance tag (set to slot index by the pack pass).

use std::ops::Deref;

use bevy::asset::embedded_asset;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::{render_resource::AsBindGroup, storage::ShaderBuffer};
use bevy::shader::ShaderRef;

use crate::render::material::UntypedMaterial;

pub struct FieldColorMaterialPlugin;

impl Plugin for FieldColorMaterialPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "field_color.wgsl");
        embedded_asset!(app, "field_pbr.wgsl");
        app.add_plugins(MaterialPlugin::<FieldColorMaterial>::default());
        app.add_plugins(MaterialPlugin::<FieldPbrMaterial>::default());
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct FieldColorMaterial {
    #[storage(0, read_only)]
    pub colors: Handle<ShaderBuffer>,
}

impl Material for FieldColorMaterial {
    fn vertex_shader() -> ShaderRef {
        "embedded://processing_render/field/field_color.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://processing_render/field/field_color.wgsl".into()
    }
}

#[derive(Component, Clone)]
pub struct FieldColorMaterial3d(pub Handle<FieldColorMaterial>);

impl bevy::asset::AsAssetId for FieldColorMaterial3d {
    type Asset = FieldColorMaterial;
    fn as_asset_id(&self) -> AssetId<Self::Asset> {
        self.0.id()
    }
}

/// Sibling to `add_processing_materials` / `add_custom_materials`. Promotes
/// `UntypedMaterial(handle)` entities whose handle is a [`FieldColorMaterial`]
/// to having the typed `MeshMaterial3d<FieldColorMaterial>` component required
/// by the render pipeline.
pub fn add_field_color_materials(
    mut commands: Commands,
    meshes: Query<(Entity, &UntypedMaterial)>,
) {
    for (entity, handle) in meshes.iter() {
        let handle = handle.deref().clone();
        if let Ok(handle) = handle.try_typed::<FieldColorMaterial>() {
            commands
                .entity(entity)
                .insert(MeshMaterial3d::<FieldColorMaterial>(handle));
        }
    }
}

/// PBR-lit per-particle color material. Wraps `StandardMaterial` via
/// `ExtendedMaterial` so the user gets standard PBR lighting behavior on top
/// of per-particle albedo from a storage buffer.
pub type FieldPbrMaterial = ExtendedMaterial<StandardMaterial, FieldPbrExtension>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct FieldPbrExtension {
    #[storage(100, read_only)]
    pub colors: Handle<ShaderBuffer>,
}

impl MaterialExtension for FieldPbrExtension {
    fn fragment_shader() -> ShaderRef {
        "embedded://processing_render/field/field_pbr.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "embedded://processing_render/field/field_pbr.wgsl".into()
    }
}

pub fn add_field_pbr_materials(
    mut commands: Commands,
    meshes: Query<(Entity, &UntypedMaterial)>,
) {
    for (entity, handle) in meshes.iter() {
        let handle = handle.deref().clone();
        if let Ok(handle) = handle.try_typed::<FieldPbrMaterial>() {
            commands
                .entity(entity)
                .insert(MeshMaterial3d::<FieldPbrMaterial>(handle));
        }
    }
}
