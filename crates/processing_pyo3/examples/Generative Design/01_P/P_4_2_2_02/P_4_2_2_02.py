# P_4_2_2_02
#
# Generative Gestaltung, ISBN: 978-3-87439-759-9
# First Edition, Hermann Schmidt, Mainz, 2009
# Hartmut Bohnacker, Benedikt Gross, Julia Laub, Claudius Lazzeroni
# Copyright 2009 Hartmut Bohnacker, Benedikt Gross, Julia Laub, Claudius Lazzeroni
#
# http://www.generative-gestaltung.de
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# Ported from the Processing original to processing_py.

# timelapse camera. after each interval_time a picture is saved to the sketch folder
from datetime import datetime

from processing_py import *


def get_timestamp():
    return datetime.now().strftime("%y%m%d_%H%M%S")


my_cam = None

# interval_time in sec. here 5 min
interval_time = 5 * 60

seconds_since_start = 0
start_time = get_timestamp()
counter = 0
do_save = True


def setup():
    global my_cam
    size(640, 480)
    print(webcam_list())  # GAP: no way to list the capture devices (Capture.list())
    # s = "Logitech QuickCam Messenger-WDM"
    # my_cam = create_webcam(s, width, height, 30)
    my_cam = create_webcam(width, height, 30)  # GAP: create_webcam() is in the engine (webcam feature) but not built into this package
    my_cam.start()  # GAP: Webcam.start()

    no_stroke()


def draw():
    global seconds_since_start, do_save, counter
    if my_cam.available():  # GAP: Webcam.available() (a new frame has arrived); only is_connected() exists
        my_cam.read()  # GAP: Webcam.read()
        image(my_cam, 0, 0)

        seconds_since_start = millis() // 1000
        interval = seconds_since_start % interval_time

        if interval == 0 and do_save == True:
            save_file_name = start_time + "-" + str(counter).zfill(5)
            save_frame(save_file_name + ".png")
            do_save = False
            counter += 1
        elif interval != 0:
            do_save = True

        # visualize the time to the next shot
        fill(random(0, 1), random(0, 1), random(0, 1))
        rect(remap(interval, 0, interval_time, 0, width), 0, 5, 5)


run()
