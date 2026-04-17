//! Cycles through all implemented filters (2s each) applied to a color palette.
//!
//! Bottom-right blue rect is drawn AFTER the filter, so it should always stay
//! pure blue — that's the Processing 4 immediate-mode semantic: `filter()` bakes
//! into the current canvas; subsequent draws land on top unfiltered.
use std::time::Instant;

use bevy::color::Color;
use processing_glfw::GlfwContext;

use processing::prelude::*;
use processing_render::render::command::DrawCommand;

fn main() {
    match sketch() {
        Ok(_) => exit(0).unwrap(),
        Err(e) => {
            eprintln!("Sketch error: {:?}", e);
            exit(1).unwrap();
        }
    };
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(400, 400)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(400, 400)?;
    let graphics = graphics_create(surface, 400, 400, TextureFormat::Rgba16Float)?;

    let palette = [
        (40.0, 40.0, Color::srgb(1.0, 0.2, 0.2)),
        (200.0, 40.0, Color::srgb(0.2, 1.0, 0.2)),
        (40.0, 200.0, Color::srgb(0.2, 0.4, 1.0)),
        (200.0, 200.0, Color::srgb(0.95, 0.85, 0.2)),
    ];
    let filters = [
        FilterKind::Invert,
        FilterKind::Gray,
        FilterKind::Threshold { cutoff: 0.5 },
        FilterKind::Posterize { levels: 4 },
        FilterKind::Opaque,
    ];

    let start = Instant::now();

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;
        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
        )?;
        graphics_record_command(graphics, DrawCommand::NoStroke)?;

        for &(x, y, color) in &palette {
            graphics_record_command(graphics, DrawCommand::Fill(color))?;
            graphics_record_command(
                graphics,
                DrawCommand::Rect {
                    x,
                    y,
                    w: 160.0,
                    h: 160.0,
                    radii: [0.0; 4],
                },
            )?;
        }

        let idx = (start.elapsed().as_secs() / 2) as usize % filters.len();
        graphics_apply_filter(graphics, FilterOp::new(filters[idx]))?;

        // Drawn AFTER the filter: should always render pure blue regardless of cycle.
        graphics_record_command(graphics, DrawCommand::Fill(Color::srgb(0.0, 0.0, 1.0)))?;
        graphics_record_command(
            graphics,
            DrawCommand::Rect {
                x: 160.0,
                y: 160.0,
                w: 80.0,
                h: 80.0,
                radii: [0.0; 4],
            },
        )?;

        graphics_end_draw(graphics)?;
    }
    Ok(())
}
