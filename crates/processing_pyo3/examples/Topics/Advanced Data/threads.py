# Thread function example
# by Daniel Shiffman.
#
# This example demonstrates how to use threading to spawn
# a process that happens outside of the main animation thread.
#
# When the thread is started, the draw() loop will continue while
# the code inside the function passed to the Thread will operate
# in the background.
from mewnala import *
import threading
from urllib.request import Request, urlopen

# This sketch will load data from all of these URLs in a separate thread
urls = [
    "http://processing.org",
    "http://www.processing.org/exhibition/",
    "http://www.processing.org/reference/",
    "http://www.processing.org/reference/libraries",
    "http://www.processing.org/reference/tools",
    "http://www.processing.org/reference/environment",
    "http://www.processing.org/learning/",
    "http://www.processing.org/learning/basics/",
    "http://www.processing.org/learning/topics/",
    "http://www.processing.org/learning/gettingstarted/",
    "http://www.processing.org/download/",
    "http://www.processing.org/shop/",
    "http://www.processing.org/about/",
]

# This will keep track of whether the thread is finished
finished = False
# And how far along
percent = 0

# A variable to keep all the data loaded
all_data = ""


def setup():
    size(640, 360)
    # Spawn the thread!
    threading.Thread(target=load_data).start()


def draw():
    background(0.0)

    # If we're not finished draw a "loading bar"
    # This is so that we can see the progress of the thread
    # This would not be necessary in a sketch where you wanted to load data in the background
    # and hide this from the user, allowing the draw() loop to simply continue
    if not finished:
        stroke(1.0)
        no_fill()
        rect(width / 2 - 150, height / 2, 300, 10)
        fill(1.0)
        # The size of the rectangle is mapped to the percentage completed
        w = remap(percent, 0, 1, 0, 300)
        rect(width / 2 - 150, height / 2, w, 10)
        text_size(16)
        text_align(CENTER)
        fill(1.0)
        text("Loading", width / 2, height / 2 + 30)
    else:
        # The thread is complete!
        text_align(CENTER)
        text_size(24)
        fill(1.0)
        text("Finished loading. Click the mouse to load again.", width / 2, height / 2)


def mouse_pressed():
    threading.Thread(target=load_data).start()


def load_data():
    global finished, percent, all_data
    # The thread is not completed
    finished = False
    # Reset the data to empty
    all_data = ""

    # Look at each URL
    # This example is doing some highly arbitrary things just to make it take longer
    # If you had a lot of data parsing you needed to do, this can all happen in the background
    for i in range(len(urls)):
        # The site rejects the default Python user agent
        request = Request(urls[i], headers={"User-Agent": "Mozilla/5.0"})
        all_txt = urlopen(request).read().decode("utf-8")
        # Demonstrating some arbitrary text splitting, joining, and sorting to make the thread take longer
        for d in "\t+\n <>=\\-!@#$%^&*(),.;:/?\"'":
            all_txt = all_txt.replace(d, " ")
        words = all_txt.split()
        for j in range(len(words)):
            words[j] = words[j].strip()
            words[j] = words[j].lower()
        words = sorted(words)
        all_data += " ".join(words)
        percent = i / len(urls)

    words = all_data.split(" ")
    words = sorted(words)
    all_data = " ".join(words)

    # The thread is completed!
    finished = True


run()
