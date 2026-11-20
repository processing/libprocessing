use processing::prelude::*;

const AGE_SHADER: &str = r#"
struct Params { dt: f32, ttl: f32, _pad0: f32, _pad1: f32 }

@group(0) @binding(0) var<storage, read_write> scale: array<f32>;
@group(0) @binding(1) var<storage, read_write> age:   array<f32>;
@group(0) @binding(2) var<storage, read_write> life:  array<f32>;
@group(0) @binding(3) var<uniform>             params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&age);
    if i >= count { return; }
    if life[i] <= 0.0 { return; }

    age[i] = age[i] + params.dt;
    let t = age[i] / params.ttl;
    let rise = clamp(t / 0.05, 0.0, 1.0);
    let fall = clamp((1.0 - t) / 0.6, 0.0, 1.0);
    let s = rise * fall;
    scale[i * 3u + 0u] = s;
    scale[i * 3u + 1u] = s;
    scale[i * 3u + 2u] = s;

    if age[i] > params.ttl { life[i] = 0.0; }
}
"#;

const BURST: u32 = 600;
const DT: f32 = 1.0 / 60.0;
const TTL: f32 = 4.0;

struct ParticlesScatter {
    particles: Particles,
    particle: Geometry,
    // read by the scatter kernel
    _source: Geometry,
    scatter: Compute,
    aging: Compute,
    mat: Material,
    key: Light,
}

impl Sketch for ParticlesScatter {
    fn settings() -> Settings {
        Settings::size(900, 700)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((0.0, -0.4, 4.5));
        g.camera_look_at((0, 0, 0));

        let key = g.create_directional_light((1.0, 0.95, 0.85), 4500.0);

        let source = Geometry::sphere(1.2, 96, 48)?;
        let scatter = Particles::scatter_surface(&source)?;

        let particle = Geometry::sphere(0.005, 6, 4)?;

        let age = Attribute::new("age", AttributeFormat::Float)?;
        let particles = Particles::new(
            40_000,
            &[
                Attribute::position(),
                Attribute::scale(),
                Attribute::life(),
                age,
            ],
        )?;

        let aging = Compute::new(&Shader::new(AGE_SHADER)?)?;

        let mut mat = Material::pbr()?;
        mat.set("color", (0.9, 0.85, 1.0, 1.0));

        Ok(ParticlesScatter {
            particles,
            particle,
            _source: source,
            scatter,
            aging,
            mat,
            key,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.light(&self.key);
        g.background((0.03, 0.03, 0.05));
        g.material(&self.mat);
        g.particles(&self.particles).geometry(&self.particle);

        let t = g.elapsed_time();
        let cam_x = (t * 0.25).cos() * 4.5;
        let cam_z = (t * 0.25).sin() * 4.5;
        g.camera_position((cam_x, -0.4, cam_z));
        g.camera_look_at((0, 0, 0));

        let seed = (t * 1000.0) as u32 ^ 0xc0ffee;
        self.scatter.set("seed", seed);
        self.particles.emit_gpu(BURST, &self.scatter);

        self.aging.set("dt", DT);
        self.aging.set("ttl", TTL);
        self.particles.apply(&self.aging);
    }
}

fn main() -> Result<()> {
    processing::run::<ParticlesScatter>()
}
