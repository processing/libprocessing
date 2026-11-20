use bevy::prelude::Entity;
use processing_core::error::Result;
use processing_webcam::{
    webcam_create, webcam_destroy, webcam_image, webcam_is_connected, webcam_resolution,
};

use crate::AsImage;
use crate::graphics::expect;

/// The default camera.
#[derive(Debug)]
pub struct Webcam(Entity);

impl Webcam {
    pub fn new() -> Result<Self> {
        webcam_create().map(Self)
    }

    pub fn connected(&self) -> bool {
        expect(webcam_is_connected(self.0))
    }

    pub fn resolution(&self) -> (u32, u32) {
        expect(webcam_resolution(self.0))
    }
}

impl AsImage for Webcam {
    fn image_entity(&self) -> Entity {
        expect(webcam_image(self.0))
    }
}

impl Drop for Webcam {
    fn drop(&mut self) {
        let _ = webcam_destroy(self.0);
    }
}
