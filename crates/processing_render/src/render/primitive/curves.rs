use bevy::prelude::*;
use lyon::{geom::Point, path::Path};

use crate::render::primitive::{StrokeConfig, TessellationMode, tessellate_path};

/// Draw a standalone cubic bezier curve.
pub fn bezier(
    mesh: &mut Mesh,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    x4: f32,
    y4: f32,
    color: Color,
    weight: f32,
    stroke_config: &StrokeConfig,
) {
    let mut builder = Path::builder();
    builder.begin(Point::new(x1, y1));
    builder.cubic_bezier_to(Point::new(x2, y2), Point::new(x3, y3), Point::new(x4, y4));
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

/// Draw a standalone Catmull-Rom curve segment.
pub fn curve(
    mesh: &mut Mesh,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    x4: f32,
    y4: f32,
    color: Color,
    weight: f32,
    stroke_config: &StrokeConfig,
) {
    // see https://en.wikipedia.org/wiki/Catmull%E2%80%93Rom_spline#Converting_to_B%C3%A9zier_curve
    let cp1x = x2 + (x3 - x1) / 6.0;
    let cp1y = y2 + (y3 - y1) / 6.0;
    let cp2x = x3 - (x4 - x2) / 6.0;
    let cp2y = y3 - (y4 - y2) / 6.0;

    let mut builder = Path::builder();
    builder.begin(Point::new(x2, y2));
    builder.cubic_bezier_to(
        Point::new(cp1x, cp1y),
        Point::new(cp2x, cp2y),
        Point::new(x3, y3),
    );
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
