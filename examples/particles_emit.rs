use processing::{Attribute, Geometry, Graphics, Material, Particles, Result, Settings, Sketch};

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
        let far_away: Vec<f32> = vec![1.0e6; capacity as usize * 3];
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
        let mut positions: Vec<f32> = Vec::with_capacity(burst as usize * 3);
        let mut colors: Vec<f32> = Vec::with_capacity(burst as usize * 4);
        for k in 0..burst {
            let i = self.frame * burst + k;
            let t = i as f32 * 0.05;
            let radius = 1.5 + (t * 0.02).min(3.0);
            let height = -(t * 0.1).sin() * 2.0;
            positions.push(t.cos() * radius);
            positions.push(height);
            positions.push(t.sin() * radius);
            let h = (i as f32 * 0.012) % 1.0;
            let (r, g, b) = hsv_to_rgb(h, 0.85, 1.0);
            colors.push(r);
            colors.push(g);
            colors.push(b);
            colors.push(1.0);
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

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match (i as i32) % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

fn main() -> Result<()> {
    processing::run::<ParticlesEmit>()
}
