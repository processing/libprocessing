//! Shows incoming MIDI: every held note is a bar (pitch -> x, velocity -> height),
//! and any control change (a knob or slider) sets the background brightness.

use bevy::color::Color;
use processing::prelude::*;
use processing_glfw::GlfwContext;
use processing_render::render::command::DrawCommand;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 400;
/// Which MIDI input to use. To pick one by name instead, call
/// `midi_connect_input_by_name("IAC")` below.
const PORT: usize = 0;

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

    midi_refresh_ports()?;
    let ports = midi_list_input_ports()?;
    if ports.is_empty() {
        eprintln!("no MIDI input ports found — connect a controller or virtual port and retry");
        return Ok(());
    }
    println!("MIDI inputs: {ports:?}");
    println!("listening to {}", midi_connect_input(PORT)?);

    // velocity of each held note, indexed by note number
    let mut held = [0u8; 128];
    let mut brightness = 0.05;

    while glfw_ctx.poll_events() {
        for msg in midi_read()? {
            let [_, data1, data2] = msg.msg;
            match midi_message_kind(&msg) {
                MidiMessageKind::NoteOn => held[data1 as usize] = data2,
                MidiMessageKind::NoteOff => held[data1 as usize] = 0,
                MidiMessageKind::ControlChange => brightness = data2 as f32 / 127.0 * 0.5,
                _ => println!("{msg:?}"),
            }
        }

        graphics_begin_draw(graphics)?;
        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(Color::srgb(brightness, brightness, brightness + 0.03)),
        )?;

        let bar_w = WIDTH as f32 / 128.0;
        for (note, &velocity) in held.iter().enumerate() {
            if velocity == 0 {
                continue;
            }
            let h = velocity as f32 / 127.0 * HEIGHT as f32;
            let hue = (note % 12) as f32 / 12.0 * 360.0;
            graphics_record_command(graphics, DrawCommand::Fill(Color::hsv(hue, 0.7, 0.95)))?;
            graphics_record_command(
                graphics,
                DrawCommand::Rect {
                    x: note as f32 * bar_w,
                    y: HEIGHT as f32 - h,
                    w: bar_w,
                    h,
                    radii: [0.0; 4],
                },
            )?;
        }

        graphics_end_draw(graphics)?;
    }

    midi_disconnect_input()?;
    Ok(())
}
