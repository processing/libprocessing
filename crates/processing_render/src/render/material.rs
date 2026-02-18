use bevy::{prelude::*, render::alpha::AlphaMode};
use std::ops::Deref;

#[derive(Component, Deref)]
pub struct UntypedMaterial(pub UntypedHandle);

#[derive(Clone, PartialEq, Debug)]
pub enum MaterialSource {
    Immediate(MaterialKey),
    Explicit(Entity),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MaterialKey {
    Color {
        transparent: bool,
        background_image: Option<Handle<Image>>,
    },
    Pbr {
        albedo: [u8; 4],
        roughness: u8,
        metallic: u8,
        emissive: [u8; 4],
    },
    Custom(Entity),
}

impl MaterialKey {
    pub fn simple_2d(transparent: bool) -> Self {
        Self::Color {
            transparent,
            background_image: None,
        }
    }

    pub fn with_image(image: Handle<Image>, transparent: bool) -> Self {
        Self::Color {
            transparent,
            background_image: Some(image),
        }
    }

    pub fn to_material(&self, materials: &mut ResMut<Assets<StandardMaterial>>) -> UntypedHandle {
        match self {
            MaterialKey::Color {
                transparent,
                background_image,
            } => {
                let mat = StandardMaterial {
                    base_color: Color::WHITE,
                    unlit: true,
                    cull_mode: None,
                    base_color_texture: background_image.clone(),
                    alpha_mode: if *transparent {
                        AlphaMode::Blend
                    } else {
                        AlphaMode::Opaque
                    },
                    ..default()
                };
                materials.add(mat).untyped()
            }
            MaterialKey::Pbr {
                albedo,
                roughness,
                metallic,
                emissive,
            } => {
                let base_color = Color::srgba(
                    albedo[0] as f32 / 255.0,
                    albedo[1] as f32 / 255.0,
                    albedo[2] as f32 / 255.0,
                    albedo[3] as f32 / 255.0,
                );
                let mat = StandardMaterial {
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
                };
                materials.add(mat).untyped()
            }
            MaterialKey::Custom(_entity) => {
                todo!("implement custom materials")
            }
        }
    }
}

pub fn add_standard_materials(
    mut commands: Commands,
    meshes: Query<(Entity, &UntypedMaterial)>,
) {
    for (entity, handle) in meshes.iter() {
        let handle = handle.deref().clone();
        if let Ok(handle) = handle.try_typed::<StandardMaterial>() {
            commands.entity(entity).insert(MeshMaterial3d(handle));
        }
    }
}
