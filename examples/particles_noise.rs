use processing::{
    Attribute, Compute, Geometry, Graphics, Light, Material, Particles, Result, Settings, Sketch,
};

struct ParticlesNoise {
    sun: Light,
    particle: Geometry,
    particles: Particles,
    material: Material,
    noise: Compute,
}

impl Sketch for ParticlesNoise {
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

        // hue from u
        let uvs = particles.buffer(Attribute::uv()).read::<f32>()?;
        let mut colors: Vec<f32> = Vec::with_capacity(uvs.len() * 2);
        for uv in uvs.chunks_exact(2) {
            let (r, g, b) = hsv_to_rgb(uv[0], 0.85, 1.0);
            colors.push(r);
            colors.push(g);
            colors.push(b);
            colors.push(1.0);
        }
        let mut color_buf = particles.buffer(Attribute::color());
        color_buf.write(&colors);

        let particle = Geometry::sphere(0.18, 10, 8)?;
        let mut material = Material::pbr()?;
        material.albedo_buffer(&color_buf);

        let noise = Particles::noise()?;

        Ok(ParticlesNoise {
            sun,
            particle,
            particles,
            material,
            noise,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.light(&self.sun);
        g.background((0.06, 0.06, 0.08));
        g.material(&self.material);
        g.particles(&self.particles).geometry(&self.particle);

        let t = g.elapsed_time();
        self.noise.set("scale", 0.25);
        self.noise.set("strength", 0.02);
        self.noise.set("time", t * 0.5);
        self.particles.apply(&self.noise);
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match (i as i32).rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

fn main() -> Result<()> {
    processing::run::<ParticlesNoise>()
}
