use processing::prelude::*;

struct ParticlesEmit {
    sphere: Geometry,
    particles: Particles,
    material: Material,
    frame: u32,
}

impl Sketch for ParticlesEmit {
    fn settings() -> Settings {
        Settings::size(900, 700)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((0.0, -4.0, 14.0));
        g.camera_look_at((0, 0, 0));

        let sphere = Geometry::sphere(0.08, 8, 6)?;

        let capacity: u32 = 2000;
        let particles = Particles::new(capacity, &[Attribute::position(), Attribute::color()])?;

        // offscreen until emitted
        let far_away = vec![Vec3::splat(1.0e6); capacity as usize];
        particles.buffer(Attribute::position()).write(&far_away);

        let mut material = Material::unlit()?;
        material.albedo_buffer(&particles.buffer(Attribute::color()));

        Ok(ParticlesEmit {
            sphere,
            particles,
            material,
            frame: 0,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background((0.06, 0.06, 0.08));
        g.material(&self.material);
        g.particles(&self.particles).geometry(&self.sphere);

        let burst = 4u32;
        let mut positions: Vec<Vec3> = Vec::with_capacity(burst as usize);
        let mut colors: Vec<Color> = Vec::with_capacity(burst as usize);
        for k in 0..burst {
            let i = self.frame * burst + k;
            let t = i as f32 * 0.05;
            let radius = 1.5 + (t * 0.02).min(3.0);
            let height = -(t * 0.1).sin() * 2.0;
            positions.push(Vec3::new(t.cos() * radius, height, t.sin() * radius));
            let h = (i as f32 * 0.012) % 1.0;
            colors.push(Color::hsv(h * 360.0, 0.85, 1.0));
        }

        self.particles.emit(
            burst,
            &[
                (Attribute::position(), &positions),
                (Attribute::color(), &colors),
            ],
        );
        self.frame += 1;
    }
}

fn main() -> Result<()> {
    processing::run::<ParticlesEmit>()
}
