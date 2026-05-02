from mewnala import *

field_obj = None
sphere = None
mat = None
spin = None

SPIN_SHADER = """
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
"""


def setup():
    global field_obj, sphere, mat, spin

    size(900, 700)
    mode_3d()

    create_directional_light((0.9, 0.85, 0.8), 300.0)

    sphere = Geometry.sphere(0.25, 12, 8)

    capacity = 1000
    positions = []
    for x in range(10):
        for y in range(10):
            for z in range(10):
                positions.extend([x - 4.5, y - 4.5, z - 4.5])

    field_obj = Field(capacity=capacity, attributes=[Attribute.position()])
    pos_buf = field_obj.buffer(Attribute.position())
    pos_buf.write(positions)

    mat = Material(roughness=0.4)
    spin = Compute(Shader(SPIN_SHADER))


def draw():
    camera_position(0.0, 8.0, 25.0)
    camera_look_at(0.0, 0.0, 0.0)
    background(15, 15, 20)
    fill(230, 128, 75)

    use_material(mat)
    draw_field(field_obj, sphere)

    spin.set(dt=0.01)
    field_obj.apply(spin)


run()
