use processing_glfw::GlfwContext;

use processing::prelude::*;

const MODES: &[BlendMode] = &[
    BlendMode::Blend,
    BlendMode::Add,
    BlendMode::Subtract,
    BlendMode::Darkest,
    BlendMode::Lightest,
    BlendMode::Difference,
    BlendMode::Exclusion,
    BlendMode::Multiply,
    BlendMode::Screen,
    BlendMode::Replace,
];

fn main() {
    match sketch() {
        Ok(_) => exit(0).unwrap(),
        Err(e) => {
            eprintln!("Error: {e:?}");
            exit(1).unwrap();
        }
    }
}

fn sketch() -> error::Result<()> {
    let width = 500;
    let height = 500;
    let mut glfw_ctx = GlfwContext::new(width, height)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(width, height)?;
    let graphics = graphics_create(surface, width, height, TextureFormat::Rgba16Float)?;

    let mut index: usize = 0;

    while glfw_ctx.poll_events() {
        if input_key_just_pressed(KeyCode::ArrowRight)? || input_key_just_pressed(KeyCode::Space)? {
            index = (index + 1) % MODES.len();
            eprintln!("{}", MODES[index].name());
        } else if input_key_just_pressed(KeyCode::ArrowLeft)? {
            index = (index + MODES.len() - 1) % MODES.len();
            eprintln!("{}", MODES[index].name());
        }

        graphics_begin_draw(graphics)?;

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(bevy::color::Color::srgba(0.15, 0.15, 0.15, 1.0)),
        )?;
        graphics_record_command(graphics, DrawCommand::NoStroke)?;
        graphics_record_command(
            graphics,
            DrawCommand::BlendMode(MODES[index].to_blend_state()),
        )?;

        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgba(0.9, 0.2, 0.2, 0.75)),
        )?;
        graphics_record_command(
            graphics,
            DrawCommand::Rect {
                x: 80.0,
                y: 100.0,
                w: 200.0,
                h: 250.0,
                radii: [0.0; 4],
            },
        )?;

        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgba(0.2, 0.8, 0.2, 0.75)),
        )?;
        graphics_record_command(
            graphics,
            DrawCommand::Rect {
                x: 180.0,
                y: 80.0,
                w: 200.0,
                h: 250.0,
                radii: [0.0; 4],
            },
        )?;

        graphics_record_command(
            graphics,
            DrawCommand::Fill(bevy::color::Color::srgba(0.2, 0.3, 0.9, 0.75)),
        )?;
        graphics_record_command(
            graphics,
            DrawCommand::Rect {
                x: 130.0,
                y: 200.0,
                w: 200.0,
                h: 200.0,
                radii: [0.0; 4],
            },
        )?;

        graphics_end_draw(graphics)?;
    }

    Ok(())
}
