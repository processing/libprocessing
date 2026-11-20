use processing::prelude::*;

struct Shapes {
    t: f32,
}

impl Sketch for Shapes {
    fn settings() -> Settings {
        Settings::size(600, 600)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        Ok(Shapes { t: 0.0 })
    }

    fn draw(&mut self, g: &mut Graphics) {
        let t = self.t;
        g.background((235, 230, 222));

        g.fill((0.2, 0.3, 0.5, 0.25));
        g.stroke_weight(1.5);
        g.stroke((38, 51, 89));
        let n = 8;
        let blob = g.begin_shape(ShapeKind::Polygon);
        for i in 0..n + 3 {
            let idx = (i + n - 1) % n;
            let angle = idx as f32 * 2.0 * PI / n as f32;
            let r = 100.0 + 30.0 * (t * 2.0 + idx as f32 * 0.8).sin();
            blob.curve_vertex(170.0 + r * angle.cos(), 200.0 + r * angle.sin());
        }
        blob.close();

        g.fill((0.15, 0.5, 0.3, 0.2));
        g.stroke_weight(1);
        g.stroke((26, 89, 51));
        let cx = 430.0;
        let cy = 200.0;
        let s = 70.0 + 10.0 * (t * 1.5).sin();
        let lens = g.begin_shape(ShapeKind::Polygon);
        lens.vertex(cx, cy - s);
        lens.bezier_vertex(cx + s, cy - s, cx + s, cy + s, cx, cy + s);
        lens.bezier_vertex(cx - s, cy + s, cx - s, cy - s, cx, cy - s);
        lens.close();

        g.fill((0.8, 0.35, 0.1, 0.15));
        g.no_stroke();
        let fan = g.begin_shape(ShapeKind::TriangleFan);
        fan.vertex(170, 450);
        for i in 0..=16 {
            let angle = i as f32 * 2.0 * PI / 16.0;
            let r = 60.0 + 20.0 * (t * 3.0 + angle * 2.0).sin();
            fan.vertex(170.0 + r * angle.cos(), 450.0 + r * angle.sin());
        }
        fan.close();

        g.fill((0.5, 0.2, 0.6, 0.2));
        g.stroke_weight(0.5);
        g.stroke((77, 26, 102, 102));
        let strip = g.begin_shape(ShapeKind::TriangleStrip);
        for i in 0..16 {
            let x = 320.0 + i as f32 * 17.0;
            let wave = 30.0 * (t * 2.0 + i as f32 * 0.4).sin();
            strip.vertex(x, 420.0 + wave);
            strip.vertex(x, 480.0 + wave);
        }
        strip.end();

        self.t += 0.02;
    }
}

fn main() -> Result<()> {
    processing::run::<Shapes>()
}
