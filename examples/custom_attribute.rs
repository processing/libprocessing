mod glfw;

use glfw::GlfwContext;
use processing::prelude::*;
use processing_render::geometry::{AttributeFormat, Topology};
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
    let mut glfw_ctx = GlfwContext::new(600, 600)?;
    init()?;

    let width = 600;
    let height = 600;
    let scale_factor = 1.0;

    let surface = glfw_ctx.create_surface(width, height, scale_factor)?;
    let graphics = graphics_create(surface, width, height)?;

    let custom_attr = geometry_attribute_create("Custom", AttributeFormat::Float)?;

    let layout = geometry_layout_create()?;
    geometry_layout_add_position(layout)?;
    geometry_layout_add_normal(layout)?;
    geometry_layout_add_color(layout)?;
    geometry_layout_add_attribute(layout, custom_attr)?;

    let mesh = geometry_create_with_layout(layout, Topology::LineStrip)?;

    geometry_color(mesh, 1.0, 0.0, 0.0, 1.0)?;
    geometry_normal(mesh, 0.0, 0.0, 1.0)?;
    geometry_attribute_float(mesh, custom_attr, 0.0)?;
    geometry_vertex(mesh, -50.0, -50.0, 0.0)?;

    geometry_color(mesh, 0.0, 1.0, 0.0, 1.0)?;
    geometry_attribute_float(mesh, custom_attr, 0.5)?;
    geometry_vertex(mesh, 50.0, -50.0, 0.0)?;

    geometry_color(mesh, 0.0, 0.0, 1.0, 1.0)?;
    geometry_attribute_float(mesh, custom_attr, 1.0)?;
    geometry_vertex(mesh, 0.0, 50.0, 0.0)?;

    geometry_index(mesh, 0)?;
    geometry_index(mesh, 1)?;
    geometry_index(mesh, 2)?;
    geometry_index(mesh, 0)?;

    graphics_mode_3d(graphics)?;
    graphics_camera_position(graphics, 0.0, 0.0, 200.0)?;
    graphics_camera_look_at(graphics, 0.0, 0.0, 0.0)?;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;
        graphics_record_command(graphics, DrawCommand::Geometry(mesh))?;
        graphics_end_draw(graphics)?;
    }
    Ok(())
}
