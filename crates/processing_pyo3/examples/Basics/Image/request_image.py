# Request Image
# by Ira Greenberg
#
# Shows how to use the request_image() function with preloader animation.
# The request_image() function loads images on a separate thread so that
# the sketch does not freeze while they load. It's useful when you are
# loading large images. These images are small for a quick download, but
# try it with your own huge images to get the full effect.
from mewnala import *

img_count = 12
imgs = []
img_w = 0.0

# Keeps track of loaded images (True or False)
load_states = [False] * img_count

# For loading animation
loader_x = 0.0
loader_y = 0.0
theta = 0.0


def setup():
    global img_w
    size(640, 360)
    img_w = width / img_count

    # Load images asynchronously
    for i in range(img_count):
        imgs.append(load_image("data/PT_anim" + str(i).zfill(4) + ".png"))  # GAP: request_image() (threaded load, width 0 until ready) is missing; load_image blocks


def draw():
    background(0.0)

    # Start loading animation
    run_loader_ani()

    for i in range(len(imgs)):
        # Check if individual images are fully loaded
        if imgs[i].width != 0 and imgs[i].width != -1:
            # As images are loaded set True in the list
            load_states[i] = True
    # When all images are loaded draw them to the screen
    if check_load_states():
        draw_images()


def draw_images():
    y = (height - imgs[0].height) // 2
    for i in range(len(imgs)):
        image(imgs[i], width // len(imgs) * i, y, imgs[i].height, imgs[i].height)


# Loading animation
def run_loader_ani():
    global loader_x, loader_y, theta
    # Only run when images are loading
    if not check_load_states():
        ellipse(loader_x, loader_y, 10, 10)
        loader_x += 2
        loader_y = height / 2 + sin(theta) * (height / 8)
        theta += PI / 22
        # Reposition ellipse if it goes off the screen
        if loader_x > width + 5:
            loader_x = -5


# Return True when all images are loaded - no False values left in the list
def check_load_states():
    for i in range(len(imgs)):
        if not load_states[i]:
            return False
    return True


run()
