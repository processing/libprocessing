use processing::prelude::*;

struct CameraControllers {
    cube: Geometry,
    sun: Light,
    angle: f32,
    mode: u8,
}

impl Sketch for CameraControllers {
    fn settings() -> Settings {
        Settings::size(800, 600)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.orbit_camera();

        let sun = g.create_directional_light((1.0, 0.98, 0.95), 1500);
        sun.position((300, -400, 300)).look_at((0, 0, 0));

        let cube = Geometry::cube(100)?;
        Ok(CameraControllers {
            cube,
            sun,
            angle: 0.0,
            mode: 0,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        if g.key_just_pressed(Key::Num1) {
            g.mode_3d();
            g.orbit_camera();
            self.mode = 0;
        }
        if g.key_just_pressed(Key::Num2) {
            g.mode_3d();
            g.free_camera();
            self.mode = 1;
        }
        if g.key_just_pressed(Key::Num3) {
            g.mode_2d();
            g.pan_camera();
            self.mode = 2;
        }

        g.light(&self.sun);
        g.background((0.05, 0.05, 0.07));

        if self.mode < 2 {
            g.fill((1.0, 0.85, 0.57));
            g.roughness(0.3);
            g.metallic(0.8);
            let g = g.push();
            g.rotate(-self.angle);
            g.model(&self.cube);
        } else {
            g.fill((0.8, 0.3, 0.2));
            g.rect(300, 200, 200, 200);
        }

        self.angle += 0.02;
    }
}

fn main() -> Result<()> {
    processing::run::<CameraControllers>()
}
