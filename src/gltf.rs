use bevy::prelude::Entity;
use processing_core::error::Result;
use processing_render::{
    gltf_camera, gltf_geometry, gltf_light, gltf_load, gltf_material, gltf_material_names,
    gltf_mesh_names,
};

use crate::{Geometry, Graphics, Light, Material};

#[derive(Debug)]
pub struct Gltf(Entity);

impl Gltf {
    pub fn load(g: &Graphics, path: &str) -> Result<Self> {
        gltf_load(g.entity, path).map(Self)
    }

    pub fn geometry(&self, name: &str) -> Result<Geometry> {
        gltf_geometry(self.0, name).map(Geometry)
    }

    pub fn material(&self, name: &str) -> Result<Material> {
        gltf_material(self.0, name).map(Material)
    }

    pub fn mesh_names(&self) -> Result<Vec<String>> {
        gltf_mesh_names(self.0)
    }

    pub fn material_names(&self) -> Result<Vec<String>> {
        gltf_material_names(self.0)
    }

    /// Moves the camera to the scene's camera.
    pub fn camera(&self, index: usize) -> Result<()> {
        gltf_camera(self.0, index)
    }

    pub fn light(&self, index: usize) -> Result<Light> {
        gltf_light(self.0, index).map(Light)
    }
}
