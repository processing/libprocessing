use bevy::prelude::Color;
use processing_glfw::GlfwContext;

use processing::prelude::*;
use processing_render::render::command::DrawCommand;

fn main() {
    match sketch() {
        Ok(_) => {
            eprintln!("Sketch completed successfully");
            exit(0).unwrap();
        }
        Err(e) => {
            eprintln!("Sketch error: {:?}", e);
            exit(1).unwrap();
        }
    };
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(600, 400)?;
    init(Config::default())?;

    let width = 600;
    let height = 400;
    let surface = glfw_ctx.create_surface(width, height)?;
    let graphics = graphics_create(surface, width, height, TextureFormat::Rgba16Float)?;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        // White background
        graphics_record_command(graphics, DrawCommand::BackgroundColor(Color::WHITE))?;

        // Set fill to black for text
        graphics_record_command(graphics, DrawCommand::Fill(Color::BLACK))?;

        // Set text size
        graphics_record_command(graphics, DrawCommand::TextSize(32.0))?;

        // Draw text
        graphics_record_command(
            graphics,
            DrawCommand::Text {
                content: "Hello, Processing!".to_string(),
                x: 50.0,
                y: 100.0,
                z: 0.0,
                max_w: None,
                max_h: None,
            },
        )?;

        // Smaller text
        graphics_record_command(graphics, DrawCommand::TextSize(18.0))?;

        graphics_record_command(
            graphics,
            DrawCommand::Text {
                content: "Text rendering with parley + skrifa + lyon".to_string(),
                x: 50.0,
                y: 160.0,
                z: 0.0,
                max_w: None,
                max_h: None,
            },
        )?;

        // Text with bounding box (word wrap)
        graphics_record_command(graphics, DrawCommand::TextSize(16.0))?;

        graphics_record_command(
            graphics,
            DrawCommand::Text {
                content: "This is a longer paragraph of text that should wrap within its bounding box. The text uses parley for layout, skrifa for glyph outlines, and lyon for tessellation.".to_string(),
                x: 50.0,
                y: 220.0,
                z: 0.0,
                max_w: Some(300.0),
                max_h: Some(200.0),
            },
        )?;

        // Center-aligned text
        graphics_record_command(
            graphics,
            DrawCommand::TextAlign {
                h: TextAlignH::Center,
                v: TextAlignV::Top,
            },
        )?;
        graphics_record_command(graphics, DrawCommand::TextSize(24.0))?;

        graphics_record_command(
            graphics,
            DrawCommand::Text {
                content: "Centered".to_string(),
                x: 450.0,
                y: 100.0,
                z: 0.0,
                max_w: Some(200.0),
                max_h: None,
            },
        )?;

        graphics_end_draw(graphics)?;
    }
    Ok(())
}
