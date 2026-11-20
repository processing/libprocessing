use processing::prelude::*;

struct BoxSketch {
    cube: Geometry,
    angle: f32,
}

impl Sketch for BoxSketch {
    fn settings() -> Settings {
        Settings::size(400, 400)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((100, -100, 300));
        g.camera_look_at((0, 0, 0));

        let cube = Geometry::cube(100)?;
        Ok(BoxSketch { cube, angle: 0.0 })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background((0.1, 0.1, 0.15));

        {
            let g = g.push();
            g.rotate(-self.angle);
            g.model(&self.cube);
        }

        self.angle += 0.02;
    }
}

fn main() -> Result<()> {
    processing::run::<BoxSketch>()
}
