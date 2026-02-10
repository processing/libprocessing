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
    let mut glfw_ctx = GlfwContext::new(400, 400)?;
    init(Config::default())?;

    let width = 400;
    let height = 400;
    let scale_factor = 1.0;

    let surface = glfw_ctx.create_surface(width, height, scale_factor)?;
    let graphics = graphics_create(surface, width, height)?;
    let box_geo = geometry_box(100.0, 100.0, 100.0)?;

    // We will only declare lights in `setup`
    // rather than calling some sort of `light()` method inside of `draw`

    // create a directional light
    let _dir_light =
        light_create_directional(graphics, bevy::color::Color::srgb(0.35, 0.25, 0.5), 500.0)?;

    // create a point light
    let point_light_a = light_create_point(
        graphics,
        bevy::color::Color::srgb(1.0, 0.5, 0.25),
        1_000_000.0,
        200.0,
        0.5,
    )?;
    transform_set_position(point_light_a, -25.0, 5.0, 51.0)?;
    transform_look_at(point_light_a, 0.0, 0.0, 0.0)?;

    // create another point light
    let point_light_b = light_create_point(
        graphics,
        bevy::color::Color::srgb(0.0, 0.5, 0.75),
        2_000_000.0,
        200.0,
        0.25,
    )?;
    transform_set_position(point_light_b, 0.0, 5.0, 50.5)?;
    transform_look_at(point_light_b, 0.0, 0.0, 0.0)?;

    // and a spot light, too!
    let spot_light = light_create_spot(
        graphics,
        bevy::color::Color::srgb(0.25, 0.8, 0.19),
        15.0 * 1_000_000.0,
        200.0,
        0.84,
        0.0,
        core::f32::consts::FRAC_PI_4,
    )?;
    transform_set_position(spot_light, 40.0, 0.0, 70.0)?;
    transform_look_at(spot_light, 0.0, 0.0, 0.0)?;

    graphics_mode_3d(graphics)?;
    transform_set_position(graphics, 100.0, 100.0, 300.0)?;
    transform_look_at(graphics, 0.0, 0.0, 0.0)?;

    let mut angle = 0.0;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.18, 0.20, 0.15)),
        )?;

        graphics_record_command(graphics, DrawCommand::PushMatrix)?;
        graphics_record_command(graphics, DrawCommand::Rotate { angle })?;
        graphics_record_command(graphics, DrawCommand::Geometry(box_geo))?;
        graphics_record_command(graphics, DrawCommand::PopMatrix)?;

        graphics_end_draw(graphics)?;

        angle += 0.02;
    }
    Ok(())
}
