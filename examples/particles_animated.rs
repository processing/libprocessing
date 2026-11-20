use processing::prelude::*;

const SPIN_SHADER: &str = r#"
struct Params {
    dt: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count {
        return;
    }
    let cs = cos(params.dt);
    let sn = sin(params.dt);
    let x = position[i * 3u + 0u];
    let z = position[i * 3u + 2u];
    position[i * 3u + 0u] = x * cs - z * sn;
    position[i * 3u + 2u] = x * sn + z * cs;
}
"#;

struct ParticlesAnimated {
    sun: Light,
    sphere: Geometry,
    particles: Particles,
    pbr: Material,
    spin: Compute,
}

impl Sketch for ParticlesAnimated {
    fn settings() -> Settings {
        Settings::size(900, 700)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((0.0, -8.0, 25.0));
        g.camera_look_at((0, 0, 0));

        let sun = g.create_directional_light((0.9, 0.85, 0.8), 300.0);

        let sphere = Geometry::sphere(0.25, 12, 8)?;

        let capacity: u32 = 1000;
        let mut positions: Vec<Vec3> = Vec::with_capacity(capacity as usize);
        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    positions.push(Vec3::new(
                        (x as f32 - 4.5) * 1.0,
                        (y as f32 - 4.5) * 1.0,
                        (z as f32 - 4.5) * 1.0,
                    ));
                }
            }
        }

        let particles = Particles::new(capacity, &[Attribute::position()])?;
        particles.buffer(Attribute::position()).write(&positions);

        let mut pbr = Material::pbr()?;
        pbr.set("roughness", 0.4);

        let spin = Compute::new(&Shader::new(SPIN_SHADER)?)?;

        Ok(ParticlesAnimated {
            sun,
            sphere,
            particles,
            pbr,
            spin,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.light(&self.sun);
        g.background((0.06, 0.06, 0.08));
        g.fill((0.9, 0.5, 0.3));
        g.material(&self.pbr);
        g.particles(&self.particles).geometry(&self.sphere);

        self.spin.set("dt", 0.01);
        self.particles.apply(&self.spin);
    }
}

fn main() -> Result<()> {
    processing::run::<ParticlesAnimated>()
}
