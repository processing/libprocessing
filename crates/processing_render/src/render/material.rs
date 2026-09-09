use bevy::math::Affine2;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;
use std::ops::Deref;

use crate::material::ProcessingMaterial;
use crate::material::custom::{CustomMaterial, CustomMaterial3d};
use crate::particles::material::ParticlesMaterial;

#[derive(Component, Deref)]
pub struct UntypedMaterial(pub UntypedHandle);

pub type ProcessingExtendedMaterial = ExtendedMaterial<StandardMaterial, ProcessingMaterial>;

#[derive(Clone, PartialEq, Debug)]
pub enum MaterialKey {
    Color {
        transparent: bool,
        background_image: Option<Handle<Image>>,
        uv_transform: Affine2,
        blend_state: Option<BlendState>,
    },
    Pbr {
        albedo: [u8; 4],
        roughness: u8,
        metallic: u8,
        emissive: [u8; 4],
        base_color_texture: Option<Handle<Image>>,
        uv_transform: Affine2,
        blend_state: Option<BlendState>,
    },
    Custom {
        entity: Entity,
        blend_state: Option<BlendState>,
    },
}

pub struct PbrFields {
    pub albedo: [u8; 4],
    pub roughness: u8,
    pub metallic: u8,
    pub emissive: [u8; 4],
    pub base_color_texture: Option<Handle<Image>>,
    pub uv_transform: Affine2,
    pub blend_state: Option<BlendState>,
}

impl Default for PbrFields {
    fn default() -> Self {
        Self {
            albedo: [255, 255, 255, 255],
            roughness: 128,
            metallic: 0,
            emissive: [0, 0, 0, 0],
            base_color_texture: None,
            uv_transform: Affine2::IDENTITY,
            blend_state: None,
        }
    }
}

impl From<PbrFields> for MaterialKey {
    fn from(f: PbrFields) -> Self {
        MaterialKey::Pbr {
            albedo: f.albedo,
            roughness: f.roughness,
            metallic: f.metallic,
            emissive: f.emissive,
            base_color_texture: f.base_color_texture,
            uv_transform: f.uv_transform,
            blend_state: f.blend_state,
        }
    }
}

impl MaterialKey {
    pub fn as_pbr(&self) -> PbrFields {
        match self {
            MaterialKey::Pbr {
                albedo,
                roughness,
                metallic,
                emissive,
                base_color_texture,
                uv_transform,
                blend_state,
            } => PbrFields {
                albedo: *albedo,
                roughness: *roughness,
                metallic: *metallic,
                emissive: *emissive,
                base_color_texture: base_color_texture.clone(),
                uv_transform: *uv_transform,
                blend_state: *blend_state,
            },
            _ => PbrFields::default(),
        }
    }

    pub fn blend_state(&self) -> Option<BlendState> {
        match self {
            MaterialKey::Color { blend_state, .. } => *blend_state,
            MaterialKey::Pbr { blend_state, .. } => *blend_state,
            MaterialKey::Custom { blend_state, .. } => *blend_state,
        }
    }

    pub(crate) fn to_standard_material(&self) -> StandardMaterial {
        match self {
            MaterialKey::Color {
                transparent,
                background_image,
                uv_transform,
                blend_state,
            } => StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                cull_mode: None,
                base_color_texture: background_image.clone(),
                uv_transform: *uv_transform,
                alpha_mode: if blend_state.is_some() || *transparent {
                    AlphaMode::Blend
                } else {
                    AlphaMode::Opaque
                },
                ..default()
            },
            MaterialKey::Pbr {
                albedo,
                roughness,
                metallic,
                emissive,
                base_color_texture,
                uv_transform,
                ..
            } => {
                let base_color = Color::srgba(
                    albedo[0] as f32 / 255.0,
                    albedo[1] as f32 / 255.0,
                    albedo[2] as f32 / 255.0,
                    albedo[3] as f32 / 255.0,
                );
                StandardMaterial {
                    base_color,
                    unlit: false,
                    cull_mode: None,
                    perceptual_roughness: *roughness as f32 / 255.0,
                    metallic: *metallic as f32 / 255.0,
                    emissive: LinearRgba::new(
                        emissive[0] as f32 / 255.0,
                        emissive[1] as f32 / 255.0,
                        emissive[2] as f32 / 255.0,
                        emissive[3] as f32 / 255.0,
                    ),
                    base_color_texture: base_color_texture.clone(),
                    uv_transform: *uv_transform,
                    ..default()
                }
            }
            MaterialKey::Custom { .. } => unreachable!(),
        }
    }

    pub fn to_material(
        &self,
        materials: &mut ResMut<Assets<ProcessingExtendedMaterial>>,
    ) -> UntypedHandle {
        let blend_state = self.blend_state();
        let base = self.to_standard_material();
        let extended = ProcessingExtendedMaterial {
            base,
            extension: ProcessingMaterial {
                blend_state,
                depth_write: None,
            },
        };
        materials.add(extended).untyped()
    }
}

/// Project `UntypedMaterial` onto the typed material component bevy renders
/// with, and clear the slots the handle's type does *not* call for.
///
/// Draw entities outlive a single draw — every instanced body draw of one
/// particle system shares `particles_data.draw_entity`, and mesh draws reuse
/// their entity across frames — so a typed component left behind by an earlier
/// material of a different type would keep the entity queued in that material's
/// pipeline and overdraw the current one. Inserting without removing is what
/// made a particle system drawn first with a constant-albedo (extended)
/// material and then with a per-particle (particles) material render with the
/// stale extended material.
pub fn sync_typed_materials(mut commands: Commands, meshes: Query<(Entity, &UntypedMaterial)>) {
    for (entity, handle) in meshes.iter() {
        let handle = handle.deref().clone();
        let mut entity = commands.entity(entity);

        if let Ok(handle) = handle.clone().try_typed::<ProcessingExtendedMaterial>() {
            entity
                .insert(MeshMaterial3d(handle))
                .remove::<CustomMaterial3d>()
                .remove::<MeshMaterial3d<ParticlesMaterial>>();
        } else if let Ok(handle) = handle.clone().try_typed::<CustomMaterial>() {
            entity
                .insert(CustomMaterial3d(handle))
                .remove::<MeshMaterial3d<ProcessingExtendedMaterial>>()
                .remove::<MeshMaterial3d<ParticlesMaterial>>();
        } else if let Ok(handle) = handle.try_typed::<ParticlesMaterial>() {
            entity
                .insert(MeshMaterial3d(handle))
                .remove::<MeshMaterial3d<ProcessingExtendedMaterial>>()
                .remove::<CustomMaterial3d>();
        }
    }
}
