# Save Frames
# by Daniel Shiffman.
#
# This example demonstrates how to use save_frame() to render
# out an image sequence that you can assemble into a movie.
from mewnala import *

# A boolean to track whether we are recording are not
recording = False


def setup():
    size(640, 360)


def draw():
    background(0.0)

    # An arbitrary oscillating rotating animation
    # so that we have something to render
    a = 0.0
    while a < TWO_PI:
        push_matrix()
        translate(width / 2, height / 2)
        rotate(a + sin(frame_count * 0.004 * a))
        stroke(1.0)
        line(-100, 0, 100, 0)
        pop_matrix()
        a += 0.2

    # If we are recording call save_frame!
    # The number signs (#) indicate to number the files automatically
    if recording:
        save_frame("output/frames####.png")

    # Let's draw some stuff to tell us what is happening
    # It's important to note that none of this will show up in the
    # rendered files b/c it is drawn *after* save_frame()
    text_align(CENTER)
    fill(1.0)
    if not recording:
        text("Press r to start recording.", width / 2, height - 24)
    else:
        text("Press r to stop recording.", width / 2, height - 24)

    # A red dot for when we are recording
    stroke(1.0)
    if recording:
        fill(1.0, 0.0, 0.0)
    else:
        no_fill()
    ellipse(width / 2, height - 48, 16, 16)


def key_pressed():
    global recording
    # If we press r, start or stop recording!
    if key == "r" or key == "R":
        recording = not recording


run()
