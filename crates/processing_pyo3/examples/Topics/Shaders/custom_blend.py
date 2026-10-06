# Custom Blend
#
# The OpenGL-based renderers (P2D and P3D) only support some of the
# blending modes available in the default renderer. The reason for this
# is that the blend equations in OpenGL allow for combinations of the
# form dest_factor * dest_color + src_factor * src_color of the source and
# destination colors (see this page http://www.opengl.org/wiki/Blending
# for an extensive discussion of blending in OpenGL).
# Complex blending modes typically available in photo editing tools,
# like hard light or dodge, cannot be modeled with those equations.
# However, we can implement virtually any blending math directly in the
# fragment shader.
#
# This example shows how custom blend shaders can be loaded and used in
# mewnala.
# For detailed information on how to implement Photoshop-like blending modes,
# check the following pages (a bit old but still useful):
# http://www.pegtop.net/delphi/articles/blendmodes/index.htm
# http://mouaif.wordpress.com/2009/01/05/photoshop-math-with-glsl-shaders/
from mewnala import *

dest_image = None
src_image = None
dodge = None
burn = None
overlay = None
difference = None
dodge_output = None
burn_output = None
overlay_output = None
difference_output = None


def setup():
    global dest_image, src_image
    global dodge_output, burn_output, overlay_output, difference_output
    size(640, 360)
    dest_image = load_image("data/leaves.jpg")
    src_image = load_image("data/moonwalk.jpg")

    # Each blend is rendered on its own offscreen canvas, the size of
    # the quad it is displayed on.
    dodge_output = create_graphics(width // 2, height // 2)
    burn_output = create_graphics(width // 2, height // 2)
    overlay_output = create_graphics(width // 2, height // 2)
    difference_output = create_graphics(width // 2, height // 2)

    init_shaders()


def draw():
    background(0.0)

    draw_output(dodge, dodge_output, 0, 0, width / 2, height / 2)
    draw_output(burn, burn_output, width / 2, 0, width / 2, height / 2)
    draw_output(overlay, overlay_output, 0, height / 2, width / 2, height / 2)
    draw_output(difference, difference_output, width / 2, height / 2, width / 2, height / 2)

    no_loop()


def init_shaders():
    global dodge, burn, overlay, difference
    dodge = load_shader("data/dodge.wesl")
    burn = load_shader("data/burn.wesl")
    overlay = load_shader("data/overlay.wesl")
    difference = load_shader("data/difference.wesl")


def draw_output(shader, output, x, y, w, h):
    # The names destination and source come from the OpenGL terminology:
    # destination from the image already in the framebuffer, or "base layer",
    # and source for the image that will be blended into the framebuffer, or
    # "blend layer". The filter samples both over its whole canvas; the sizes
    # of the images and the rectangular areas used for blending are set with it.
    output.begin_draw()
    output.filter(shader,
                  dest_texture=dest_image, src_texture=src_image,
                  dest_size=(640.0, 360.0), dest_rect=(100.0, 50.0, 200.0, 200.0),
                  src_size=(640.0, 360.0), src_rect=(0.0, 0.0, 640.0, 360.0))
    output.end_draw()
    image(output, x, y, w, h)


run()
