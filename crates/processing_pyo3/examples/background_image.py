from processing import *

def setup():
    global i
    size(800, 600)
    i = image("images/logo.png")


def draw():
    background(220, 100, 24)
    background(i)


# TODO: this should happen implicitly on module load somehow
run()
