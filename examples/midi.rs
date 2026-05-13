use bevy::color::{Color, ColorToComponents};
use processing::prelude::*;
use processing_glfw::GlfwContext;
use processing_render::render::command::DrawCommand;

const PENTATONIC: [u8; 5] = [0, 3, 5, 7, 10];
const ROOT: u8 = 48;
const OCTAVES: u8 = 3;
const WIDTH: u32 = 640;
const HEIGHT: u32 = 400;

fn note_at_x(x: f32) -> u8 {
    let steps = (PENTATONIC.len() as u8) * OCTAVES;
    let t = (x / WIDTH as f32).clamp(0.0, 0.999);
    let idx = (t * steps as f32) as u8;
    let octave = idx / PENTATONIC.len() as u8;
    let degree = (idx % PENTATONIC.len() as u8) as usize;
    ROOT + octave * 12 + PENTATONIC[degree]
}

fn velocity_at_y(y: f32) -> u8 {
    let t = 1.0 - (y / HEIGHT as f32).clamp(0.0, 1.0);
    (16.0 + t * 110.0) as u8
}

fn note_color(note: u8) -> Color {
    let hue = ((note - ROOT) as f32 / 12.0).fract() * 360.0;
    Color::hsv(hue, 0.7, 0.95)
}

struct Ripple {
    x: f32,
    y: f32,
    age: f32,
    color: Color,
}

fn main() {
    if let Err(e) = sketch() {
        eprintln!("Sketch error: {e:?}");
        exit(1).unwrap();
    }
    exit(0).unwrap();
}

fn sketch() -> error::Result<()> {
    let mut glfw_ctx = GlfwContext::new(WIDTH, HEIGHT)?;
    init(Config::default())?;

    let surface = glfw_ctx.create_surface(WIDTH, HEIGHT)?;
    let graphics = graphics_create(surface, WIDTH, HEIGHT, TextureFormat::Rgba16Float)?;

    midi_refresh_ports()?;
    let ports = midi_list_ports()?;
    if ports.is_empty() {
        eprintln!("no MIDI output ports found — connect a synth or virtual port and retry");
        return Ok(());
    }
    for p in &ports {
        println!("{p}");
    }
    midi_connect(0)?;

    let mut held: Option<u8> = None;
    let mut ripples: Vec<Ripple> = Vec::new();

    while glfw_ctx.poll_events() {
        graphics_begin_draw(graphics)?;

        let mx = input_mouse_x(surface)?;
        let my = input_mouse_y(surface)?;
        let pressed = input_mouse_is_pressed()?;

        match (pressed, held) {
            (true, None) => {
                let note = note_at_x(mx);
                let velocity = velocity_at_y(my);
                midi_note_on(note, velocity)?;
                ripples.push(Ripple {
                    x: mx,
                    y: my,
                    age: 0.0,
                    color: note_color(note),
                });
                held = Some(note);
            }
            (true, Some(note)) => {
                let target = note_at_x(mx);
                if target != note {
                    midi_note_off(note)?;
                    let velocity = velocity_at_y(my);
                    midi_note_on(target, velocity)?;
                    ripples.push(Ripple {
                        x: mx,
                        y: my,
                        age: 0.0,
                        color: note_color(target),
                    });
                    held = Some(target);
                }
            }
            (false, Some(note)) => {
                midi_note_off(note)?;
                held = None;
            }
            _ => {}
        }

        graphics_record_command(
            graphics,
            DrawCommand::BackgroundColor(Color::srgb(0.04, 0.04, 0.07)),
        )?;

        let octave_w = WIDTH as f32 / OCTAVES as f32;
        for i in 0..OCTAVES {
            let alpha = if i % 2 == 0 { 0.04 } else { 0.0 };
            graphics_record_command(
                graphics,
                DrawCommand::Fill(Color::srgba(1.0, 1.0, 1.0, alpha)),
            )?;
            graphics_record_command(
                graphics,
                DrawCommand::Rect {
                    x: i as f32 * octave_w,
                    y: 0.0,
                    w: octave_w,
                    h: HEIGHT as f32,
                    radii: [0.0; 4],
                },
            )?;
        }

        ripples.retain_mut(|r| {
            r.age += 0.022;
            r.age < 1.0
        });
        for r in &ripples {
            let size = 24.0 + r.age * 220.0;
            let alpha = (1.0 - r.age).powi(2);
            let lin = r.color.to_linear().to_f32_array();
            graphics_record_command(
                graphics,
                DrawCommand::Fill(Color::linear_rgba(lin[0], lin[1], lin[2], alpha)),
            )?;
            graphics_record_command(
                graphics,
                DrawCommand::Rect {
                    x: r.x - size / 2.0,
                    y: r.y - size / 2.0,
                    w: size,
                    h: size,
                    radii: [size / 2.0; 4],
                },
            )?;
        }

        let preview = note_color(note_at_x(mx));
        graphics_record_command(graphics, DrawCommand::Fill(preview))?;
        graphics_record_command(
            graphics,
            DrawCommand::Rect {
                x: mx - 8.0,
                y: my - 8.0,
                w: 16.0,
                h: 16.0,
                radii: [8.0; 4],
            },
        )?;

        graphics_end_draw(graphics)?;

        if input_key_is_pressed()? && input_key_is_down(KeyCode::Escape)? {
            break;
        }
    }

    if let Some(note) = held {
        midi_note_off(note)?;
    }
    midi_disconnect()?;
    Ok(())
}
