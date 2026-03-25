use bevy::prelude::*;
use lyon::{geom::Point, path::Path};

use crate::render::primitive::{StrokeConfig, TessellationMode, tessellate_path};

pub fn line(
    mesh: &mut Mesh,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: Color,
    weight: f32,
    stroke_config: &StrokeConfig,
) {
    let mut builder = Path::builder();
    builder.begin(Point::new(x1, y1));
    builder.line_to(Point::new(x2, y2));
    builder.end(false);
    let path = builder.build();
    tessellate_path(
        mesh,
        &path,
        color,
        TessellationMode::Stroke(weight),
        stroke_config,
    );
}
