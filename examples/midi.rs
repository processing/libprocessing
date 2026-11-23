// MIDI out and in example
// MIDI OUT: click to play a note, showing up as colorful ripples
// MIDI IN: incoming MIDI notes show up as colorful bars. control change (CC) changes brightness of the bar.

use bevy::color::{Alpha, Color};
use processing::prelude::*;
use processing_glfw::GlfwContext;
use processing_render::render::command::DrawCommand;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const LOW_NOTE: u8 = 48; // C3
const NOTES: u8 = 36; // notes range 3 octaves
const PLAY_H: f32 = 250.0;

fn note_color(note: u8) -> Color {
    Color::hsv((note % 12) as f32 / 12.0 * 360.0, 0.7, 0.95)
}

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
    let outputs = midi_list_ports()?;
    println!("MIDI outputs: {outputs:?}");
    println!("MIDI inputs: {:?}", midi_list_input_ports()?);
    midi_connect(0)?;

    let out_port = outputs[0]
        .split_once(": ")
        .map_or(&*outputs[0], |(_, name)| name);
    let in_port = midi_connect_input(0)?;
    println!("sending to {out_port}");
    println!("listening to {in_port}");
    let out_label = format!("MIDI OUT: {out_port} (click to play)");
    let in_label = format!("MIDI IN: {in_port}");

    let key_w = WIDTH as f32 / NOTES as f32;
    let mut playing: Option<u8> = None;
    let mut ripples: Vec<(f32, f32, f32, Color)> = Vec::new(); // (x, y, age, color) per note played
    let mut held_in = [0u8; 128]; // incoming notes: velocity per note number
    let mut cc = 0.0; // last control change value, 0-1

    let draw = |command| graphics_record_command(graphics, command);
    let rect = |x, y, w, h| {
        draw(DrawCommand::Rect {
            x,
            y,
            w,
            h,
            radii: [0.0; 4],
        })
    };

    while glfw_ctx.poll_events() {
        let mx = input_mouse_x(surface)?;
        let my = input_mouse_y(surface)?;

        // MIDI OUT
        let pressed = input_mouse_is_pressed()?;
        if pressed && playing.is_none() && my < PLAY_H {
            let note = LOW_NOTE + ((mx / key_w) as u8).min(NOTES - 1);
            midi_note_on(note, 100)?;
            ripples.push((mx, my, 0.0, note_color(note)));
            playing = Some(note);
        } else if !pressed && let Some(note) = playing {
            midi_note_off(note)?;
            playing = None;
        }

        // MIDI IN
        for msg in midi_read()? {
            let [_, data1, data2] = msg.msg;
            match midi_message_kind(&msg) {
                MidiMessageKind::NoteOn => held_in[data1 as usize] = data2,
                MidiMessageKind::NoteOff => held_in[data1 as usize] = 0,
                MidiMessageKind::ControlChange => cc = data2 as f32 / 127.0,
                _ => {}
            }
        }

        graphics_begin_draw(graphics)?;
        draw(DrawCommand::NoStroke)?;
        draw(DrawCommand::BackgroundColor(Color::BLACK))?;

        // MIDI OUT dot visualizer
        for (x, y, age, color) in ripples.iter_mut() {
            *age += 0.02;
            let size = 24.0 + *age * 220.0;
            draw(DrawCommand::Fill(color.with_alpha((1.0 - *age).powi(2))))?;
            draw(DrawCommand::Ellipse {
                cx: *x,
                cy: *y,
                w: size,
                h: size,
            })?;
        }
        ripples.retain(|&(_, _, age, _)| age < 1.0);

        // MIDI IN bar visualizer
        draw(DrawCommand::Fill(Color::srgb(cc * 0.4, cc * 0.4, cc * 0.4)))?;
        rect(0.0, PLAY_H, WIDTH as f32, HEIGHT as f32 - PLAY_H)?;
        draw(DrawCommand::Fill(Color::srgb(0.3, 0.3, 0.3)))?;
        rect(0.0, PLAY_H, WIDTH as f32, 1.0)?;
        for (n, &velocity) in held_in.iter().enumerate() {
            if velocity > 0 {
                let h = velocity as f32 / 127.0 * (HEIGHT as f32 - PLAY_H - 36.0);
                draw(DrawCommand::Fill(note_color(n as u8)))?;
                rect(
                    (n as f32 - LOW_NOTE as f32) * key_w,
                    HEIGHT as f32 - h,
                    key_w - 2.0,
                    h,
                )?;
            }
        }

        draw(DrawCommand::Fill(Color::WHITE))?;
        draw(DrawCommand::TextSize(14.0))?;
        for (label, y) in [(&out_label, 22.0), (&in_label, PLAY_H + 22.0)] {
            draw(DrawCommand::Text {
                content: label.to_string(),
                x: 12.0,
                y,
                z: 0.0,
                max_w: None,
                max_h: None,
                stroke_weight: None,
            })?;
        }

        graphics_end_draw(graphics)?;
    }

    if let Some(note) = playing {
        midi_note_off(note)?;
    }
    midi_disconnect()?;
    midi_disconnect_input()?;
    Ok(())
}
