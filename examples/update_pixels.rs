mod glfw;

use bevy::{color::Color, prelude::LinearRgba};
use glfw::GlfwContext;
use processing::prelude::*;

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
    let mut glfw_ctx = GlfwContext::new(100, 100)?;
    init(Config::default())?;

    let width = 100;
    let height = 100;
    let scale_factor = 1.0;

    let surface = glfw_ctx.create_surface(width, height, scale_factor)?;
    let graphics = graphics_create(surface, width, height, TextureFormat::Rgba16Float)?;

    let rect_w = 10;
    let rect_h = 10;
    let red = LinearRgba::new(1.0, 0.0, 0.0, 1.0);
    let red_pixels: Vec<LinearRgba> = vec![red; rect_w * rect_h];

    let blue = LinearRgba::new(0.0, 0.0, 1.0, 1.0);
    let blue_pixels: Vec<LinearRgba> = vec![blue; rect_w * rect_h];

    let mut first_frame = true;

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        graphics_record_command(graphics, DrawCommand::BackgroundColor(Color::BLACK))?;
        graphics_flush(graphics)?;
        graphics_update_region(graphics, 20, 20, rect_w as u32, rect_h as u32, &red_pixels)?;
        graphics_update_region(graphics, 60, 60, rect_w as u32, rect_h as u32, &blue_pixels)?;

        graphics_end_draw(graphics)?;

        if first_frame {
            first_frame = false;

            let pixels = graphics_readback(graphics)?;
            eprintln!("Total pixels: {}", pixels.len());

            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) as usize;
                    if idx < pixels.len() {
                        let pixel = pixels[idx];
                        if pixel.red > 0.5 {
                            eprint!("R");
                        } else if pixel.blue > 0.5 {
                            eprint!("B");
                        } else if pixel.alpha > 0.5 {
                            eprint!(".");
                        } else {
                            eprint!(" ");
                        }
                    }
                }
                eprintln!();
            }

            eprintln!("\nSample pixels:");
            eprintln!("(25, 25): {:?}", pixels[25 * width as usize + 25]);
            eprintln!("(65, 65): {:?}", pixels[65 * width as usize + 65]);
            eprintln!("(0, 0): {:?}", pixels[0]);
            eprintln!("(50, 50): {:?}", pixels[50 * width as usize + 50]);
        }
    }

    Ok(())
}
