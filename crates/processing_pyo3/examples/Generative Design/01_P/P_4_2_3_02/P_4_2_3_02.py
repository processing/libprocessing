# P_4_2_3_02
#
# Generative Gestaltung – Creative Coding im Web
# ISBN: 978-3-87439-902-9, First Edition, Hermann Schmidt, Mainz, 2018
# Benedikt Groß, Hartmut Bohnacker, Julia Laub, Claudius Lazzeroni
# with contributions by Joey Lee and Niels Poldervaart
# Copyright 2018
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
# Ported from the p5.js original to processing_py.

# Create montage of video with a search query for Part Of Speech.
#
# CONTRIBUTED BY
# [Niels Poldervaart](http://NielsPoldervaart.nl)
#
# VIDEO CREDITS
# European Space Agency. (2016, December 23). The amazing adventures of Rosetta and Philae. Retrieved from http://m.esa.int/spaceinvideos/Videos/2016/12/The_amazing_adventures_of_Rosetta_and_Philae
#
# The QuickSettings panel of the original is not ported; its parameters
# (video_src, subtitle_src, search_query, tile_mode) are the variables below, and
# its buttons are the functions generate_montage() and toggle_playback().
# The original is a web page: the video is an HTML element lying over the canvas and
# the dialogs are HTML spans. Here the video is drawn onto the canvas in draw(), the
# dialogs are drawn as text, and the setTimeout() fragment timer is checked in draw().
import re

from processing_py import *


class SubTitleObject:
    def __init__(self, start_time, end_time, dialog):
        self.start_time_stamp = start_time
        self.end_time_stamp = end_time
        self.start_time = get_time_in_seconds(start_time)
        self.end_time = get_time_in_seconds(end_time)
        self.dialog = re.sub(r"\s\d+\s$|<(?:.)*?>", "", dialog).strip()
        self.duration = self.end_time - self.start_time
        self.dialog_pos = RiTa.get_pos_tags(self.dialog)  # GAP: no part-of-speech tagger (RiTa.getPosTags of the RiTa library); not an engine feature


# You have to download the video file first
# Please see https://github.com/generative-design/Code-Package-p5.js/tree/master/data
video_src = "data/video.mp4"
video = None
subtitle_src = "data/subs.vtt"
subtitles = []

# List space-seperated PENN tags (see https://rednoise.org/rita/reference/PennTags.html)
search_query = "nns jjr"  # text field

search_results = []
current_result = None

# time in milliseconds at which the current fragment ends, None if no timer is running
fragment_timer = None
fragment_index = 0

row = 0
col = 0
frame_width = 0
frame_height = 0

tile_mode = True  # checkbox

# position, size and visibility of the playing video (the video element of the original)
video_x = 0
video_y = 0
video_width = 0
video_height = 0
video_visible = False


def preload():
    global video
    video = load_movie(video_src)  # GAP: no video playback; load_movie(path) -> Movie is missing
    file = open(subtitle_src, encoding="utf-8")
    lines = file.read().split("\n")
    file.close()
    parse_subtitles(lines)


def parse_subtitles(lines):
    global subtitles
    subtitles = []
    timecode_reg_ex = re.compile(r"((\d{2}:){2}\d{2}(,|\.)\d{3})\s-->\s((\d{2}:){2}\d{2}(,|\.)\d{3})")
    start_time = None
    end_time = None
    dialog = None
    for i in range(len(lines)):
        line = lines[i]
        if timecode_reg_ex.search(line) or i == len(lines):
            if dialog:
                subtitles.append(SubTitleObject(start_time, end_time, dialog))

            if i < len(lines):
                start_time = re.sub(r"\s.+$", "", line)
                end_time = re.sub(r"^.+\s", "", line)
                dialog = ""
        else:
            if start_time and end_time:
                dialog += line + " "
    print(len(subtitles), "subtitles")


# the leading digits of a string as a number (JavaScript parseInt)
def parse_int(number_string):
    digits = ""
    for character in number_string:
        if character.isdigit():
            digits += character
        else:
            break
    return int(digits)


def get_time_in_seconds(time_string):
    hours = parse_int(re.sub(r":.+$", "", time_string, count=1))
    minutes = parse_int(re.sub(r"^\d.+?:|:\d.+$", "", time_string, count=1))
    seconds = parse_int(re.sub(r"^\d.+:|(\,|\.).+$", "", time_string, count=1))
    mil_seconds = parse_int(re.sub(r"^.+(\,|\.)", "", time_string, count=1))
    return (hours * 60 * 60) + (minutes * 60) + seconds + (mil_seconds / 1000)


def find_subtiles(search_pattern):
    search_pattern = search_pattern.split(" ")
    results = []
    for subtitle in subtitles:
        every = True
        for search_group in search_pattern:
            if search_group not in subtitle.dialog_pos:
                every = False
        if every:
            results.append(subtitle)
    return results


def generate_montage():
    global fragment_timer, row, col, search_results
    global video_x, video_y, video_width, video_height, video_visible
    fragment_timer = None
    video.stop()  # GAP: Movie.stop()
    if tile_mode:
        video_width = width * 0.25
    else:
        video_width = width
    video_height = video_width * video.height / video.width  # GAP: Movie.width / Movie.height
    video_x = 0
    video_y = 0
    video_visible = True
    background(0.0)
    row = 0
    col = 0

    search_results = find_subtiles(search_query)
    print("Found " + str(len(search_results)) + " results for search query " + search_query)

    if len(search_results) > 0:
        # the page of the original scrolls; a window taller than the screen is clamped to the screen
        window_resize(width, int(len(search_results) * video_height))  # GAP: a window_resize() taller than the screen leaves the canvas blank (white) from then on
        background(0.0)
        query_result_montage(search_results, 0)


def query_result_montage(search_results, i):
    global current_result, fragment_timer, fragment_index
    current_result = search_results[i]
    duration = current_result.duration

    video.play()  # GAP: Movie.play()
    video.jump(current_result.start_time)  # GAP: Movie.jump(seconds)

    print(current_result.start_time_stamp, current_result.dialog)
    fragment_index = i
    fragment_timer = millis() + duration * 1000


# runs when the fragment timer has run out
def fragment_finished():
    global fragment_timer, col, video_x, video_y
    i = fragment_index
    fragment_timer = None

    video.pause()  # GAP: Movie.pause()

    if tile_mode:
        frame_pos = get_frame_pos()
        img = video.get()  # GAP: Movie.get() (the current frame as an Image)
        image(img, frame_pos.x, frame_pos.y, frame_width, frame_height)

        draw_dialog(current_result.dialog, frame_pos.x, frame_pos.y, frame_width, frame_height)

        text(current_result.end_time_stamp, frame_pos.x, frame_pos.y)

        col += 1
        frame_pos = get_frame_pos()
        video_x = frame_pos.x
        video_y = frame_pos.y

    if i < len(search_results) - 1:
        query_result_montage(search_results, i + 1)
    else:
        fragment_timer = None


# the subtitle span of the original: white, centered at the bottom of the frame, with a black shadow
def draw_dialog(dialog, x, y, w, h):
    push_style()
    no_stroke()
    text_align(CENTER, BOTTOM)
    fill(0.0)
    text(dialog, x + 1, y + 1, max_w=w, max_h=h)
    fill(1.0)
    text(dialog, x, y, max_w=w, max_h=h)
    pop_style()


def toggle_playback():
    global fragment_timer
    fragment_timer = None
    if video.is_playing():  # GAP: Movie.is_playing()
        video.pause()  # GAP: Movie.pause()
    else:
        video.play()  # GAP: Movie.play()


def get_frame_pos():
    global frame_width, frame_height, col, row
    frame_width = video_width
    frame_height = video_height
    x = frame_width * col
    y = frame_height * row
    if x + frame_width > width:
        col = 0
        row += 1
        x = frame_width * col
        y = frame_height * row
    return Vec2(x, y)


def setup():
    size(1280, 1000)
    preload()

    fill(1.0)
    stroke(0.0)
    text_size(10)
    text_align(LEFT, TOP)

    generate_montage()


def draw():
    # show the playing video at its current position
    if video_visible:
        image(video, video_x, video_y, video_width, video_height)  # GAP: image() with a Movie source

    if fragment_timer != None and millis() >= fragment_timer:
        fragment_finished()


run()
