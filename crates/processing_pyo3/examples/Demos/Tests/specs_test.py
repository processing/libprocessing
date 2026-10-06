# Specs Test
#
# The Java sketch prints the OpenGL vendor, renderer, version, GLSL version
# and extensions. mewnala exposes no GPU adapter information, so this prints
# what it does know about the display and the sketch surface.
from mewnala import *

size(100, 100)
mode_3d()
print("display:", display_width, "x", display_height, "density", display_density())
print("pixel density:", pixel_density(), "canvas:", pixel_width, "x", pixel_height)
print("monitors:", len(monitors()))
print("fonts:", len(list_fonts()))
# GAP: no renderer_info() (GPU vendor, renderer, graphics API version, extensions)

run()
