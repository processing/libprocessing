use processing::{
    Attribute, Compute, Geometry, Graphics, Light, Material, Particles, Result, Settings, Shader,
    Sketch,
};

const GRID: u32 = 100; // GRID^3 particles
const SPACING: f32 = 1.0;

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
    if i >= count { return; }
    let cs = cos(params.dt);
    let sn = sin(params.dt);
    let x = position[i * 3u + 0u];
    let z = position[i * 3u + 2u];
    position[i * 3u + 0u] = x * cs - z * sn;
    position[i * 3u + 2u] = x * sn + z * cs;
}
"#;

fn hash_u32(mut x: u32) -> u32 {
    x = (x ^ 61).wrapping_add(x >> 16);
    x = x.wrapping_add(x << 3);
    x ^= x >> 4;
    x = x.wrapping_mul(0x27d4eb2d);
    x ^= x >> 15;
    x
}

fn hash_unit(seed: u32) -> f32 {
    (hash_u32(seed) as f32) / (u32::MAX as f32)
}

struct ParticlesStress {
    red: Light,
    green: Light,
    blue: Light,
    cube: Geometry,
    particles: Particles,
    mat: Material,
    spin: Compute,
}

impl Sketch for ParticlesStress {
    fn settings() -> Settings {
        Settings::size(900, 700)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        let extent = (GRID as f32) * SPACING * 0.5;
        g.camera_position((0.0, -extent * 0.6, extent * 2.5));
        g.camera_look_at((0, 0, 0));
        g.orbit_camera();

        let red = g.create_directional_light((1.0, 0.0, 0.0), 1000.0);
        red.position((1, 0, 0)).look_at((0, 0, 0));

        let green = g.create_directional_light((0.0, 1.0, 0.0), 1000.0);
        green.position((0, -1, 0)).look_at((0, 0, 0));

        let blue = g.create_directional_light((0.0, 0.0, 1.0), 1000.0);
        blue.position((0, 0, 1)).look_at((0, 0, 0));

        let cube = Geometry::cube((0.35, 0.35, 0.35))?;

        let capacity = GRID * GRID * GRID;
        let particles = Particles::new(capacity, &[Attribute::position(), Attribute::color()])?;

        let mut positions: Vec<f32> = Vec::with_capacity(capacity as usize * 3);
        let mut colors: Vec<f32> = Vec::with_capacity(capacity as usize * 4);
        let extent_half = (GRID as f32) * SPACING * 0.5;
        for i in 0..capacity {
            let rx = hash_unit(i.wrapping_mul(2654435761).wrapping_add(0x9E37));
            let ry = hash_unit(i.wrapping_mul(40503).wrapping_add(0x68E1));
            let rz = hash_unit(i.wrapping_mul(2246822519).wrapping_add(0xC2B2));
            positions.push((rx * 2.0 - 1.0) * extent_half);
            positions.push((1.0 - ry * 2.0) * extent_half);
            positions.push((rz * 2.0 - 1.0) * extent_half);
            colors.push(rx);
            colors.push(ry);
            colors.push(rz);
            colors.push(1.0);
        }
        particles.buffer(Attribute::position()).write(&positions);
        let mut color_buf = particles.buffer(Attribute::color());
        color_buf.write(&colors);

        let mut mat = Material::pbr()?;
        mat.albedo_buffer(&color_buf);

        let spin = Compute::new(&Shader::new(SPIN_SHADER)?)?;

        eprintln!("{capacity} particles");

        Ok(ParticlesStress {
            red,
            green,
            blue,
            cube,
            particles,
            mat,
            spin,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.light(&self.red);
        g.light(&self.green);
        g.light(&self.blue);
        g.background((0.04, 0.04, 0.07));
        g.material(&self.mat);
        g.particles(&self.particles).geometry(&self.cube);

        self.spin.set("dt", 0.003);
        self.particles.apply(&self.spin);
    }
}

fn main() -> Result<()> {
    processing::run::<ParticlesStress>()
}
