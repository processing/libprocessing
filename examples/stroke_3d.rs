use processing::{Graphics, Result, Settings, Sketch};

struct Stroke3d {
    angle: f32,
}

impl Sketch for Stroke3d {
    fn settings() -> Settings {
        Settings::size(600, 400)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((200, -150, 350));
        g.camera_look_at((0, 0, 0));
        Ok(Stroke3d { angle: 0.0 })
    }

    fn draw(&mut self, g: &mut Graphics) {
        let angle = self.angle;
        g.background((0.08, 0.08, 0.12));
        g.directional_light((0.9, 0.85, 0.8), 800);

        // thin wireframe box
        {
            let mut g = g.push();
            g.translate((-80, 0));
            g.rotate(-angle);
            g.fill((0.3, 0.4, 0.7));
            g.stroke(255);
            g.stroke_weight(1);
            g.cube(60);
        }

        // thick wireframe box
        {
            let mut g = g.push();
            g.rotate(-angle * 0.7);
            g.fill((0.7, 0.3, 0.4));
            g.stroke((1.0, 0.9, 0.2));
            g.stroke_weight(3);
            g.cube((50, 70, 50));
        }

        // thick wireframe sphere
        {
            let mut g = g.push();
            g.translate((80, 0));
            g.rotate(-angle * 0.5);
            g.fill((0.3, 0.7, 0.4));
            g.stroke((0.9, 0.4, 1.0));
            g.stroke_weight(2);
            g.sphere(35).detail(16, 12);
        }

        // wireframe-only sphere
        {
            let mut g = g.push();
            g.translate((160, 0));
            g.rotate(angle * 0.3);
            g.no_fill();
            g.stroke((0.2, 0.8, 1.0));
            g.stroke_weight(1.5);
            g.sphere(30).detail(24, 16);
        }

        self.angle += 0.015;
    }
}

fn main() -> Result<()> {
    processing::run::<Stroke3d>()
}
