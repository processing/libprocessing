# Wiggling
#
# Press 'w' to start wiggling, space to restore
# original positions. Press '1', '2' or '3' to change the stroke weight.
from mewnala import *

cube = []  # six faces, each a list of [x, y, z] vertices
cube_size = 320
circle_rad = 100
circle_res = 40
noise_mag = 1
line_weight = 1

wiggling = False


def setup():
    size(1024, 768)
    mode_3d()

    create_cube()


def draw():
    background(0.0)

    rotate_x(frame_count * 0.01)
    rotate_y(frame_count * 0.01)

    draw_cube()

    if wiggling:
        for face in cube:
            for pos in face:
                pos[0] += random(-noise_mag / 2, +noise_mag / 2)
                pos[1] += random(-noise_mag / 2, +noise_mag / 2)
                pos[2] += random(-noise_mag / 2, +noise_mag / 2)

    if frame_count % 60 == 0:
        print(frame_rate())


def key_pressed():
    global wiggling, line_weight
    if key == "w":
        wiggling = not wiggling
    elif key == " ":
        restore_cube()
    elif key == "1":
        line_weight = 1
    elif key == "2":
        line_weight = 5
    elif key == "3":
        line_weight = 10


def create_cube():
    # Create all faces at front position
    for i in range(6):
        face = []
        create_face_with_hole(face)
        cube.append(face)


def create_face_with_hole(face):
    # Draw main shape Clockwise
    face.append([-cube_size / 2, -cube_size / 2, +cube_size / 2])
    face.append([+cube_size / 2, -cube_size / 2, +cube_size / 2])
    face.append([+cube_size / 2, +cube_size / 2, +cube_size / 2])
    face.append([-cube_size / 2, +cube_size / 2, +cube_size / 2])

    # Draw contour (hole) Counter-Clockwise
    for i in range(circle_res):
        angle = TWO_PI * i / circle_res
        x = circle_rad * sin(angle)
        y = circle_rad * cos(angle)
        z = +cube_size / 2
        face.append([x, y, z])


# Each face is drawn at the front position and rotated into place
def draw_cube():
    rotations = [
        (0, 0),  # Front face
        (0, radians(180)),  # Back face
        (0, radians(90)),  # Right face
        (0, radians(-90)),  # Left face
        (radians(90), 0),  # Top face
        (radians(-90), 0),  # Bottom face
    ]
    for i in range(6):
        push_matrix()
        rotate_x(rotations[i][0])
        rotate_y(rotations[i][1])
        draw_face_with_hole(cube[i])
        pop_matrix()


def draw_face_with_hole(face):
    stroke(1.0, 0.0, 0.0)
    stroke_weight(line_weight)
    fill(1.0)
    # The whole face sits on the z = cube_size / 2 plane
    translate(0, 0, cube_size / 2)
    begin_shape(POLYGON)
    for i in range(4):
        vertex(face[i][0], face[i][1])  # GAP: vertex(x, y, z) in immediate mode; the z wiggle is not drawn
    begin_contour()
    for i in range(circle_res):
        vertex(face[4 + i][0], face[4 + i][1])
    end_contour()
    end_shape(CLOSE)


def restore_cube():
    # Rotation of faces is preserved, so we just reset them
    # the same way as the "front" face and they will stay
    # rotated correctly
    for face in cube:
        restore_face_with_hole(face)


def restore_face_with_hole(face):
    face[0] = [-cube_size / 2, -cube_size / 2, +cube_size / 2]
    face[1] = [+cube_size / 2, -cube_size / 2, +cube_size / 2]
    face[2] = [+cube_size / 2, +cube_size / 2, +cube_size / 2]
    face[3] = [-cube_size / 2, +cube_size / 2, +cube_size / 2]
    for i in range(circle_res):
        angle = TWO_PI * i / circle_res
        x = circle_rad * sin(angle)
        y = circle_rad * cos(angle)
        z = +cube_size / 2
        face[4 + i] = [x, y, z]


run()
