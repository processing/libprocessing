from processing import *

def setup():
    size(800, 600)

def draw():
    background(220)

    fill(255, 0, 100)
    stroke(1)
    stroke_weight(2)
    rect(100, 100, 200, 150)

# TODO: this should happen implicitly on module load somehow
run()
