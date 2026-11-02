use processing::{Graphics, Result, Settings, Sketch};

struct Curves {
    t: f32,
}

impl Sketch for Curves {
    fn settings() -> Settings {
        Settings::size(800, 400)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        Ok(Curves { t: 0.0 })
    }

    fn draw(&mut self, g: &mut Graphics) {
        let t = self.t;
        g.background((0.04, 0.04, 0.06));
        g.no_fill();

        // bezier waves
        g.stroke_weight(1.5);
        for i in 0..20 {
            let y = 20.0 + i as f32 * 15.0;
            let phase = t + i as f32 * 0.3;
            let amp = 30.0 + (i as f32 * 0.5).sin() * 20.0;
            let v = 0.3 + (i as f32 / 20.0) * 0.5;
            g.stroke((v * 0.6, v, (v * 1.2).min(1.0)));
            g.bezier(
                0,
                y,
                250,
                y + amp * phase.sin(),
                550,
                y - amp * (phase * 1.3).cos(),
                800,
                y,
            );
        }

        // catmull-rom curves
        g.stroke_weight(0.8);
        g.stroke((1.0, 0.6, 0.2, 0.4));
        for i in 0..8 {
            let y = 50.0 + i as f32 * 40.0;
            let phase = t * 0.7 + i as f32 * 0.5;
            g.curve(
                -50,
                y + 40.0 * phase.sin(),
                200,
                y + 30.0 * (phase * 1.2).cos(),
                600,
                y - 30.0 * (phase * 0.8).sin(),
                850,
                y + 20.0 * phase.cos(),
            );
        }

        // guide lines
        g.stroke_weight(0.3);
        g.stroke((1.0, 1.0, 1.0, 0.08));
        for i in 0..20 {
            let y = 20 * i;
            g.line(0, y, 800, y);
        }

        self.t += 0.012;
    }
}

fn main() -> Result<()> {
    processing::run::<Curves>()
}
