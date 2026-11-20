use processing::prelude::*;

struct Stroke2d;

impl Sketch for Stroke2d {
    fn settings() -> Settings {
        Settings::size(600, 300)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        Ok(Stroke2d)
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background((0.15, 0.15, 0.2));
        g.stroke_weight(12);
        g.stroke((1.0, 0.9, 0.3));

        let joins = [StrokeJoin::Round, StrokeJoin::Miter, StrokeJoin::Bevel];
        for (i, join) in joins.into_iter().enumerate() {
            let hue = i as f32 * 0.3;
            g.stroke_join(join);
            g.fill((0.3 + hue, 0.5, 0.8 - hue));
            g.rect(30.0 + i as f32 * 190.0, 50, 150, 180);
        }
    }
}

fn main() -> Result<()> {
    processing::run::<Stroke2d>()
}
