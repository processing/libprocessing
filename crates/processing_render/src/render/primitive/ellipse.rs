use bevy::prelude::*;
use lyon::{geom::Point, path::Path};

use crate::render::primitive::{StrokeConfig, TessellationMode, tessellate_path};

// magic number for cubic bezier approximation of a quarter circle.
// kappa = 4 * (sqrt(2) - 1) / 3
const KAPPA: f32 = 0.5522847498;

fn ellipse_path(cx: f32, cy: f32, w: f32, h: f32) -> Path {
    let rx = w / 2.0;
    let ry = h / 2.0;
    let kx = rx * KAPPA;
    let ky = ry * KAPPA;

    let mut b = Path::builder();

    b.begin(Point::new(cx, cy - ry));

    // rc -> rc
    b.cubic_bezier_to(
        Point::new(cx + kx, cy - ry),
        Point::new(cx + rx, cy - ky),
        Point::new(cx + rx, cy),
    );
    // rc -> bc
    b.cubic_bezier_to(
        Point::new(cx + rx, cy + ky),
        Point::new(cx + kx, cy + ry),
        Point::new(cx, cy + ry),
    );
    // bc -> lc
    b.cubic_bezier_to(
        Point::new(cx - kx, cy + ry),
        Point::new(cx - rx, cy + ky),
        Point::new(cx - rx, cy),
    );
    // lc -> tc
    b.cubic_bezier_to(
        Point::new(cx - rx, cy - ky),
        Point::new(cx - kx, cy - ry),
        Point::new(cx, cy - ry),
    );

    b.end(true);
    b.build()
}

pub fn ellipse(
    mesh: &mut Mesh,
    cx: f32,
    cy: f32,
    w: f32,
    h: f32,
    color: Color,
    mode: TessellationMode,
    stroke_config: &StrokeConfig,
) {
    let path = ellipse_path(cx, cy, w, h);
    tessellate_path(mesh, &path, color, mode, stroke_config);
}
