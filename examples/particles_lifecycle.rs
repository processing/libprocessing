use processing::prelude::*;

const AGING_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read_write> age: array<f32>;
@group(0) @binding(1) var<storage, read_write> life: array<f32>;
@group(0) @binding(2) var<storage, read_write> position: array<f32>;
@group(0) @binding(3) var<storage, read_write> scale: array<f32>;
@group(0) @binding(4) var<uniform> params: vec4<f32>;  // x = dt, y = ttl

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&age);
    if i >= count {
        return;
    }
    let dt = params.x;
    let ttl = params.y;

    if life[i] <= 0.0 {
        return;
    }

    age[i] = age[i] + dt;
    position[i * 3u + 1u] = position[i * 3u + 1u] + dt * 1.5;

    let remaining = clamp(1.0 - age[i] / ttl, 0.0, 1.0);
    let s = remaining * remaining;
    scale[i * 3u + 0u] = s;
    scale[i * 3u + 1u] = s;
    scale[i * 3u + 2u] = s;

    if age[i] > ttl {
        life[i] = 0.0;
    }
}
"#;

const BURST: u32 = 6;
const DT: f32 = 1.0 / 60.0;
const TTL: f32 = 1.0;

struct ParticlesLifecycle {
    sphere: Geometry,
    particles: Particles,
    mat: Material,
    aging: Compute,
    frame: u32,
}

impl Sketch for ParticlesLifecycle {
    fn settings() -> Settings {
        Settings::size(900, 700)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((0.0, -2.0, 14.0));
        g.camera_look_at((0, 0, 0));

        let sphere = Geometry::sphere(0.1, 8, 6)?;

        let capacity: u32 = 800;
        let particles = Particles::new(
            capacity,
            &[
                Attribute::position(),
                Attribute::color(),
                Attribute::scale(),
                Attribute::life(),
                Attribute::age(),
            ],
        )?;

        let mut mat = Material::unlit()?;
        mat.albedo_buffer(&particles.buffer(Attribute::color()));

        let aging = Compute::new(&Shader::new(AGING_SHADER)?)?;

        Ok(ParticlesLifecycle {
            sphere,
            particles,
            mat,
            aging,
            frame: 0,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background((0.04, 0.04, 0.07));
        g.material(&self.mat);
        g.particles(&self.particles).geometry(&self.sphere);

        let mut positions: Vec<Vec3> = Vec::with_capacity(BURST as usize);
        let mut colors: Vec<Color> = Vec::with_capacity(BURST as usize);
        let mut scales: Vec<Vec3> = Vec::with_capacity(BURST as usize);
        let mut ages: Vec<f32> = Vec::with_capacity(BURST as usize);
        let mut lives: Vec<f32> = Vec::with_capacity(BURST as usize);
        for k in 0..BURST {
            let i = self.frame * BURST + k;
            let u = ((i.wrapping_mul(2654435761) >> 8) & 0xFFFF) as f32 / 65535.0;
            let v = ((i.wrapping_mul(40503) >> 8) & 0xFFFF) as f32 / 65535.0;
            let theta = u * TAU;
            let r = v * 0.6;
            positions.push(Vec3::new(theta.cos() * r, -2.5, theta.sin() * r));
            let h = (i as f32 * 0.013) % 1.0;
            colors.push(Color::hsv(h * 360.0, 0.85, 1.0));
            scales.push(Vec3::new(1.0, 1.0, 1.0));
            ages.push(0.0);
            lives.push(1.0);
        }
        self.particles.emit(
            BURST,
            &[
                (Attribute::position(), &positions),
                (Attribute::color(), &colors),
                (Attribute::scale(), &scales),
                (Attribute::age(), &ages),
                (Attribute::life(), &lives),
            ],
        );

        self.aging.set("params", (DT, TTL, 0.0, 0.0));
        self.particles.apply(&self.aging);

        self.frame += 1;
    }
}

fn main() -> Result<()> {
    processing::run::<ParticlesLifecycle>()
}
