mod glfw;

use glfw::GlfwContext;
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
    let width = 400;
    let height = 400;
    let mut glfw_ctx = GlfwContext::new(width, height)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(width, height)?;
    let graphics = graphics_create(surface, width, height, TextureFormat::Rgba16Float)?;
    let box_geo = geometry_box(100.0, 100.0, 100.0)?;

    graphics_mode_3d(graphics)?;
    transform_set_position(graphics, 100.0, 100.0, 300.0)?;
    transform_look_at(graphics, 0.0, 0.0, 0.0)?;

    let shader = shader_load("shaders/custom_material.wesl")?;
    let mat = material_create_custom(shader)?;
    material_set(
        mat,
        "color",
        material::MaterialValue::Float4([1.0, 0.2, 0.4, 1.0]),
    )?;

    let mut angle = 0.0;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.1, 0.1, 0.15)),
        )?;

        graphics_record_command(graphics, DrawCommand::PushMatrix)?;
        graphics_record_command(graphics, DrawCommand::Rotate { angle })?;
        graphics_record_command(graphics, DrawCommand::Material(mat))?;
        graphics_record_command(graphics, DrawCommand::Geometry(box_geo))?;
        graphics_record_command(graphics, DrawCommand::PopMatrix)?;

        graphics_end_draw(graphics)?;

        angle += 0.02;
    }

    material_destroy(mat)?;
    shader_destroy(shader)?;

    Ok(())
}
