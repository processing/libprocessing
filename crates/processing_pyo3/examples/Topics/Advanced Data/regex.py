# Regular Expression example
# by Daniel Shiffman.
#
# This example demonstrates how to use re.finditer() to create
# a list of all matches of a given regex.
#
# Here we'll load the raw HTML from a URL and search for any
# <a href=" "> links
from mewnala import *
import re
from urllib.request import Request, urlopen

# Our source url
url = "http://processing.org"
# We'll store the results in a list
links = []


def setup():
    global links
    size(640, 360)
    # Load the links
    links = load_links(url)


def draw():
    background(0.0)
    # Display the raw links
    fill(1.0)
    for i in range(len(links)):
        text(links[i], 10, 16 + i * 16)


def load_links(s):
    # Load the raw HTML as one big string (the site rejects the default Python user agent)
    request = Request(s, headers={"User-Agent": "Mozilla/5.0"})
    html = urlopen(request).read().decode("utf-8")

    # A wacky regex for matching a URL
    regex = "<\\s*a\\s+href\\s*=\\s*\"(.*?)\""
    # Each match knows its groups

    # A list for the results
    results = []

    # We want group 1 for each result
    for m in re.finditer(regex, html):
        results.append(m.group(1))

    # Return the results
    return results


run()
