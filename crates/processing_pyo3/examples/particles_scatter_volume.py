from mewnala import *

CAPACITY = 30_000
BURST = 250

p = None
particle = None
mat = None
scatter = None


def setup():
    global p, particle, mat, scatter

    size(900, 700)
    mode_3d()

    camera_position(0.0, 100.0, 400.0)
    camera_look_at(0.0, 80.0, 0.0)
    orbit_camera()

    gltf = load_gltf("gltf/Duck.glb")
    duck = gltf.geometry("LOD3spShape")
    scatter = Particles.scatter_volume(duck)

    particle = Geometry.sphere(0.15, 4, 3)

    p = create_particles(
        capacity=CAPACITY,
        attributes=[
            Attribute.position(),
            Attribute.scale(),
            Attribute.life(),
            Attribute.age(),
        ],
    )
    mat = create_material(unlit=True, albedo=[1.0, 1.0, 1.0, 1.0])


def draw():
    background(8, 8, 13)
    material(mat)
    particles(p, particle)

    seed = (int(elapsed_time * 1000.0) ^ 0xC0FFEE) & 0xFFFFFFFF
    scatter.set(seed=seed)
    p.emit_gpu(BURST, scatter)
    p.apply(MAP, a="scale", op=AFFINE, scale=0.985, offset=0.0)


run()
