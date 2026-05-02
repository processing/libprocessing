from mewnala import *
import math

field_obj = None
sphere = None
mat = None
frame = 0


def setup():
    global field_obj, sphere, mat

    size(900, 700)
    mode_3d()

    sphere = Geometry.sphere(0.08, 8, 6)

    capacity = 2000
    field_obj = Field(
        capacity=capacity,
        attributes=[Attribute.position(), Attribute.color()],
    )

    # Push unemitted slots far off-screen so they don't all render at the
    # origin while the ring buffer is still filling.
    pos_buf = field_obj.buffer(Attribute.position())
    pos_buf.write([1.0e6] * (capacity * 3))

    color_buf = field_obj.buffer(Attribute.color())
    mat = Material.unlit(albedo=color_buf)


def draw():
    global frame
    camera_position(0.0, 4.0, 14.0)
    camera_look_at(0.0, 0.0, 0.0)
    background(15, 15, 20)

    use_material(mat)
    draw_field(field_obj, sphere)

    # Emit 4 particles per frame in an outward-spiraling ring; once the ring
    # buffer fills (~500 frames at 4/frame for capacity 2000), oldest get
    # overwritten and the swirl continues without bound.
    burst = 4
    positions = []
    colors = []
    for k in range(burst):
        i = frame * burst + k
        t = i * 0.05
        radius = 1.5 + min(t * 0.02, 3.0)
        height = math.sin(t * 0.1) * 2.0
        positions.extend([math.cos(t) * radius, height, math.sin(t) * radius])
        c = hsva((i * 4.32) % 360.0, 0.85, 1.0)
        colors.extend([c.r, c.g, c.b, 1.0])

    field_obj.emit(burst, position=positions, color=colors)
    frame += 1


run()
