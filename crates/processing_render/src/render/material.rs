use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;
use std::ops::Deref;

use crate::material::ProcessingMaterial;
use crate::material::custom::{CustomMaterial, CustomMaterial3d};

#[derive(Component, Deref)]
pub struct UntypedMaterial(pub UntypedHandle);

pub type ProcessingExtendedMaterial = ExtendedMaterial<StandardMaterial, ProcessingMaterial>;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MaterialKey {
    Color {
        transparent: bool,
        background_image: Option<Handle<Image>>,
        blend_state: Option<BlendState>,
    },
    Pbr {
        albedo: [u8; 4],
        roughness: u8,
        metallic: u8,
        emissive: [u8; 4],
        blend_state: Option<BlendState>,
    },
    Custom {
        entity: Entity,
        blend_state: Option<BlendState>,
    },
}

impl MaterialKey {
    pub fn blend_state(&self) -> Option<BlendState> {
        match self {
            MaterialKey::Color { blend_state, .. } => *blend_state,
            MaterialKey::Pbr { blend_state, .. } => *blend_state,
            MaterialKey::Custom { blend_state, .. } => *blend_state,
        }
    }

    pub fn to_standard_material(&self) -> StandardMaterial {
        match self {
            MaterialKey::Color {
                transparent,
                background_image,
                blend_state,
            } => StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                cull_mode: None,
                base_color_texture: background_image.clone(),
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
            extension: ProcessingMaterial { blend_state },
        };
        materials.add(extended).untyped()
    }
}

pub fn add_processing_materials(mut commands: Commands, meshes: Query<(Entity, &UntypedMaterial)>) {
    for (entity, handle) in meshes.iter() {
        let handle = handle.deref().clone();
        if let Ok(handle) = handle.try_typed::<ProcessingExtendedMaterial>() {
            commands.entity(entity).insert(MeshMaterial3d(handle));
        }
    }
}

pub fn add_custom_materials(mut commands: Commands, meshes: Query<(Entity, &UntypedMaterial)>) {
    for (entity, handle) in meshes.iter() {
        let handle = handle.deref().clone();
        if let Ok(handle) = handle.try_typed::<CustomMaterial>() {
            commands.entity(entity).insert(CustomMaterial3d(handle));
        }
    }
}
