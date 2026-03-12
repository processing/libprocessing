use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;

/// Ensure a mesh has vertex colors, inserting default white if missing.
fn ensure_vertex_colors(mesh: &mut Mesh) {
    if mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_none() {
        let vertex_count = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .map(|a| a.len())
            .unwrap_or(0);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            VertexAttributeValues::Float32x4(vec![[1.0, 1.0, 1.0, 1.0]; vertex_count]),
        );
    }
}

pub fn box_mesh(width: f32, height: f32, depth: f32) -> Mesh {
    let cuboid = bevy::math::primitives::Cuboid::new(width, height, depth);
    let mut mesh = Mesh::from(cuboid);
    ensure_vertex_colors(&mut mesh);
    mesh
}

pub fn sphere_mesh(radius: f32, sectors: u32, stacks: u32) -> Mesh {
    let sphere = bevy::math::primitives::Sphere::new(radius);
    let mut mesh = sphere.mesh().uv(sectors, stacks);
    ensure_vertex_colors(&mut mesh);
    mesh
}
