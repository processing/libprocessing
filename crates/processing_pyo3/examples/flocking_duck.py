# Flocking inside a duck: the GPU boids from flocking_gpu.py, seeded from
# the vertices of the Duck glTF mesh. Each boid remembers its spawn vertex
# in a `home` attribute; a homing force that is negligible near home but
# grows quadratically with distance lets the boids swirl and flock locally
# while the swarm as a whole never loses the duck's shape.
from mewnala import *
from math import cos, sin
from random import uniform

DT = 1.0 / 60.0

# Pass 1: Reynolds' three rules plus the tether. Every boid reads the whole
# flock's state and writes only its steering force, so no boid ever sees a
# half-updated neighbor.
FLOCK_SHADER = """
struct Params {
    neighbor_dist: f32,
    separation_dist: f32,
    max_speed: f32,
    max_force: f32,
    home_radius: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<storage, read_write> home: array<f32>;
@group(0) @binding(3) var<storage, read_write> steer: array<f32>;
@group(0) @binding(4) var<uniform> params: Params;

fn limit(v: vec3<f32>, max_len: f32) -> vec3<f32> {
    let len = length(v);
    if len > max_len { return v * (max_len / len); }
    return v;
}

// Reynolds: steering = desired - velocity
fn steer_toward(desired: vec3<f32>, vel: vec3<f32>) -> vec3<f32> {
    let len = length(desired);
    if len < 1e-6 { return vec3<f32>(0.0); }
    return limit(desired * (params.max_speed / len) - vel, params.max_force);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count { return; }

    let pos = vec3<f32>(position[i * 3u], position[i * 3u + 1u], position[i * 3u + 2u]);
    let vel = vec3<f32>(velocity[i * 3u], velocity[i * 3u + 1u], velocity[i * 3u + 2u]);

    var separation = vec3<f32>(0.0);
    var alignment = vec3<f32>(0.0);
    var cohesion = vec3<f32>(0.0);
    var separation_count = 0u;
    var neighbor_count = 0u;

    for (var j = 0u; j < count; j = j + 1u) {
        if j == i { continue; }
        let other = vec3<f32>(position[j * 3u], position[j * 3u + 1u], position[j * 3u + 2u]);
        let d = distance(pos, other);
        if d > 0.0 && d < params.separation_dist {
            // Point away from the neighbor, weighted by closeness
            separation = separation + normalize(pos - other) / d;
            separation_count = separation_count + 1u;
        }
        if d < params.neighbor_dist {
            alignment = alignment
                + vec3<f32>(velocity[j * 3u], velocity[j * 3u + 1u], velocity[j * 3u + 2u]);
            cohesion = cohesion + other;
            neighbor_count = neighbor_count + 1u;
        }
    }

    var force = vec3<f32>(0.0);
    if separation_count > 0u {
        force = force + steer_toward(separation / f32(separation_count), vel) * 1.5;
    }
    if neighbor_count > 0u {
        force = force + steer_toward(alignment, vel);
        force = force + steer_toward(cohesion / f32(neighbor_count) - pos, vel);
    }

    // The tether: inside home_radius the weight is < 1 and flocking wins;
    // past it the pull grows quadratically until it dominates everything.
    let home_pos = vec3<f32>(home[i * 3u], home[i * 3u + 1u], home[i * 3u + 2u]);
    let to_home = home_pos - pos;
    let w = length(to_home) / params.home_radius;
    force = force + steer_toward(to_home, vel) * min(w * w, 8.0);

    steer[i * 3u] = force.x;
    steer[i * 3u + 1u] = force.y;
    steer[i * 3u + 2u] = force.z;
}
"""

# Pass 2: integrate the steering force and point each instanced boid along
# its velocity. No wrapping — the tether is the only containment needed.
INTEGRATE_SHADER = """
struct Params {
    dt: f32,
    max_speed: f32,
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(0) var<storage, read_write> position: array<f32>;
@group(0) @binding(1) var<storage, read_write> velocity: array<f32>;
@group(0) @binding(2) var<storage, read_write> steer: array<f32>;
@group(0) @binding(3) var<storage, read_write> rotation: array<f32>;
@group(0) @binding(4) var<uniform> params: Params;

// shortest-arc quaternion rotating the mesh's +Z axis onto dir
fn quat_z_to(dir: vec3<f32>) -> vec4<f32> {
    let z = vec3<f32>(0.0, 0.0, 1.0);
    let d = dot(z, dir);
    if d < -0.9999 { return vec4<f32>(0.0, 1.0, 0.0, 0.0); }
    return normalize(vec4<f32>(cross(z, dir), 1.0 + d));
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = arrayLength(&position) / 3u;
    if i >= count { return; }

    var pos = vec3<f32>(position[i * 3u], position[i * 3u + 1u], position[i * 3u + 2u]);
    var vel = vec3<f32>(velocity[i * 3u], velocity[i * 3u + 1u], velocity[i * 3u + 2u]);
    let force = vec3<f32>(steer[i * 3u], steer[i * 3u + 1u], steer[i * 3u + 2u]);

    vel = vel + force * params.dt;
    let speed = length(vel);
    if speed > params.max_speed { vel = vel * (params.max_speed / speed); }
    pos = pos + vel * params.dt;

    position[i * 3u] = pos.x;
    position[i * 3u + 1u] = pos.y;
    position[i * 3u + 2u] = pos.z;
    velocity[i * 3u] = vel.x;
    velocity[i * 3u + 1u] = vel.y;
    velocity[i * 3u + 2u] = vel.z;

    if speed > 1e-6 {
        let q = quat_z_to(vel / speed);
        rotation[i * 4u] = q.x;
        rotation[i * 4u + 1u] = q.y;
        rotation[i * 4u + 2u] = q.z;
        rotation[i * 4u + 3u] = q.w;
    }
}
"""

p = None
boid = None
mat = None
flock_pass = None
integrate_pass = None
center = None
extent = 0.0
max_speed = 0.0


# Two triangles folded slightly along the nose-tail spine, like a paper
# boid pointing down +Z. The fold keeps the boid visible edge-on and gives
# each wing its own normal, so the flock glints as it banks.
def boid_geometry(half_width, length, droop):
    g = Geometry()
    n = (half_width * half_width + droop * droop) ** 0.5
    nose = (0.0, 0.0, length * 0.5)
    tail = (0.0, 0.0, -length * 0.5)
    g.normal(-droop / n, half_width / n, 0.0)
    g.vertex(*nose)
    g.vertex(-half_width, -droop, -length * 0.5)
    g.vertex(*tail)
    g.normal(droop / n, half_width / n, 0.0)
    g.vertex(*nose)
    g.vertex(*tail)
    g.vertex(half_width, -droop, -length * 0.5)
    for i in range(6):
        g.index(i)
    return g


def setup():
    global p, boid, mat, flock_pass, integrate_pass, center, extent, max_speed

    size(900, 700)
    mode_3d()

    directional_light((0.95, 0.9, 0.85), 800.0)

    gltf = load_gltf("gltf/Duck.glb")
    duck = gltf.geometry("LOD3spShape")

    velocity_attr = Attribute("velocity", AttributeFormat.Float3)
    home_attr = Attribute("home", AttributeFormat.Float3)
    steer_attr = Attribute("steer", AttributeFormat.Float3)

    p = Particles(
        geometry=duck,
        attributes=[
            Attribute.position(),
            Attribute.rotation(),
            Attribute.color(),
            velocity_attr,
            home_attr,
            steer_attr,
        ],
    )

    # The duck's vertices become the boids' homes. Every tuning constant is
    # derived from the mesh's bounding box, so the sketch doesn't care what
    # units the model was authored in.
    homes = p.buffer(Attribute.position()).read()
    lo = [min(v[i] for v in homes) for i in range(3)]
    hi = [max(v[i] for v in homes) for i in range(3)]
    center = [(lo[i] + hi[i]) * 0.5 for i in range(3)]
    extent = sum((hi[i] - lo[i]) ** 2 for i in range(3)) ** 0.5
    max_speed = 0.15 * extent

    p.buffer(home_attr).write(homes)

    velocities = []
    rotations = []
    colors = []
    for _ in homes:
        velocities.append([uniform(-1.0, 1.0) * max_speed * 0.4 for _ in range(3)])
        rotations.append([0.0, 0.0, 0.0, 1.0])
        c = hsva(uniform(38.0, 58.0), 0.85, 1.0)
        colors.append([c.r, c.g, c.b, 1.0])

    p.buffer(velocity_attr).write(velocities)
    p.buffer(Attribute.rotation()).write(rotations)
    color_buf = p.buffer(Attribute.color())
    color_buf.write(colors)

    s = 0.008 * extent
    boid = boid_geometry(1.2 * s, 3.5 * s, 0.4 * s)
    mat = Material.pbr(albedo=color_buf)

    flock_pass = Compute(Shader(FLOCK_SHADER))
    integrate_pass = Compute(Shader(INTEGRATE_SHADER))


def draw():
    t = elapsed_time * 0.2
    r = extent * 1.1
    camera_position(center[0] + cos(t) * r, center[1] + extent * 0.35, center[2] + sin(t) * r)
    camera_look_at(center[0], center[1], center[2])
    background(10, 12, 18)

    use_material(mat)
    particles(p, boid)

    flock_pass.set(
        neighbor_dist=0.06 * extent,
        separation_dist=0.03 * extent,
        max_speed=max_speed,
        max_force=2.0 * max_speed,
        home_radius=0.04 * extent,
    )
    p.apply(flock_pass)

    integrate_pass.set(dt=DT, max_speed=max_speed)
    p.apply(integrate_pass)


run()
