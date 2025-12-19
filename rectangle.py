from processing import *

def moon_func():
    print("Hello, Moon!")
    
def setup():
    print("HELLO")
    size(800, 600)

def draw():
    background(220)

    fill(255, 0, 100)
    stroke(0)
    stroke_weight(2)
    rect(100, 100, 200, 150)

# TODO: this should happen implicitly on module load somehow
run()
