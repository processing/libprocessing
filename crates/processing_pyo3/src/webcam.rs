use bevy::prelude::Entity;
use processing_webcam::{
    WebcamFormat, webcam_create, webcam_create_with_format, webcam_destroy, webcam_image,
    webcam_is_connected, webcam_resolution,
};
use pyo3::{exceptions::PyRuntimeError, prelude::*};

use crate::graphics::Image;

#[pyclass(unsendable)]
pub struct Webcam {
    entity: Entity,
}

#[pymethods]
impl Webcam {
    #[new]
    #[pyo3(signature = (width=None, height=None, framerate=None))]
    pub fn new(width: Option<u32>, height: Option<u32>, framerate: Option<u32>) -> PyResult<Self> {
        let entity = match (width, height, framerate) {
            (Some(w), Some(h), Some(fps)) => webcam_create_with_format(WebcamFormat::Exact {
                resolution: bevy::math::UVec2::new(w, h),
                framerate: fps,
            }),
            (Some(w), Some(h), None) => {
                webcam_create_with_format(WebcamFormat::Resolution(bevy::math::UVec2::new(w, h)))
            }
            (None, None, Some(fps)) => webcam_create_with_format(WebcamFormat::FrameRate(fps)),
            _ => webcam_create(),
        }
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        Ok(Self { entity })
    }

    pub fn is_connected(&self) -> PyResult<bool> {
        webcam_is_connected(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn resolution(&self) -> PyResult<(u32, u32)> {
        webcam_resolution(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn image(&self) -> PyResult<Image> {
        let entity =
            webcam_image(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Image::from_entity(entity))
    }
}

impl Drop for Webcam {
    fn drop(&mut self) {
        let _ = webcam_destroy(self.entity);
    }
}
