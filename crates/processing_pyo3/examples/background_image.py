from processing import *

i = None 

def setup():
    global i
    size(800, 600)
    i = image("images/logo.png")


def draw():
    global i
    background(220, 100, 24)
    # i = image("images/logo.png")
    background(i)


# TODO: this should happen implicitly on module load somehow
run()
