"""Classic TouchDesigner-style feedback patch.

The TD network is:

    source (this frame) ─┐
                         ├─► Composite ─┬─► Out (display)
    Feedback ─► Level ───┘             │
       ▲          (decay)              │
       └───────────────────────────────┘   feedback taps the composite output

i.e. out(t) = composite( source(t), decay * out(t-1) ).

Here the sketch canvas *is* the composite output. Because it isn't cleared each
frame, it persists and feeds back into itself. So:

  - feedback(decay=...)  is the Feedback TOP + Level (samples the previous
    composite output and fades it, optionally zooming/rotating it);
  - the shapes drawn afterwards are this frame's source, composited *over* the
    faded feedback;
  - the canvas persists, closing the loop.
"""
from mewnala import *
from math import sin, cos

t = 0.0


def setup():
    size(800, 600)
    background(0, 0, 0)  # clear once; draw() never clears, so the canvas accumulates


def draw():
    global t

    # --- feedback line: previous composite output, decayed + slowly zoomed/rotated ---
    feedback(decay=0.94, zoom=1.008, angle=0.006)

    # --- this frame's source, composited over the feedback (drawn on top) ---
    no_stroke()

    x = width / 2 + cos(t) * 230
    y = height / 2 + sin(t * 1.3) * 170
    fill(0, 220, 255)
    circle(x, y, 44)

    x2 = width / 2 + cos(t * 0.7 + 2.0) * 150
    y2 = height / 2 + sin(t * 1.1) * 120
    fill(255, 90, 200)
    circle(x2, y2, 28)

    t += 0.03


# TODO: this should happen implicitly on module load somehow
run()
