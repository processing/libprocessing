use processing::prelude::*;

const SPIN_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read_write> rotation: array<f32>;
@group(0) @binding(1) var<uniform> params: vec4<f32>;  // x = dt

fn quat_mul(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        a.w * b.xyz + b.w * a.xyz + cross(a.xyz, b.xyz),
        a.w * b.w - dot(a.xyz, b.xyz),
    );
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&rotation) / 4u;
    if i >= count {
        return;
    }
    let dt = params.x;
    let q = vec4<f32>(
        rotation[i * 4u + 0u],
        rotation[i * 4u + 1u],
        rotation[i * 4u + 2u],
        rotation[i * 4u + 3u],
    );
    let half_angle = dt * 0.5;
    let dq = vec4<f32>(0.0, sin(half_angle), 0.0, cos(half_angle));
    let q_new = quat_mul(q, dq);
    rotation[i * 4u + 0u] = q_new.x;
    rotation[i * 4u + 1u] = q_new.y;
    rotation[i * 4u + 2u] = q_new.z;
    rotation[i * 4u + 3u] = q_new.w;
}
"#;

struct ParticlesOriented {
    sun: Light,
    cube: Geometry,
    particles: Particles,
    pbr: Material,
    spin: Compute,
}

impl Sketch for ParticlesOriented {
    fn settings() -> Settings {
        Settings::size(900, 700)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((0.0, -6.0, 18.0));
        g.camera_look_at((0, 0, 0));

        let sun = g.create_directional_light((0.9, 0.85, 0.8), 300.0);

        let cube = Geometry::cube((0.6, 0.6, 0.6))?;

        let capacity: u32 = 125;
        let mut positions: Vec<Vec3> = Vec::with_capacity(capacity as usize);
        let mut rotations: Vec<Vec4> = Vec::with_capacity(capacity as usize);
        let mut scales: Vec<Vec3> = Vec::with_capacity(capacity as usize);
        for x in 0..5 {
            for y in 0..5 {
                for z in 0..5 {
                    positions.push(Vec3::new(
                        (x as f32 - 2.0) * 1.6,
                        (2.0 - y as f32) * 1.6,
                        (z as f32 - 2.0) * 1.6,
                    ));
                    rotations.push(Vec4::new(0.0, 0.0, 0.0, 1.0));
                    let s = 0.5 + ((x + y + z) as f32 * 0.06);
                    scales.push(Vec3::new(s, s, s));
                }
            }
        }

        let particles = Particles::new(
            capacity,
            &[
                Attribute::position(),
                Attribute::rotation(),
                Attribute::scale(),
            ],
        )?;
        particles.buffer(Attribute::position()).write(&positions);
        particles.buffer(Attribute::rotation()).write(&rotations);
        particles.buffer(Attribute::scale()).write(&scales);

        let mut pbr = Material::pbr()?;
        pbr.set("roughness", 0.4);

        let spin = Compute::new(&Shader::new(SPIN_SHADER)?)?;

        Ok(ParticlesOriented {
            sun,
            cube,
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
        g.particles(&self.particles).geometry(&self.cube);

        self.spin.set("params", (0.015, 0.0, 0.0, 0.0));
        self.particles.apply(&self.spin);
    }
}

fn main() -> Result<()> {
    processing::run::<ParticlesOriented>()
}
