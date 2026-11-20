use processing::prelude::*;

struct ParticlesColored {
    sphere: Geometry,
    particles: Particles,
    material: Material,
}

impl Sketch for ParticlesColored {
    fn settings() -> Settings {
        Settings::size(900, 700)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((0.0, -6.0, 18.0));
        g.camera_look_at((0, 0, 0));

        let sphere = Geometry::sphere(0.25, 12, 8)?;

        let capacity: u32 = 1000;
        let mut positions: Vec<Vec3> = Vec::with_capacity(capacity as usize);
        let mut colors: Vec<Color> = Vec::with_capacity(capacity as usize);
        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    positions.push(Vec3::new(x as f32 - 4.5, 4.5 - y as f32, z as f32 - 4.5));
                    colors.push(Color::rgb(x as f32 / 9.0, y as f32 / 9.0, z as f32 / 9.0));
                }
            }
        }

        let particles = Particles::new(capacity, &[Attribute::position(), Attribute::color()])?;
        particles.buffer(Attribute::position()).write(&positions);
        let mut color_buf = particles.buffer(Attribute::color());
        color_buf.write(&colors);

        let mut material = Material::unlit()?;
        material.albedo_buffer(&color_buf);

        Ok(ParticlesColored {
            sphere,
            particles,
            material,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background((0.06, 0.06, 0.08));
        g.material(&self.material);
        g.particles(&self.particles).geometry(&self.sphere);
    }
}

fn main() -> Result<()> {
    processing::run::<ParticlesColored>()
}
