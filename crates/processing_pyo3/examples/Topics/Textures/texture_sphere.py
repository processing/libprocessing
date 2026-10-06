# Texture Sphere
# by Gillian Ramsay
#
# Rewritten by Gillian Ramsay to better display the poles.
# Previous version by Mike 'Flux' Chang (and cleaned up by Aaron Koblin).
# Original based on code by Toxi.
#
# A 3D textured sphere with simple rotation control.
from mewnala import *

pts_w = 0
pts_h = 0

img = None
globe = None

num_points_w = 0
num_points_h_2pi = 0
num_points_h = 0

coor_x = []
coor_y = []
coor_z = []
mult_xz = []


def setup():
    global img, globe, pts_w, pts_h
    size(640, 360)
    mode_3d()
    background(0.0)
    no_stroke()
    img = load_image("data/world32k.jpg")
    pts_w = 30
    pts_h = 30
    # Parameters below are the number of vertices around the width and height
    initialize_sphere(pts_w, pts_h)
    globe = texture_sphere(200, 200, 200, img)


# Use arrow keys to change detail settings
def key_pressed():
    global globe, pts_w, pts_h
    if key_code == ENTER:
        save_frame()
    if key_code == UP:
        pts_h += 1
    if key_code == DOWN:
        pts_h -= 1
    if key_code == LEFT_ARROW:
        pts_w -= 1
    if key_code == RIGHT_ARROW:
        pts_w += 1
    if pts_w == 0:
        pts_w = 1
    if pts_h == 0:
        pts_h = 2
    # Parameters below are the number of vertices around the width and height
    initialize_sphere(pts_w, pts_h)
    globe = texture_sphere(200, 200, 200, img)


def draw():
    background(0.0)
    # The world is centered and y-up, so the eye and center lose the
    # width/2, height/2 offsets and flip their y
    camera(remap(mouse_x, 0, width, -2 * width, 2 * width),
           -remap(mouse_y, 0, height, -height, height),
           height / 2 / tan(PI * 30.0 / 180.0),
           width / 2, 0, 0,
           0, 1, 0)

    push_matrix()
    unlit()
    texture(img)
    draw_geometry(globe)
    no_texture()
    pop_matrix()


def initialize_sphere(num_pts_w, num_pts_h_2pi):
    global num_points_w, num_points_h_2pi, num_points_h, coor_x, coor_y, coor_z, mult_xz

    # The number of points around the width and height
    num_points_w = num_pts_w + 1
    num_points_h_2pi = num_pts_h_2pi  # How many actual pts around the sphere (not just from top to bottom)
    num_points_h = ceil(num_points_h_2pi / 2) + 1  # How many pts from top to bottom (abs(....) b/c of the possibility of an odd numPointsH_2pi)

    coor_x = []  # All the x-coor in a horizontal circle radius 1
    coor_y = []  # All the y-coor in a vertical circle radius 1
    coor_z = []  # All the z-coor in a horizontal circle radius 1
    mult_xz = []  # The radius of each horizontal circle (that you will multiply with coorX and coorZ)

    for i in range(num_points_w):  # For all the points around the width
        theta_w = i * 2 * PI / (num_points_w - 1)
        coor_x.append(sin(theta_w))
        coor_z.append(cos(theta_w))

    for i in range(num_points_h):  # For all points from top to bottom
        if num_points_h_2pi // 2 != num_points_h_2pi / 2 and i == num_points_h - 1:  # If the numPointsH_2pi is odd and it is at the last pt
            theta_h = (i - 1) * 2 * PI / num_points_h_2pi
            coor_y.append(cos(PI + theta_h))
            mult_xz.append(0)
        else:
            # The numPointsH_2pi and 2 below allows there to be a flat bottom if the numPointsH is odd
            theta_h = i * 2 * PI / num_points_h_2pi

            # PI+ below makes the top always the point instead of the bottom.
            coor_y.append(cos(PI + theta_h))
            mult_xz.append(sin(theta_h))


def texture_sphere(rx, ry, rz, t):
    # These are so we can map certain parts of the image on to the shape
    # (uv coordinates are the image coordinates divided by the image size)
    change_u = 1.0 / (num_points_w - 1)
    change_v = 1.0 / (num_points_h - 1)
    u = 0  # Width variable for the texture
    v = 0  # Height variable for the texture

    # The world is y-up, so the y of every vertex and normal is flipped
    g = create_geometry(topology=TRIANGLE_STRIP)
    for i in range(num_points_h - 1):  # For all the rings but top and bottom
        # Goes into the array here instead of loop to save time
        coory = coor_y[i]
        coory_plus = coor_y[i + 1]

        multxz = mult_xz[i]
        multxz_plus = mult_xz[i + 1]

        for j in range(num_points_w):  # For all the pts in the ring
            g.normal(-coor_x[j] * multxz, coory, -coor_z[j] * multxz)
            g.uv(u, v)
            g.vertex(coor_x[j] * multxz * rx, -coory * ry, coor_z[j] * multxz * rz)
            g.normal(-coor_x[j] * multxz_plus, coory_plus, -coor_z[j] * multxz_plus)
            g.uv(u, v + change_v)
            g.vertex(coor_x[j] * multxz_plus * rx, -coory_plus * ry, coor_z[j] * multxz_plus * rz)
            u += change_u
        v += change_v
        u = 0
    return g


run()
