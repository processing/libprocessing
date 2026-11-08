use processing::{Geometry, Graphics, Material, Result, Settings, Shader, Sketch};

struct CustomMaterial {
    cube: Geometry,
    material: Material,
    angle: f32,
}

impl Sketch for CustomMaterial {
    fn settings() -> Settings {
        Settings::size(400, 400)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((100, -100, 300));
        g.camera_look_at((0, 0, 0));

        let shader = Shader::load("shaders/custom_material.wesl")?;
        let mut material = Material::new(&shader)?;
        material.set("color", (1.0, 0.2, 0.4, 1.0));

        let cube = Geometry::cube(100)?;
        Ok(CustomMaterial {
            cube,
            material,
            angle: 0.0,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background((0.1, 0.1, 0.15));

        {
            let mut g = g.push();
            g.rotate(-self.angle);
            g.material(&self.material);
            g.model(&self.cube);
        }

        self.angle += 0.02;
    }
}

fn main() -> Result<()> {
    processing::run::<CustomMaterial>()
}
