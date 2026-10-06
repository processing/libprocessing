# Embedding Iteration.
#
# Embedding "for" structures allows repetition in two dimensions.
from mewnala import *

size(640, 360)
background(0.0)

grid_size = 40

for x in range(grid_size, width - grid_size + 1, grid_size):
    for y in range(grid_size, height - grid_size + 1, grid_size):
        no_stroke()
        fill(1.0)
        rect(x - 1, y - 1, 3, 3)
        stroke(1.0, 0.4)
        line(x, y, width / 2, height / 2)

run()
