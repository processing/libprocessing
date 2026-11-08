use processing::{Color, Geometry, Graphics, Light, Material, Result, Settings, Sketch};

const COLS: usize = 11;
const ROWS: usize = 5;
const SPACING: f32 = 70.0;

struct Materials {
    sphere: Geometry,
    materials: Vec<Material>,
    sun: Light,
    lamp: Light,
}

impl Sketch for Materials {
    fn settings() -> Settings {
        Settings::size(800, 600)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((0, 0, 600));
        g.camera_look_at((0, 0, 0));

        let sun = g.create_directional_light((1.0, 0.98, 0.95), 1500);
        sun.position((300, -400, 300)).look_at((0, 0, 0));
        let lamp = g.create_point_light(Color::WHITE, 100_000, 800, 0);
        lamp.position((200, -200, 400));

        // roughness across, metallic down
        let mut materials = Vec::new();
        for row in 0..ROWS {
            for col in 0..COLS {
                let mut material = Material::pbr()?;
                material.set("roughness", col as f32 / (COLS - 1) as f32);
                material.set("metallic", row as f32 / (ROWS - 1) as f32);
                materials.push(material);
            }
        }

        let sphere = Geometry::sphere(30, 32, 18)?;
        Ok(Materials {
            sphere,
            materials,
            sun,
            lamp,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.light(&self.sun);
        g.light(&self.lamp);
        g.background((0.05, 0.05, 0.07));
        g.fill((1.0, 0.85, 0.57));

        let offset_x = (COLS - 1) as f32 * SPACING / 2.0;
        let offset_y = (ROWS - 1) as f32 * SPACING / 2.0;
        for row in 0..ROWS {
            for col in 0..COLS {
                let mut g = g.push();
                g.translate((
                    col as f32 * SPACING - offset_x,
                    offset_y - row as f32 * SPACING,
                ));
                g.material(&self.materials[row * COLS + col]);
                g.model(&self.sphere);
            }
        }
    }
}

fn main() -> Result<()> {
    processing::run::<Materials>()
}
