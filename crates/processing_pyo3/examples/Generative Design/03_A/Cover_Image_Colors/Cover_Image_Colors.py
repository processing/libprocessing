# Cover_Image_Colors
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

# Loads images and searches for the most saturated color. It tries to avoid
# the special green color that we've chosen for the book. Otherwise most of
# the colors would be that green.
# Returns an array with these colors and prints them to the console.
from processing_py import *


# Basic port of the MMCQ (modified median cut quantization) algorithm from
# quantize.js, Copyright 2008 Nick Rabinowitz (MIT license), which the p5.js
# original loads as a library.

sigbits = 5
rshift = 8 - sigbits
max_iterations = 1000
fract_by_populations = 0.75


# get reduced-space color index for a pixel
def get_color_index(r, g, b):
    return (r << (2 * sigbits)) + (g << sigbits) + b


# Simple priority queue
class PQueue:
    def __init__(self, priority):
        self.contents = []
        self.is_sorted = False
        self.priority = priority

    def sort(self):
        self.contents.sort(key=self.priority)
        self.is_sorted = True

    def push(self, o):
        self.contents.append(o)
        self.is_sorted = False

    def peek(self, index=None):
        if not self.is_sorted:
            self.sort()
        if index == None:
            index = len(self.contents) - 1
        return self.contents[index]

    def pop(self):
        if not self.is_sorted:
            self.sort()
        return self.contents.pop()

    def size(self):
        return len(self.contents)


# 3d color space box
class VBox:
    def __init__(self, r1, r2, g1, g2, b1, b2, histo):
        self.r1 = r1
        self.r2 = r2
        self.g1 = g1
        self.g2 = g2
        self.b1 = b1
        self.b2 = b2
        self.histo = histo
        self.cached_volume = 0
        self.cached_count = 0
        self.count_set = False
        self.cached_avg = None

    def volume(self, force=False):
        if not self.cached_volume or force:
            self.cached_volume = (self.r2 - self.r1 + 1) * (self.g2 - self.g1 + 1) * (self.b2 - self.b1 + 1)
        return self.cached_volume

    def count(self, force=False):
        histo = self.histo
        if not self.count_set or force:
            npix = 0
            for i in range(self.r1, self.r2 + 1):
                for j in range(self.g1, self.g2 + 1):
                    for k in range(self.b1, self.b2 + 1):
                        index = get_color_index(i, j, k)
                        npix += histo[index]
            self.cached_count = npix
            self.count_set = True
        return self.cached_count

    def copy(self):
        return VBox(self.r1, self.r2, self.g1, self.g2, self.b1, self.b2, self.histo)

    def avg(self, force=False):
        histo = self.histo
        if not self.cached_avg or force:
            ntot = 0
            mult = 1 << (8 - sigbits)
            rsum = 0
            gsum = 0
            bsum = 0
            for i in range(self.r1, self.r2 + 1):
                for j in range(self.g1, self.g2 + 1):
                    for k in range(self.b1, self.b2 + 1):
                        histoindex = get_color_index(i, j, k)
                        hval = histo[histoindex]
                        ntot += hval
                        rsum += hval * (i + 0.5) * mult
                        gsum += hval * (j + 0.5) * mult
                        bsum += hval * (k + 0.5) * mult
            if ntot:
                self.cached_avg = [int(rsum / ntot), int(gsum / ntot), int(bsum / ntot)]
            else:
                # print("empty box")
                self.cached_avg = [int(mult * (self.r1 + self.r2 + 1) / 2), int(mult * (self.g1 + self.g2 + 1) / 2), int(mult * (self.b1 + self.b2 + 1) / 2)]
        return self.cached_avg


# Color map
class CMap:
    def __init__(self):
        self.vboxes = PQueue(cmap_priority)

    def push(self, vbox):
        self.vboxes.push({"vbox": vbox, "color": vbox.avg()})

    def palette(self):
        colors = []
        for vb in self.vboxes.contents:
            colors.append(vb["color"])
        return colors

    def size(self):
        return self.vboxes.size()


def cmap_priority(a):
    return a["vbox"].count() * a["vbox"].volume()


def count_priority(a):
    return a.count()


def count_volume_priority(a):
    return a.count() * a.volume()


# histo (1-d array, giving the number of pixels in
# each quantized region of color space)
def get_histo(pixels):
    histosize = 1 << (3 * sigbits)
    histo = [0] * histosize
    for pixel in pixels:
        rval = pixel[0] >> rshift
        gval = pixel[1] >> rshift
        bval = pixel[2] >> rshift
        index = get_color_index(rval, gval, bval)
        histo[index] = histo[index] + 1
    return histo


def vbox_from_pixels(pixels, histo):
    rmin = 1000000
    rmax = 0
    gmin = 1000000
    gmax = 0
    bmin = 1000000
    bmax = 0
    # find min/max
    for pixel in pixels:
        rval = pixel[0] >> rshift
        gval = pixel[1] >> rshift
        bval = pixel[2] >> rshift
        if rval < rmin:
            rmin = rval
        elif rval > rmax:
            rmax = rval
        if gval < gmin:
            gmin = gval
        elif gval > gmax:
            gmax = gval
        if bval < bmin:
            bmin = bval
        elif bval > bmax:
            bmax = bval
    return VBox(rmin, rmax, gmin, gmax, bmin, bmax, histo)


def median_cut_apply(histo, vbox):
    if not vbox.count():
        return None

    rw = vbox.r2 - vbox.r1 + 1
    gw = vbox.g2 - vbox.g1 + 1
    bw = vbox.b2 - vbox.b1 + 1
    maxw = max(rw, gw, bw)
    # only one pixel, no split
    if vbox.count() == 1:
        return [vbox.copy(), None]
    # Find the partial sum arrays along the selected axis.
    total = 0
    partialsum = {}
    lookaheadsum = {}
    if maxw == rw:
        for i in range(vbox.r1, vbox.r2 + 1):
            sum = 0
            for j in range(vbox.g1, vbox.g2 + 1):
                for k in range(vbox.b1, vbox.b2 + 1):
                    index = get_color_index(i, j, k)
                    sum += histo[index]
            total += sum
            partialsum[i] = total
    elif maxw == gw:
        for i in range(vbox.g1, vbox.g2 + 1):
            sum = 0
            for j in range(vbox.r1, vbox.r2 + 1):
                for k in range(vbox.b1, vbox.b2 + 1):
                    index = get_color_index(j, i, k)
                    sum += histo[index]
            total += sum
            partialsum[i] = total
    else:  # maxw == bw
        for i in range(vbox.b1, vbox.b2 + 1):
            sum = 0
            for j in range(vbox.r1, vbox.r2 + 1):
                for k in range(vbox.g1, vbox.g2 + 1):
                    index = get_color_index(j, k, i)
                    sum += histo[index]
            total += sum
            partialsum[i] = total
    for i in partialsum:
        lookaheadsum[i] = total - partialsum[i]

    # determine the cut planes
    if maxw == rw:
        return do_cut(vbox, vbox.r1, vbox.r2, "r", partialsum, lookaheadsum, total)
    elif maxw == gw:
        return do_cut(vbox, vbox.g1, vbox.g2, "g", partialsum, lookaheadsum, total)
    else:
        return do_cut(vbox, vbox.b1, vbox.b2, "b", partialsum, lookaheadsum, total)


def do_cut(vbox, dim1, dim2, color_name, partialsum, lookaheadsum, total):
    for i in range(dim1, dim2 + 1):
        if partialsum[i] > total / 2:
            vbox1 = vbox.copy()
            vbox2 = vbox.copy()
            left = i - dim1
            right = dim2 - i
            if left <= right:
                d2 = min(dim2 - 1, int(i + right / 2))
            else:
                d2 = max(dim1, int(i - 1 - left / 2))
            # avoid 0-count boxes
            while not partialsum.get(d2):
                d2 += 1
            count2 = lookaheadsum.get(d2)
            while not count2 and partialsum.get(d2 - 1):
                d2 -= 1
                count2 = lookaheadsum.get(d2)
            # set dimensions
            if color_name == "r":
                vbox1.r2 = d2
                vbox2.r1 = vbox1.r2 + 1
            elif color_name == "g":
                vbox1.g2 = d2
                vbox2.g1 = vbox1.g2 + 1
            else:
                vbox1.b2 = d2
                vbox2.b1 = vbox1.b2 + 1
            # print("vbox counts:", vbox.count(), vbox1.count(), vbox2.count())
            return [vbox1, vbox2]


# inner function to do the iteration
def quantize_iter(histo, lh, target):
    ncolors = 1
    niters = 0
    while niters < max_iterations:
        vbox = lh.pop()
        if not vbox.count():  # just put it back
            lh.push(vbox)
            niters += 1
            continue
        # do the cut
        vboxes = median_cut_apply(histo, vbox)
        vbox1 = vboxes[0]
        vbox2 = vboxes[1]

        if not vbox1:
            # print("vbox1 not defined; shouldn't happen!")
            return
        lh.push(vbox1)
        if vbox2:  # vbox2 can be null
            lh.push(vbox2)
            ncolors += 1
        if ncolors >= target:
            return
        niters += 1
        if niters - 1 > max_iterations:
            # print("infinite loop; perhaps too few pixels!")
            return


def quantize(pixels, maxcolors):
    # short-circuit
    if not len(pixels) or maxcolors < 2 or maxcolors > 256:
        # print("wrong number of maxcolors")
        return False

    histo = get_histo(pixels)

    # get the beginning vbox from the colors
    vbox = vbox_from_pixels(pixels, histo)
    pq = PQueue(count_priority)
    pq.push(vbox)

    # first set of colors, sorted by population
    quantize_iter(histo, pq, fract_by_populations * maxcolors)

    # Re-sort by the product of pixel occupancy times the size in color space.
    pq2 = PQueue(count_volume_priority)
    while pq.size():
        pq2.push(pq.pop())

    # next set - generate the median cuts using the (npix * vol) sorting.
    quantize_iter(histo, pq2, maxcolors - pq2.size())

    # calculate the actual colors
    cmap = CMap()
    while pq2.size():
        cmap.push(pq2.pop())

    return cmap


image_files = []
image_samples = []
image_colors = []

possible_images_count = 256
primary_green = None


def preload():
    for i in range(1, possible_images_count + 1):
        image_files.append(load_image("data/PNG150/Seite_" + str(i).zfill(3) + ".png"))


def setup():
    global primary_green
    size(512, 512)
    preload()
    no_stroke()

    primary_green = color(14 / 255, 232 / 255, 117 / 255)

    # create image samples
    n = 1
    for i in range(len(image_files)):
        for j in range(n):
            image_samples.append(ImageSample(image_files[i], i, j / n, 1 / n))

    for i in range(len(image_samples)):
        c = image_samples[i].sample_color
        image_colors.append({"r": int(red(c) * 255), "g": int(green(c) * 255), "b": int(blue(c) * 255), "a": int(alpha(c) * 255)})

    print(image_colors)


def draw():
    w = width / len(image_colors)
    for index in range(len(image_colors)):
        image_color = image_colors[index]
        fill(image_color["r"] / 255, image_color["g"] / 255, image_color["b"] / 255)
        rect(index * w, 0, w, height)


class ImageSample:
    def __init__(self, img, index, y, h):
        self.index = index
        self.img = img.get(0, int(img.height * y), img.width, int(img.height * h))  # GAP: Image.get(x, y, w, h) returning a sub-image is missing
        self.img.load_pixels()
        color_count = 100
        quality = 1
        sampled_pixels = []
        for i in range(0, len(self.img.pixels), quality):
            r = int(red(self.img.pixels[i]) * 255)
            g = int(green(self.img.pixels[i]) * 255)
            b = int(blue(self.img.pixels[i]) * 255)
            a = int(alpha(self.img.pixels[i]) * 255)
            if a >= 125:
                if not (r > 250 and g > 250 and 250):
                    sampled_pixels.append([r, g, b])

        self.cmap = quantize(sampled_pixels, color_count)

        if self.cmap:
            sat_color = get_most_saturated_color(self.cmap.palette())
            sat_color_not_green = get_most_saturated_color_not_green(self.cmap.palette())
        else:
            sat_color = color(0.0)
            sat_color_not_green = color(0.0)
        sat_value = saturation_100(sat_color_not_green)
        if sat_value > 50:
            best_color = sat_color_not_green
        else:
            best_color = sat_color

        if self.cmap:
            self.sample_color = white_to_alpha(best_color)
        else:
            self.sample_color = color(1.0)


# p5's saturation() outside of HSB mode: the HSL saturation, 0-100
def saturation_100(c):
    hsla = c.to_hsla().to_list()
    return hsla[1] * 100


# p5's brightness(): the HSB brightness, 0-100
def brightness_100(c):
    return brightness(c) * 100


# searches for the most saturated color in an array of colors
def get_most_saturated_color(arr):
    max_sat = 0
    c = color(0.0)
    for i in range(len(arr)):
        cc = color(arr[i][0] / 255, arr[i][1] / 255, arr[i][2] / 255)
        sat = saturation_100(cc) + brightness_100(cc)
        if sat > max_sat:
            c = cc
            max_sat = sat
    return c


# searches for the most saturated color in an array of colors avoiding green
def get_most_saturated_color_not_green(arr):
    max_sat = 0
    c = color(0.0)
    for i in range(len(arr)):
        cc = color(arr[i][0] / 255, arr[i][1] / 255, arr[i][2] / 255)
        if abs(hue(cc) - hue(primary_green)) < 30:
            # print("similar")
            continue
        sat = saturation_100(cc) + brightness_100(cc)
        if sat > max_sat:
            c = cc
            max_sat = sat
    return c


# calculates an alpha value depending on how close to white the color is.
def white_to_alpha(col):
    r = red(col) * 255
    g = green(col) * 255
    b = blue(col) * 255

    m = min(r, g, b)
    if m > 0:
        f = m / 255

        r = (r - 255 * f) / (1 - f)
        g = (g - 255 * f) / (1 - f)
        b = (b - 255 * f) / (1 - f)
        a = (1 - f) * 255

        return color(r / 255, g / 255, b / 255, a / 255)
    return col


run()
