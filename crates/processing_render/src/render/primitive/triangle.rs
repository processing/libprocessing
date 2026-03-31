use bevy::{
    mesh::{Indices, VertexAttributeValues},
    prelude::*,
};
use lyon::{geom::Point, path::Path};

use crate::render::primitive::{StrokeConfig, TessellationMode, tessellate_path};

fn triangle_path(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> Path {
    let mut b = Path::builder();
    b.begin(Point::new(x1, y1));
    b.line_to(Point::new(x2, y2));
    b.line_to(Point::new(x3, y3));
    b.end(true);
    b.build()
}

pub fn triangle(
    mesh: &mut Mesh,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    color: Color,
    mode: TessellationMode,
    stroke_config: &StrokeConfig,
) {
    if matches!(mode, TessellationMode::Fill) {
        simple_triangle(mesh, x1, y1, x2, y2, x3, y3, color);
    } else {
        let path = triangle_path(x1, y1, x2, y2, x3, y3);
        tessellate_path(mesh, &path, color, mode, stroke_config);
    }
}

fn simple_triangle(
    mesh: &mut Mesh,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    color: Color,
) {
    let base_idx = if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        positions.len() as u32
    } else {
        0
    };

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        positions.push([x1, y1, 0.0]);
        positions.push([x2, y2, 0.0]);
        positions.push([x3, y3, 0.0]);
    }

    if let Some(VertexAttributeValues::Float32x4(colors)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
    {
        let color_array = color.to_srgba().to_f32_array();
        for _ in 0..3 {
            colors.push(color_array);
        }
    }

    if let Some(VertexAttributeValues::Float32x3(normals)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
    {
        for _ in 0..3 {
            normals.push([0.0, 0.0, 1.0]);
        }
    }

    if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        uvs.push([0.0, 0.0]);
        uvs.push([1.0, 0.0]);
        uvs.push([0.5, 1.0]);
    }

    if let Some(Indices::U32(indices)) = mesh.indices_mut() {
        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
    }
}
