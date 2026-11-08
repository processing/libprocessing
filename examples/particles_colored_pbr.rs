use processing::{
    Attribute, Geometry, Graphics, Light, Material, Particles, Result, Settings, Sketch,
};

struct ParticlesColoredPbr {
    sun: Light,
    sphere: Geometry,
    particles: Particles,
    material: Material,
}

impl Sketch for ParticlesColoredPbr {
    fn settings() -> Settings {
        Settings::size(900, 700)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((0.0, -6.0, 18.0));
        g.camera_look_at((0, 0, 0));

        let sun = g.create_directional_light((0.95, 0.9, 0.85), 200.0);

        let sphere = Geometry::sphere(0.3, 16, 12)?;

        let capacity: u32 = 512;
        let mut positions: Vec<f32> = Vec::with_capacity(capacity as usize * 3);
        let mut colors: Vec<f32> = Vec::with_capacity(capacity as usize * 4);
        for x in 0..8 {
            for y in 0..8 {
                for z in 0..8 {
                    positions.push((x as f32 - 3.5) * 1.4);
                    positions.push((3.5 - y as f32) * 1.4);
                    positions.push((z as f32 - 3.5) * 1.4);
                    colors.push(x as f32 / 7.0);
                    colors.push(y as f32 / 7.0);
                    colors.push(z as f32 / 7.0);
                    colors.push(1.0);
                }
            }
        }

        let particles = Particles::new(capacity, &[Attribute::position(), Attribute::color()])?;
        particles.buffer(Attribute::position()).write(&positions);
        let mut color_buf = particles.buffer(Attribute::color());
        color_buf.write(&colors);

        let mut material = Material::pbr()?;
        material.albedo_buffer(&color_buf);

        Ok(ParticlesColoredPbr {
            sun,
            sphere,
            particles,
            material,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.light(&self.sun);
        g.background((0.06, 0.06, 0.08));
        g.material(&self.material);
        g.particles(&self.particles).geometry(&self.sphere);
    }
}

fn main() -> Result<()> {
    processing::run::<ParticlesColoredPbr>()
}
