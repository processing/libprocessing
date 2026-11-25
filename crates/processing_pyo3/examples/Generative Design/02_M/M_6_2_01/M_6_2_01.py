# M_6_2_01
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

# loads the name of the links on the wikipedia-site "Superegg"
# and prints them to the console
#
# The original loads XML with loadXML(); here the same query is sent to the
# current MediaWiki API with urllib and the answer is read as JSON.
import json
import urllib.request

from processing_py import *

my_json = None
links = None
query = None


def setup():
    global my_json, links, query
    query = "https://en.wikipedia.org/w/api.php?titles=Superegg&format=json&formatversion=2&action=query&prop=links&pllimit=500"

    try:
        request = urllib.request.Request(query, headers={"User-Agent": "M_6_2_01 (Generative Design sketch)"})
        response = urllib.request.urlopen(request, timeout=10)
        my_json = json.loads(response.read())
        links = my_json["query"]["pages"][0]["links"]

        for i in range(len(links)):
            title = links[i]["title"]
            print("Link " + str(i) + ": " + title)
    except Exception as exception:
        print(exception)


run()
