use processing::prelude::*;

struct ParticlesBasic {
    sun: Light,
    sphere: Geometry,
    particles: Particles,
    pbr: Material,
}

impl Sketch for ParticlesBasic {
    fn settings() -> Settings {
        Settings::size(900, 700)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((0.0, 0.0, 25.0));
        g.camera_look_at((0, 0, 0));

        let sun = g.create_directional_light((0.9, 0.85, 0.8), 300.0);

        let sphere = Geometry::sphere(0.25, 12, 8)?;

        let capacity: u32 = 1000;
        let mut positions: Vec<Vec3> = Vec::with_capacity(capacity as usize);
        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    positions.push(Vec3::new(x as f32 - 4.5, y as f32 - 4.5, z as f32 - 4.5));
                }
            }
        }

        let particles = Particles::new(capacity, &[Attribute::position()])?;
        particles.buffer(Attribute::position()).write(&positions);

        let mut pbr = Material::pbr()?;
        pbr.set("roughness", 0.4);

        Ok(ParticlesBasic {
            sun,
            sphere,
            particles,
            pbr,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.light(&self.sun);
        g.background((0.06, 0.06, 0.08));
        g.fill((0.9, 0.5, 0.3));
        g.material(&self.pbr);
        g.particles(&self.particles).geometry(&self.sphere);
    }
}

fn main() -> Result<()> {
    processing::run::<ParticlesBasic>()
}
