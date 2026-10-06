# Elevated
# https://www.shadertoy.com/view/MdX3Rr by inigo quilez
# Created by inigo quilez - iq/2013
# License Creative Commons Attribution-NonCommercial-ShareAlike 3.0 Unported License.
# Processing port by Raphaël de Courville.
from mewnala import *

landscape = None


def setup():
    global landscape
    size(640, 360)
    no_stroke()

    # This WESL code shows how to use shaders from
    # shadertoy in mewnala with minimal changes.
    landscape = load_shader("data/landscape.wesl")


def draw():
    background(0.0)

    filter(landscape, time=millis() / 1000.0, mouse=(float(mouse_x), float(mouse_y)))

    if frame_count % 10 == 0:  # every 10th frame
        print("frame: " + str(frame_count) + " - fps: " + str(frame_rate()))


run()
