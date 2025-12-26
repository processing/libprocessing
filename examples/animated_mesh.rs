mod glfw;

use glfw::GlfwContext;
use processing::prelude::*;
use processing_render::geometry::Topology;
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

    let grid_size = 20;
    let spacing = 10.0;
    let offset = (grid_size as f32 * spacing) / 2.0;

    let mesh = geometry_create(Topology::TriangleList)?;

    for z in 0..grid_size {
        for x in 0..grid_size {
            let px = x as f32 * spacing - offset;
            let pz = z as f32 * spacing - offset;
            geometry_color(
                mesh,
                x as f32 / grid_size as f32,
                0.5,
                z as f32 / grid_size as f32,
                1.0,
            )?;
            geometry_normal(mesh, 0.0, 1.0, 0.0)?;
            geometry_vertex(mesh, px, 0.0, pz)?;
        }
    }

    for z in 0..(grid_size - 1) {
        for x in 0..(grid_size - 1) {
            let tl = z * grid_size + x;
            let tr = tl + 1;
            let bl = (z + 1) * grid_size + x;
            let br = bl + 1;

            geometry_index(mesh, tl)?;
            geometry_index(mesh, bl)?;
            geometry_index(mesh, tr)?;

            geometry_index(mesh, tr)?;
            geometry_index(mesh, bl)?;
            geometry_index(mesh, br)?;
        }
    }

    graphics_mode_3d(graphics)?;
    graphics_camera_position(graphics, 150.0, 150.0, 150.0)?;
    graphics_camera_look_at(graphics, 0.0, 0.0, 0.0)?;

    let mut time = 0.0f32;

    while glfw_ctx.poll_events() {
        for z in 0..grid_size {
            for x in 0..grid_size {
                let idx = (z * grid_size + x) as u32;
                let px = x as f32 * spacing - offset;
                let pz = z as f32 * spacing - offset;
                let wave = (px * 0.1 + time).sin() * (pz * 0.1 + time).cos() * 20.0;
                geometry_set_vertex(mesh, idx, px, wave, pz)?;
            }
        }

        graphics_begin_draw(graphics)?;
        graphics_record_command(graphics, DrawCommand::Geometry(mesh))?;
        graphics_end_draw(graphics)?;

        time += 0.05;
    }

    geometry_destroy(mesh)?;

    Ok(())
}
