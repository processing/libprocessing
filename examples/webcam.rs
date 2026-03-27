use processing_glfw::GlfwContext;

use processing::prelude::*;
use processing_render::render::command::DrawCommand;
use processing_webcam::{webcam_create, webcam_destroy, webcam_image, webcam_is_connected};

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
    let width = 640;
    let height = 480;

    let mut glfw_ctx = GlfwContext::new(width, height)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(width, height)?;
    let graphics = graphics_create(surface, width, height, TextureFormat::Rgba16Float)?;

    let webcam = webcam_create()?;
    let mut image_entity = None;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        // Once connected, grab an image entity from the webcam stream
        if image_entity.is_none() && webcam_is_connected(webcam)? {
            image_entity = Some(webcam_image(webcam)?);
        }

        if let Some(img) = image_entity {
            graphics_record_command(graphics, DrawCommand::BackgroundImage(img))?;
        }

        graphics_end_draw(graphics)?;
    }

    webcam_destroy(webcam)?;
    Ok(())
}
