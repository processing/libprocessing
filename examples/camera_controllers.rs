use processing_glfw::GlfwContext;

use bevy::math::Vec3;
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

    let surface = glfw_ctx.create_surface(width, height)?;
    let graphics = graphics_create(surface, width, height, TextureFormat::Rgba16Float)?;
    let box_geo = geometry_box(100.0, 100.0, 100.0)?;

    graphics_mode_3d(graphics)?;
    graphics_orbit_camera(graphics)?;

    let dir_light =
        light_create_directional(graphics, bevy::color::Color::srgb(1.0, 0.98, 0.95), 1_500.0)?;
    transform_set_position(dir_light, Vec3::new(300.0, 400.0, 300.0))?;
    transform_look_at(dir_light, Vec3::ZERO)?;

    let mut angle: f32 = 0.0;
    let mut mode = 0u8;

    while glfw_ctx.poll_events() {
        if input_key_just_pressed(KeyCode::Digit1)? {
            graphics_mode_3d(graphics)?;
            graphics_orbit_camera(graphics)?;
            mode = 0;
        }
        if input_key_just_pressed(KeyCode::Digit2)? {
            graphics_mode_3d(graphics)?;
            graphics_free_camera(graphics)?;
            mode = 1;
        }
        if input_key_just_pressed(KeyCode::Digit3)? {
            graphics_mode_2d(graphics)?;
            graphics_pan_camera(graphics)?;
            mode = 2;
        }

        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.05, 0.05, 0.07)),
        )?;

        if mode < 2 {
            graphics_record_command(
                graphics,
                DrawCommand::Fill(bevy::color::Color::srgb(1.0, 0.85, 0.57)),
            )?;
            graphics_record_command(graphics, DrawCommand::Roughness(0.3))?;
            graphics_record_command(graphics, DrawCommand::Metallic(0.8))?;
            graphics_record_command(graphics, DrawCommand::PushMatrix)?;
            graphics_record_command(graphics, DrawCommand::Rotate { angle })?;
            graphics_record_command(graphics, DrawCommand::Geometry(box_geo))?;
            graphics_record_command(graphics, DrawCommand::PopMatrix)?;
        } else {
            graphics_record_command(
                graphics,
                DrawCommand::Fill(bevy::color::Color::srgb(0.8, 0.3, 0.2)),
            )?;
            graphics_record_command(
                graphics,
                DrawCommand::Rect {
                    x: 300.0,
                    y: 200.0,
                    w: 200.0,
                    h: 200.0,
                    radii: [0.0; 4],
                },
            )?;
        }

        graphics_end_draw(graphics)?;
        angle += 0.02;
    }
    Ok(())
}
