use processing::prelude::*;

struct Lights {
    cube: Geometry,
    shiny: Material,
    sun: Light,
    warm: Light,
    cool: Light,
    spot: Light,
    angle: f32,
}

impl Sketch for Lights {
    fn settings() -> Settings {
        Settings::size(400, 400)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((100, -100, 300));
        g.camera_look_at((0, 0, 0));

        let cube = Geometry::cube(100)?;
        let mut shiny = Material::pbr()?;
        shiny.set("roughness", 0.0);

        let sun = g.create_directional_light((0.35, 0.25, 0.5), 500);

        let warm = g.create_point_light((1.0, 0.5, 0.25), 1_000_000, 200, 0.5);
        warm.position((-25, -5, 51)).look_at((0, 0, 0));

        let cool = g.create_point_light((0.0, 0.5, 0.75), 2_000_000, 200, 0.25);
        cool.position((0.0, -5.0, 50.5)).look_at((0, 0, 0));

        let spot = g.create_spot_light((0.25, 0.8, 0.19), 15_000_000, 200, 0.84, 0, QUARTER_PI);
        spot.position((40, 0, 70)).look_at((0, 0, 0));

        Ok(Lights {
            cube,
            shiny,
            sun,
            warm,
            cool,
            spot,
            angle: 0.0,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.light(&self.sun);
        g.light(&self.warm);
        g.light(&self.cool);
        g.light(&self.spot);

        g.background((0.18, 0.20, 0.15));
        g.fill(255);
        g.material(&self.shiny);

        {
            let g = g.push();
            g.rotate(-self.angle);
            g.model(&self.cube);
        }

        self.angle += 0.02;
    }
}

fn main() -> Result<()> {
    processing::run::<Lights>()
}
