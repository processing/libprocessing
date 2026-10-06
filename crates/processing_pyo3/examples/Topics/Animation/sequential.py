# Sequential
# by James Paterson.
#
# Displaying a sequence of images creates the illusion of motion.
# Twelve images are loaded and each is displayed individually in a loop.
from mewnala import *

num_frames = 12  # The number of frames in the animation
current_frame = 0
images = []


def setup():
    size(640, 360)
    frame_rate(24)

    images.append(load_image("data/PT_anim0000.png"))
    images.append(load_image("data/PT_anim0001.png"))
    images.append(load_image("data/PT_anim0002.png"))
    images.append(load_image("data/PT_anim0003.png"))
    images.append(load_image("data/PT_anim0004.png"))
    images.append(load_image("data/PT_anim0005.png"))
    images.append(load_image("data/PT_anim0006.png"))
    images.append(load_image("data/PT_anim0007.png"))
    images.append(load_image("data/PT_anim0008.png"))
    images.append(load_image("data/PT_anim0009.png"))
    images.append(load_image("data/PT_anim0010.png"))
    images.append(load_image("data/PT_anim0011.png"))

    # If you don't want to load each image separately
    # and you know how many frames you have, you
    # can create the filenames as the program runs.
    # Zero-padded formatting ensures that the number
    # is (in this case) 4 digits.
    # for i in range(num_frames):
    #     image_name = "data/PT_anim" + str(i).zfill(4) + ".png"
    #     images.append(load_image(image_name))


def draw():
    global current_frame
    background(0.0)
    current_frame = (current_frame + 1) % num_frames  # Use % to cycle through frames
    offset = 0
    x = -100
    while x < width:
        image(images[(current_frame + offset) % num_frames], x, -20)
        offset += 2
        image(images[(current_frame + offset) % num_frames], x, height / 2)
        offset += 2
        x += images[0].width


run()
