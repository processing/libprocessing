# M_6_2_02
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

# loads an xml asynchronously, thus not disrupting the animation
#
# MOUSE
# left click          : starts loading the xml
#
# The original loads XML with GenerativeDesign.loadXMLAsync(); here the same
# query is sent to the current MediaWiki API with urllib in a background
# thread and the answer is read as JSON.
import json
import threading
import urllib.request

from processing_py import *


# loadXMLAsync of the generativedesign library: returns an empty dictionary
# at once and fills it in the background as soon as the answer has arrived
def load_json_async(url):
    result = {}
    thread = threading.Thread(target=load_json, args=(url, result))
    thread.daemon = True
    thread.start()
    return result


def load_json(url, result):
    try:
        request = urllib.request.Request(url, headers={"User-Agent": "M_6_2_02 (Generative Design sketch)"})
        response = urllib.request.urlopen(request, timeout=10)
        data = json.loads(response.read())
        result.update(data)
    except Exception as exception:
        print(exception)


my_json = None
links = None
query = None

angle = 0


def setup():
    size(200, 200)
    stroke_weight(4)


def draw():
    global angle, my_json, links
    # some animation
    background(1.0)
    translate(width / 2, height / 2)
    rotate(angle)
    angle += 0.02
    line(-100, 0, 100, 0)

    # test if json is already loaded
    if my_json != None:
        if len(my_json) == 0:
            print("not loaded yet")
        else:
            links = my_json["query"]["pages"][0]["links"]
            for i in range(len(links)):
                title = links[i]["title"]
                print("Link " + str(i) + ": " + title)
            my_json = None


def mouse_pressed():
    global query, my_json
    query = "https://en.wikipedia.org/w/api.php?titles=Superegg&format=json&formatversion=2&action=query&prop=links&pllimit=500"
    my_json = load_json_async(query)


run()
