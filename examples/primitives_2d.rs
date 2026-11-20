use processing::prelude::*;

struct Primitives2d {
    t: f32,
}

impl Sketch for Primitives2d {
    fn settings() -> Settings {
        Settings::size(600, 600)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        Ok(Primitives2d { t: 0.0 })
    }

    fn draw(&mut self, g: &mut Graphics) {
        let t = self.t;
        let cx = 300.0;
        let cy = 300.0;
        g.background((0.95, 0.93, 0.9));

        g.no_fill();
        g.stroke_weight(2);
        for i in 0..12 {
            let r = 40.0 + i as f32 * 18.0;
            let start = t + i as f32 * 0.15;
            let v = 0.2 + (i as f32 / 12.0) * 0.4;
            g.stroke((v, v * 0.8, v * 0.6));
            g.arc(cx, cy, r * 2.0, r * 2.0, start, start + PI * 1.2);
        }

        g.no_stroke();
        for i in 0..12 {
            let r = 40.0 + i as f32 * 18.0;
            let angle = t + i as f32 * 0.15;
            g.fill((0.85, 0.3 + i as f32 / 24.0, 0.2));
            g.circle(cx + r * angle.cos(), cy + r * angle.sin(), 6);
        }

        g.fill((0.1, 0.1, 0.1, 0.15));
        for i in 0..6 {
            let angle = t * 0.5 + i as f32 * PI / 3.0;
            let x = cx + 250.0 * angle.cos();
            let y = cy + 250.0 * angle.sin();
            let s = 20.0;
            g.triangle(
                x,
                y - s,
                x - s * 0.866,
                y + s * 0.5,
                x + s * 0.866,
                y + s * 0.5,
            );
        }

        let d = 15.0 + 5.0 * (t * 3.0).sin();
        g.fill((0.9, 0.4, 0.1, 0.4));
        g.quad(cx, cy - d, cx + d, cy, cx, cy + d, cx - d, cy);

        self.t += 0.01;
    }
}

fn main() -> Result<()> {
    processing::run::<Primitives2d>()
}
