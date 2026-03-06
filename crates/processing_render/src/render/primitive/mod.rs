mod rect;
mod shape3d;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use lyon::{
    path::Path,
    tessellation::{
        FillOptions, FillTessellator, LineCap, LineJoin, StrokeOptions, StrokeTessellator,
    },
};
pub use rect::rect;
pub use shape3d::{box_mesh, sphere_mesh};

use super::command::{StrokeCapMode, StrokeJoinMode};
use super::mesh_builder::MeshBuilder;

pub enum TessellationMode {
    Fill,
    Stroke(f32),
}

#[derive(Debug, Clone, Copy)]
pub struct StrokeConfig {
    pub line_cap: StrokeCapMode,
    pub line_join: StrokeJoinMode,
}

impl Default for StrokeConfig {
    fn default() -> Self {
        Self {
            line_cap: StrokeCapMode::Round,
            line_join: StrokeJoinMode::Round,
        }
    }
}

impl StrokeCapMode {
    pub fn to_lyon(self) -> LineCap {
        match self {
            Self::Round => LineCap::Round,
            Self::Square => LineCap::Square,
            Self::Project => LineCap::Butt,
        }
    }
}

impl StrokeJoinMode {
    pub fn to_lyon(self) -> LineJoin {
        match self {
            Self::Round => LineJoin::Round,
            Self::Miter => LineJoin::Miter,
            Self::Bevel => LineJoin::Bevel,
        }
    }
}

pub fn tessellate_path(
    mesh: &mut Mesh,
    path: &Path,
    color: Color,
    mode: TessellationMode,
    stroke_config: &StrokeConfig,
) {
    let mut builder = MeshBuilder::new(mesh, color);
    match mode {
        TessellationMode::Fill => {
            let mut tessellator = FillTessellator::new();
            tessellator
                .tessellate_path(path, &FillOptions::default(), &mut builder)
                .expect("Failed to tessellate fill");
        }
        TessellationMode::Stroke(weight) => {
            let mut tessellator = StrokeTessellator::new();
            let options = StrokeOptions::default()
                .with_line_width(weight)
                .with_line_cap(stroke_config.line_cap.to_lyon())
                .with_line_join(stroke_config.line_join.to_lyon());

            tessellator
                .tessellate_path(path, &options, &mut builder)
                .expect("Failed to tessellate stroke");
        }
    }
}

pub fn empty_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new());
    mesh.insert_indices(Indices::U32(Vec::new()));

    mesh
}
