from processing import *
import random

def setup():
    size(800, 600)

    # Refresh midi port list, print available ports, and connect to first one
    midi_refresh_ports()
    for port in midi_list_ports():
        print(port)
    midi_connect(0)

def draw():
    background(220)

    fill(255, 0, 100)
    stroke(1)
    stroke_weight(2)
    rect(100, 100, 200, 150)

    # pick a random note value, and duration value for that note
    # then send the midi command
    note = random.randint(57,68)
    note_duration = random.randint(25, 250)
    midi_play_notes(note, note_duration)

# TODO: this should happen implicitly on module load somehow
run()
