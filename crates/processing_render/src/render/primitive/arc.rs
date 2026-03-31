use bevy::prelude::*;
use lyon::{geom::Point, path::Path};

use crate::render::command::ArcMode;
use crate::render::primitive::{StrokeConfig, TessellationMode, tessellate_path};

fn arc_path(
    cx: f32,
    cy: f32,
    w: f32,
    h: f32,
    start: f32,
    stop: f32,
    mode: ArcMode,
    for_fill: bool,
) -> Path {
    let rx = w / 2.0;
    let ry = h / 2.0;
    let angle_range = stop - start;

    // adaptive segment count based on arc size and angle
    let circumference =
        std::f32::consts::PI * (3.0 * (rx + ry) - ((3.0 * rx + ry) * (rx + 3.0 * ry)).sqrt());
    let arc_length = circumference * (angle_range.abs() / (2.0 * std::f32::consts::PI));
    let num_segments = (arc_length / 2.0).clamp(16.0, 256.0) as u32;

    let mut builder = Path::builder();

    let first_point = Point::new(cx + rx * start.cos(), cy + ry * start.sin());

    match mode {
        ArcMode::Pie if for_fill => {
            builder.begin(Point::new(cx, cy));
            builder.line_to(first_point);
        }
        _ => {
            builder.begin(first_point);
        }
    }

    for i in 1..=num_segments {
        let t = i as f32 / num_segments as f32;
        let angle = start + angle_range * t;
        builder.line_to(Point::new(cx + rx * angle.cos(), cy + ry * angle.sin()));
    }

    let should_close = match mode {
        ArcMode::Pie => true,
        ArcMode::Chord => true,
        ArcMode::Open => for_fill,
    };

    builder.end(should_close);
    builder.build()
}

pub fn arc_fill(
    mesh: &mut Mesh,
    cx: f32,
    cy: f32,
    w: f32,
    h: f32,
    start: f32,
    stop: f32,
    mode: ArcMode,
    color: Color,
    stroke_config: &StrokeConfig,
) {
    let path = arc_path(cx, cy, w, h, start, stop, mode, true);
    tessellate_path(mesh, &path, color, TessellationMode::Fill, stroke_config);
}

pub fn arc_stroke(
    mesh: &mut Mesh,
    cx: f32,
    cy: f32,
    w: f32,
    h: f32,
    start: f32,
    stop: f32,
    mode: ArcMode,
    color: Color,
    weight: f32,
    stroke_config: &StrokeConfig,
) {
    let path = arc_path(cx, cy, w, h, start, stop, mode, false);
    tessellate_path(
        mesh,
        &path,
        color,
        TessellationMode::Stroke(weight),
        stroke_config,
    );
}
