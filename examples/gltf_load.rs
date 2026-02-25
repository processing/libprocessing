mod glfw;

use glfw::GlfwContext;
use processing::prelude::*;
use processing_render::material::MaterialValue;
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
    let graphics = graphics_create(surface, width, height)?;

    let gltf = gltf_load(graphics, "gltf/Duck.glb")?;
    let duck = gltf_geometry(gltf, "LOD3spShape")?;
    let duck_mat = gltf_material(gltf, "blinn3-fx")?;

    graphics_mode_3d(graphics)?;
    gltf_camera(gltf, 0)?;
    let light = gltf_light(gltf, 0)?;

    let mut frame: u64 = 0;

    while glfw_ctx.poll_events() {
        let t = frame as f32 * 0.02;

        let radius = 1.5;
        let lx = t.cos() * radius;
        let ly = 1.5;
        let lz = t.sin() * radius;
        transform_set_position(light, lx, ly, lz)?;
        transform_look_at(light, 0.0, 0.8, 0.0)?;

        let r = (t * 8.0).sin() * 0.5 + 0.5;
        let g = (t * 8.0 + 2.0).sin() * 0.5 + 0.5;
        let b = (t * 8.0 + 4.0).sin() * 0.5 + 0.5;
        material_set(
            duck_mat,
            "base_color",
            MaterialValue::Float4([r, g, b, 1.0]),
        )?;

        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgb(0.1, 0.1, 0.12)),
        )?;

        graphics_record_command(graphics, DrawCommand::Material(duck_mat))?;
        graphics_record_command(graphics, DrawCommand::Geometry(duck))?;

        graphics_end_draw(graphics)?;

        frame += 1;
    }

    Ok(())
}
