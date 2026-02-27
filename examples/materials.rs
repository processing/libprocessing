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
    let width = 800;
    let height = 600;
    let mut glfw_ctx = GlfwContext::new(width, height)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(width, height, 1.0)?;
    let graphics = graphics_create(surface, width, height, TextureFormat::Rgba16Float)?;
    let sphere = geometry_sphere(30.0, 32, 18)?;

    graphics_mode_3d(graphics)?;
    transform_set_position(graphics, 0.0, 0.0, 600.0)?;
    transform_look_at(graphics, 0.0, 0.0, 0.0)?;

    let dir_light =
        light_create_directional(graphics, bevy::color::Color::srgb(1.0, 0.98, 0.95), 1_500.0)?;
    transform_set_position(dir_light, 300.0, 400.0, 300.0)?;
    transform_look_at(dir_light, 0.0, 0.0, 0.0)?;

    let point_light =
        light_create_point(graphics, bevy::color::Color::WHITE, 100_000.0, 800.0, 0.0)?;
    transform_set_position(point_light, 200.0, 200.0, 400.0)?;

    // Grid of materials varying roughness (x) and metallic (y)
    let cols = 11;
    let rows = 5;
    let mut materials = Vec::new();

    for row in 0..rows {
        for col in 0..cols {
            let mat = material_create_pbr()?;
            let roughness = col as f32 / (cols - 1) as f32;
            let metallic = row as f32 / (rows - 1) as f32;

            material_set(mat, "roughness", material::MaterialValue::Float(roughness))?;
            material_set(mat, "metallic", material::MaterialValue::Float(metallic))?;
            materials.push(mat);
        }
    }

    let base_color = bevy::color::Color::srgb(1.0, 0.85, 0.57);
    let spacing = 70.0;
    let offset_x = (cols - 1) as f32 * spacing / 2.0;
    let offset_y = (rows - 1) as f32 * spacing / 2.0;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.05, 0.05, 0.07)),
        )?;

        graphics_record_command(graphics, DrawCommand::Fill(base_color))?;

        for row in 0..rows {
            for col in 0..cols {
                let mat = materials[row * cols + col];

                graphics_record_command(graphics, DrawCommand::PushMatrix)?;
                graphics_record_command(
                    graphics,
                    DrawCommand::Translate {
                        x: col as f32 * spacing - offset_x,
                        y: row as f32 * spacing - offset_y,
                    },
                )?;
                graphics_record_command(graphics, DrawCommand::Material(mat))?;
                graphics_record_command(graphics, DrawCommand::Geometry(sphere))?;
                graphics_record_command(graphics, DrawCommand::PopMatrix)?;
            }
        }

        graphics_end_draw(graphics)?;
    }

    for mat in materials {
        material_destroy(mat)?;
    }

    Ok(())
}
