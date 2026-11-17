from processing_py import *
import colorsys

# shows incoming MIDI notes as colored bars (pitch -> x, velocity -> height),
# any control change sets background brightness.

PORT = 0 

held = {}  # note number = velocity
brightness = 0.05

def setup():
    size(640, 400)
    no_stroke()

    midi_refresh_ports()
    print("MIDI inputs:", midi_list_input_ports())
    print("listening to", midi_connect_input(PORT))

def draw():
    global brightness

    # everything received since the last frame
    for msg in midi_read():
        if msg.kind == "note_on":
            held[msg.note] = msg.velocity
        elif msg.kind == "note_off":
            held.pop(msg.note, None)
        elif msg.kind == "control_change":
            brightness = msg.value / 127 * 0.5
        else:
            print(msg)

    background(brightness, brightness, brightness + 0.03)

    bar_w = width / 128
    for note, velocity in held.items():
        h = velocity / 127 * height
        fill(*colorsys.hsv_to_rgb((note % 12) / 12, 0.7, 0.95))
        rect(note * bar_w, height - h, bar_w, h)

run()
