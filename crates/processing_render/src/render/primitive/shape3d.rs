use bevy::prelude::*;

pub fn box_mesh(width: f32, height: f32, depth: f32) -> Mesh {
    let cuboid = bevy::math::primitives::Cuboid::new(width, height, depth);
    Mesh::from(cuboid)
}

pub fn sphere_mesh(radius: f32, sectors: u32, stacks: u32) -> Mesh {
    let sphere = bevy::math::primitives::Sphere::new(radius);
    sphere.mesh().uv(sectors, stacks)
}
