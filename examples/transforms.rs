use processing::prelude::*;

struct Transforms {
    t: f32,
}

impl Sketch for Transforms {
    fn settings() -> Settings {
        Settings::size(400, 400)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        Ok(Transforms { t: 0.0 })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background(25);

        for i in 0..4 {
            for j in 0..4 {
                let g = g.push();
                g.translate((50 + j * 100, 50 + i * 100));
                g.rotate(self.t + (i + j) as f32 * PI / 8.0);
                g.scale(0.8 + (self.t * 2.0 + (i * j) as f32).sin() * 0.2);
                g.fill((j as f32 / 3.0, i as f32 / 3.0, 0.8));
                g.rect(-20, -20, 40, 40);
            }
        }

        self.t += 0.02;
    }
}

fn main() -> Result<()> {
    processing::run::<Transforms>()
}
