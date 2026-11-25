// OSC out and in example
// OSC OUT: moving the mouse sends /mouse x y (0-1). Pressing and releasing the mouse sends /click 1 and 0.
// OSC IN: latest message is shown at the bottom.
//
// To try it with another program:
//   - receive on port 12000
//   - send to localhost (127.0.0.1) on port 10000

use bevy::color::Color;
use processing::prelude::*;
use processing_glfw::GlfwContext;
use processing_render::render::command::DrawCommand;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

const LISTEN_PORT: u16 = 10000;
const SEND_HOST: &str = "127.0.0.1"; // this computer
const SEND_PORT: u16 = 12000;

fn main() {
    if let Err(e) = sketch() {
        eprintln!("Sketch error: {e:?}");
        exit(1).unwrap();
    }
    exit(0).unwrap();
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(WIDTH, HEIGHT, false)?;
    init(Config::default())?;
    let surface = glfw_ctx.create_surface(WIDTH, HEIGHT, false)?;
    let graphics = graphics_create(surface, WIDTH, HEIGHT, TextureFormat::Rgba16Float)?;

    let port = osc_listen(LISTEN_PORT)?;
    println!("listening on port {port}, sending to {SEND_HOST}:{SEND_PORT}");
    let out_label = format!("OSC OUT: {SEND_HOST}:{SEND_PORT}");
    let in_label = format!("OSC IN: port {LISTEN_PORT}");

    let mut latest_in = format!("nothing received on port {LISTEN_PORT}");
    let mut latest_out = "nothing sent".to_string();
    let mut was_pressed = false;

    let draw = |command| graphics_record_command(graphics, command);
    let text = |content: &str, y| {
        draw(DrawCommand::Text {
            content: content.to_string(),
            x: 12.0,
            y,
            z: 0.0,
            max_w: None,
            max_h: None,
        })
    };

    while glfw_ctx.poll_events() {
        let mx = input_mouse_x(surface)?;
        let my = input_mouse_y(surface)?;
        let pressed = input_mouse_is_pressed()?;

        // OSC IN
        for received in osc_read()? {
            latest_in = format!(
                "{}  from {}",
                osc_message_text(&received.message),
                received.from
            );
        }

        // OSC OUT
        if mx != input_pmouse_x(surface)? || my != input_pmouse_y(surface)? {
            let args = vec![
                OscType::Float(mx / WIDTH as f32),
                OscType::Float(my / HEIGHT as f32),
            ];
            let to = osc_send(SEND_HOST, SEND_PORT, "/mouse", args.clone())?;
            latest_out = format!(
                "{}  to {to}",
                osc_message_text(&OscMessage {
                    addr: "/mouse".into(),
                    args
                })
            );
        }
        if pressed != was_pressed {
            let args = vec![OscType::Int(pressed as i32)];
            let to = osc_send(SEND_HOST, SEND_PORT, "/click", args.clone())?;
            latest_out = format!(
                "{}  to {to}",
                osc_message_text(&OscMessage {
                    addr: "/click".into(),
                    args
                })
            );
            was_pressed = pressed;
        }

        graphics_begin_draw(graphics)?;
        draw(DrawCommand::BackgroundColor(Color::BLACK))?;

        // OSC OUT visualizer
        draw(DrawCommand::StrokeColor(Color::srgb(0.5, 0.5, 0.5)))?;
        if pressed {
            draw(DrawCommand::Fill(Color::srgb(1.0, 1.0, 0.0)))?;
        } else {
            draw(DrawCommand::NoFill)?;
        }
        draw(DrawCommand::Ellipse {
            cx: mx,
            cy: my,
            w: 16.0,
            h: 16.0,
        })?;
        draw(DrawCommand::NoStroke)?;

        // OSC IN/OUT DISPLAY
        draw(DrawCommand::Fill(Color::WHITE))?;
        draw(DrawCommand::TextSize(14.0))?;
        text(&out_label, 22.0)?;
        text(&in_label, 42.0)?;
        text(&format!("OSC OUT: {latest_out}"), HEIGHT as f32 - 22.0)?;
        text(&format!("OSC IN: {latest_in}"), HEIGHT as f32 - 42.0)?;

        graphics_end_draw(graphics)?;
    }

    osc_stop()?;
    Ok(())
}
