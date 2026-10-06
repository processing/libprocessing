# Storing Input.
#
# Move the mouse across the screen to change the position
# of the circles. The positions of the mouse are recorded
# into an array and played back every frame. Between each
# frame, the newest value are added to the end of each array
# and the oldest value is deleted.
from mewnala import *

num = 60
mx = [0.0] * num
my = [0.0] * num


def setup():
    size(640, 360)
    no_stroke()
    fill(1.0, 0.6)


def draw():
    background(0.2)

    # Cycle through the array, using a different entry on each frame.
    # Using modulo (%) like this is faster than moving all the values over.
    which = frame_count % num
    mx[which] = mouse_x
    my[which] = mouse_y

    for i in range(num):
        # which+1 is the smallest (the oldest in the array)
        index = (which + 1 + i) % num
        ellipse(mx[index], my[index], i, i)


run()
