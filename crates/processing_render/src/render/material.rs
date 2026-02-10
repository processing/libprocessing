use bevy::{prelude::*, render::alpha::AlphaMode};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MaterialKey {
    pub transparent: bool,
    pub background_image: Option<Handle<Image>>,
}

impl MaterialKey {
    pub fn to_material(&self) -> StandardMaterial {
        StandardMaterial {
            base_color: Color::WHITE,
            unlit: false,
            cull_mode: None,
            base_color_texture: self.background_image.clone(),
            alpha_mode: if self.transparent {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            ..default()
        }
    }
}
