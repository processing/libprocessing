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

pub fn cylinder_mesh(radius: f32, height: f32, detail: u32) -> Mesh {
    let cylinder = bevy::math::primitives::Cylinder::new(radius, height);
    let mut mesh = cylinder.mesh().resolution(detail).build();
    ensure_vertex_colors(&mut mesh);
    mesh
}

pub fn cone_mesh(radius: f32, height: f32, detail: u32) -> Mesh {
    let cone = bevy::math::primitives::Cone::new(radius, height);
    let mut mesh = cone.mesh().resolution(detail).build();
    ensure_vertex_colors(&mut mesh);
    mesh
}

pub fn torus_mesh(radius: f32, tube_radius: f32, major_segments: u32, minor_segments: u32) -> Mesh {
    let torus = bevy::math::primitives::Torus::new(tube_radius, radius);
    let mut mesh = torus
        .mesh()
        .major_resolution(major_segments as usize)
        .minor_resolution(minor_segments as usize)
        .build();
    ensure_vertex_colors(&mut mesh);
    mesh
}

pub fn capsule_mesh(radius: f32, length: f32, detail: u32) -> Mesh {
    let capsule = bevy::math::primitives::Capsule3d::new(radius, length);
    let mut mesh = capsule
        .mesh()
        .longitudes(detail)
        .latitudes(detail / 2)
        .build();
    ensure_vertex_colors(&mut mesh);
    mesh
}

pub fn conical_frustum_mesh(radius_top: f32, radius_bottom: f32, height: f32, detail: u32) -> Mesh {
    let frustum = bevy::math::primitives::ConicalFrustum {
        radius_top,
        radius_bottom,
        height,
    };
    let mut mesh = frustum.mesh().resolution(detail).build();
    ensure_vertex_colors(&mut mesh);
    mesh
}

pub fn tetrahedron_mesh(radius: f32) -> Mesh {
    let r = radius;
    let tetrahedron = bevy::math::primitives::Tetrahedron::new(
        bevy::math::Vec3::new(r, r, r),
        bevy::math::Vec3::new(r, -r, -r),
        bevy::math::Vec3::new(-r, r, -r),
        bevy::math::Vec3::new(-r, -r, r),
    );
    let mut mesh = Mesh::from(tetrahedron);
    ensure_vertex_colors(&mut mesh);
    mesh
}

pub fn plane_mesh(width: f32, height: f32) -> Mesh {
    let plane = bevy::math::primitives::Plane3d::default();
    let mut mesh = plane.mesh().size(width, height).build();
    ensure_vertex_colors(&mut mesh);
    mesh
}
