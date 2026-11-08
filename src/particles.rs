use bevy::prelude::Entity;
use processing_core::error::Result;
use processing_render::particles::{
    particles_apply, particles_capacity, particles_create, particles_create_from_geometry,
    particles_destroy, particles_emit, particles_emit_gpu, particles_ensure_attribute,
    particles_kernel_age, particles_kernel_attract, particles_kernel_bounds_box,
    particles_kernel_bounds_geometry, particles_kernel_bounds_sphere, particles_kernel_drag,
    particles_kernel_field, particles_kernel_force, particles_kernel_impulse,
    particles_kernel_integrate, particles_kernel_noise, particles_kernel_orient,
    particles_kernel_transform, particles_kernel_vortex, particles_scatter_create,
    particles_scatter_volume_create,
};

use crate::graphics::expect;
use crate::{Attribute, Buffer, Compute, Geometry};

#[derive(Debug)]
pub struct Particles(pub(crate) Entity);

impl Particles {
    pub fn new(capacity: u32, attributes: &[Attribute]) -> Result<Self> {
        particles_create(capacity, attributes.iter().map(|a| a.0).collect()).map(Self)
    }

    /// One particle per vertex.
    pub fn from_geometry(geometry: &Geometry, attributes: &[Attribute]) -> Result<Self> {
        particles_create_from_geometry(geometry.0, attributes.iter().map(|a| a.0).collect())
            .map(Self)
    }

    pub fn capacity(&self) -> u32 {
        expect(particles_capacity(self.0))
    }

    /// Adds the attribute if missing.
    pub fn buffer(&self, attribute: Attribute) -> Buffer {
        Buffer::borrowed(expect(particles_ensure_attribute(self.0, attribute.0)))
    }

    pub fn apply(&mut self, compute: &Compute) {
        expect(particles_apply(self.0, compute.0));
    }

    /// Replaces the oldest when full.
    pub fn emit(&mut self, count: u32, values: &[(Attribute, &[f32])]) {
        let data = values
            .iter()
            .map(|(attribute, v)| (attribute.0, bytemuck::cast_slice(v).to_vec()))
            .collect();
        expect(particles_emit(self.0, count, data));
    }

    pub fn emit_gpu(&mut self, count: u32, compute: &Compute) {
        expect(particles_emit_gpu(self.0, count, compute.0));
    }

    pub fn noise() -> Result<Compute> {
        particles_kernel_noise().map(Compute)
    }

    pub fn transform() -> Result<Compute> {
        particles_kernel_transform().map(Compute)
    }

    pub fn attract() -> Result<Compute> {
        particles_kernel_attract().map(Compute)
    }

    pub fn drag() -> Result<Compute> {
        particles_kernel_drag().map(Compute)
    }

    pub fn vortex() -> Result<Compute> {
        particles_kernel_vortex().map(Compute)
    }

    pub fn force() -> Result<Compute> {
        particles_kernel_force().map(Compute)
    }

    pub fn integrate() -> Result<Compute> {
        particles_kernel_integrate().map(Compute)
    }

    pub fn age() -> Result<Compute> {
        particles_kernel_age().map(Compute)
    }

    pub fn bounds_sphere() -> Result<Compute> {
        particles_kernel_bounds_sphere().map(Compute)
    }

    pub fn bounds_box() -> Result<Compute> {
        particles_kernel_bounds_box().map(Compute)
    }

    pub fn bounds_geometry(geometry: &Geometry) -> Result<Compute> {
        particles_kernel_bounds_geometry(geometry.0).map(Compute)
    }

    pub fn impulse() -> Result<Compute> {
        particles_kernel_impulse().map(Compute)
    }

    pub fn orient() -> Result<Compute> {
        particles_kernel_orient().map(Compute)
    }

    pub fn field() -> Result<Compute> {
        particles_kernel_field().map(Compute)
    }

    /// For [`Particles::emit_gpu`].
    pub fn scatter_surface(geometry: &Geometry) -> Result<Compute> {
        particles_scatter_create(geometry.0).map(Compute)
    }

    /// For [`Particles::emit_gpu`].
    pub fn scatter_volume(geometry: &Geometry) -> Result<Compute> {
        particles_scatter_volume_create(geometry.0).map(Compute)
    }
}

impl Drop for Particles {
    fn drop(&mut self) {
        let _ = particles_destroy(self.0);
    }
}
