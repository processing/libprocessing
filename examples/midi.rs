use processing::prelude::*;

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

struct Midi {
    held: Option<u8>,
    ripples: Vec<Ripple>,
}

impl Midi {
    fn play(&mut self, note: u8, x: f32, y: f32) -> Result<()> {
        midi::note_on(note, velocity_at_y(y))?;
        self.ripples.push(Ripple {
            x,
            y,
            age: 0.0,
            color: note_color(note),
        });
        self.held = Some(note);
        Ok(())
    }

    fn update_notes(&mut self, g: &Graphics) -> Result<()> {
        let x = g.mouse_x();
        let y = g.mouse_y();
        match (g.mouse_pressed(), self.held) {
            (true, None) => self.play(note_at_x(x), x, y)?,
            (true, Some(note)) => {
                let target = note_at_x(x);
                if target != note {
                    midi::note_off(note)?;
                    self.play(target, x, y)?;
                }
            }
            (false, Some(note)) => {
                midi::note_off(note)?;
                self.held = None;
            }
            (false, None) => {}
        }
        Ok(())
    }
}

impl Sketch for Midi {
    fn settings() -> Settings {
        Settings::size(WIDTH, HEIGHT)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        midi::refresh_ports()?;
        let ports = midi::ports()?;
        if ports.is_empty() {
            eprintln!("no MIDI output ports found — connect a synth or virtual port and retry");
            g.exit();
        } else {
            for port in &ports {
                println!("{port}");
            }
            midi::connect(0)?;
        }
        Ok(Midi {
            held: None,
            ripples: Vec::new(),
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        if let Err(e) = self.update_notes(g) {
            eprintln!("midi: {e}");
        }

        g.background((0.04, 0.04, 0.07));
        g.no_stroke();

        let octave_w = WIDTH as f32 / OCTAVES as f32;
        for i in 0..OCTAVES {
            if i % 2 == 0 {
                g.fill((1.0, 1.0, 1.0, 0.04));
                g.rect(i as f32 * octave_w, 0, octave_w, HEIGHT);
            }
        }

        for ripple in &mut self.ripples {
            ripple.age += 0.022;
        }
        self.ripples.retain(|r| r.age < 1.0);

        for ripple in &self.ripples {
            let size = 24.0 + ripple.age * 220.0;
            let alpha = (1.0 - ripple.age).powi(2);
            g.fill(ripple.color.with_alpha(alpha));
            g.circle(ripple.x, ripple.y, size);
        }

        g.fill(note_color(note_at_x(g.mouse_x())));
        g.circle(g.mouse_x(), g.mouse_y(), 16);

        if g.key_pressed() && g.key_down(Key::Escape) {
            g.exit();
        }
    }
}

impl Drop for Midi {
    fn drop(&mut self) {
        if let Some(note) = self.held {
            let _ = midi::note_off(note);
        }
        let _ = midi::disconnect();
    }
}

fn main() -> Result<()> {
    processing::run::<Midi>()
}
