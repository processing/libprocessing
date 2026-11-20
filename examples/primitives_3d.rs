use processing::prelude::*;

struct Primitives3d {
    t: f32,
}

impl Sketch for Primitives3d {
    fn settings() -> Settings {
        Settings::size(900, 400)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((0, -80, 800));
        g.camera_look_at((0, 0, 0));
        Ok(Primitives3d { t: 0.0 })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background((0.06, 0.06, 0.08));
        g.directional_light((0.9, 0.85, 0.8), 300);
        g.fill(255);
        g.roughness(0.35);

        for i in 0..8 {
            let g = g.push();
            g.translate((-315 + i * 90, 0));
            g.rotate(-self.t);
            match i {
                0 => {
                    g.cube(50);
                }
                1 => {
                    g.sphere(30).detail(24, 16);
                }
                2 => {
                    g.cylinder(25, 60).detail(24);
                }
                3 => {
                    g.cone(25, 60).detail(24);
                }
                4 => {
                    g.torus(30, 10).detail(24, 16);
                }
                5 => {
                    g.capsule(15, 40).detail(24);
                }
                6 => {
                    g.conical_frustum(15, 25, 60).detail(24);
                }
                _ => {
                    g.tetrahedron(30);
                }
            }
        }

        self.t += 0.015;
    }
}

fn main() -> Result<()> {
    processing::run::<Primitives3d>()
}
