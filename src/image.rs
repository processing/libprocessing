//! Images and fonts.

use bevy::prelude::Entity;
use bevy::render::render_resource::{Extent3d, TextureFormat};
use processing_core::error::Result;
use processing_render::{
    font_create, font_list, font_load, image_create, image_destroy, image_load, image_readback,
    image_size, image_update,
};

use crate::Color;

/// A texture you can draw with `g.image(..)` or use as a `g.texture(..)`.
#[derive(Debug)]
pub struct Image {
    pub(crate) entity: Entity,
    width: u32,
    height: u32,
}

impl Image {
    /// Loads an image relative to the sketch's asset directory.
    pub fn load(path: &str) -> Result<Self> {
        let entity = image_load(path)?;
        let (width, height) = image_size(entity)?;
        Ok(Self {
            entity,
            width,
            height,
        })
    }

    /// A blank, transparent image.
    pub fn create(width: u32, height: u32) -> Result<Self> {
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let data = vec![0u8; (width * height * 4) as usize];
        let entity = image_create(size, data, TextureFormat::Rgba8UnormSrgb)?;
        Ok(Self {
            entity,
            width,
            height,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Reads the pixels back from the GPU, row-major.
    pub fn pixels(&self) -> Result<Vec<Color>> {
        Ok(image_readback(self.entity)?
            .into_iter()
            .map(Color::from_linear)
            .collect())
    }

    /// Replaces every pixel; `pixels.len()` must be `width * height`.
    pub fn set_pixels(&mut self, pixels: &[Color]) -> Result<()> {
        let linear: Vec<_> = pixels.iter().map(|c| c.to_linear()).collect();
        image_update(self.entity, &linear)
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        let _ = image_destroy(self.entity);
    }
}

/// Anything that can be drawn with `g.image(..)` or used as a `g.texture(..)`.
pub trait AsImage {
    #[doc(hidden)]
    fn image_entity(&self) -> Entity;
}

impl AsImage for Image {
    fn image_entity(&self) -> Entity {
        self.entity
    }
}

/// A font for `g.text_font(..)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Font(pub(crate) Entity);

impl Font {
    /// Loads a font file relative to the sketch's asset directory.
    pub fn load(path: &str) -> Result<Self> {
        font_load(path).map(Self)
    }

    /// Looks up an installed system font by family name.
    pub fn system(name: &str) -> Result<Self> {
        font_create(name).map(Self)
    }

    /// Family names of the installed system fonts.
    pub fn list() -> Result<Vec<String>> {
        font_list()
    }
}
