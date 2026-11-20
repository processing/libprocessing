use processing::prelude::*;

struct ParticlesFromMesh {
    sun: Light,
    particle: Geometry,
    particles: Particles,
    material: Material,
}

impl Sketch for ParticlesFromMesh {
    fn settings() -> Settings {
        Settings::size(900, 700)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((0.0, -4.0, 18.0));
        g.camera_look_at((0, 0, 0));

        let sun = g.create_directional_light((0.95, 0.9, 0.85), 200.0);

        let source = Geometry::sphere(5.0, 32, 24)?;
        let particles = Particles::from_geometry(
            &source,
            &[Attribute::position(), Attribute::uv(), Attribute::color()],
        )?;

        let uvs = particles.buffer(Attribute::uv()).read::<Vec2>()?;
        let mut colors: Vec<Color> = Vec::with_capacity(uvs.len());
        for uv in &uvs {
            colors.push(Color::hsv(uv.x * 360.0, 0.85, 1.0));
        }
        let mut color_buf = particles.buffer(Attribute::color());
        color_buf.write(&colors);

        let particle = Geometry::sphere(0.18, 10, 8)?;
        let mut material = Material::pbr()?;
        material.albedo_buffer(&color_buf);

        Ok(ParticlesFromMesh {
            sun,
            particle,
            particles,
            material,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.light(&self.sun);
        g.background((0.06, 0.06, 0.08));
        g.material(&self.material);
        g.particles(&self.particles).geometry(&self.particle);
    }
}

fn main() -> Result<()> {
    processing::run::<ParticlesFromMesh>()
}
