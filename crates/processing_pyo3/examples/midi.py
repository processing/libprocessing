from processing_py import *
import colorsys

# MIDI out and in example
# MIDI OUT: click to play a note, showing up as colorful ripples
# MIDI IN: incoming MIDI notes show up as colorful bars. control change (CC) changes brightness of the bar.

LOW_NOTE = 48  # C3
NOTES = 36     # notes range 3 octaves
PLAY_H = 250   

playing = None  
ripples = []    
held_in = {}    # incoming notes: note number -> velocity
cc = 0.0        # last control change value, 0-1
out_port = "" 
in_port = ""


def note_color(note):
    return colorsys.hsv_to_rgb(note % 12 / 12, 0.7, 0.95)


def setup():
    global out_port, in_port
    size(640, 480)
    no_stroke()

    midi_refresh_ports()
    outputs = midi_list_ports()
    print("MIDI outputs:", outputs)
    print("MIDI inputs:", midi_list_input_ports())
    midi_connect(0)
    out_port = outputs[0].split(": ", 1)[1] 
    in_port = midi_connect_input(0)
    print("sending to", out_port)
    print("listening to", in_port)


def draw():
    global playing, cc
    key_w = width / NOTES

    # MIDI OUT
    if mouse_is_pressed and playing is None and mouse_y < PLAY_H:
        playing = LOW_NOTE + min(int(mouse_x / key_w), NOTES - 1)
        midi_note_on(playing, 100)
        ripples.append([mouse_x, mouse_y, 0.0, note_color(playing)])
    elif not mouse_is_pressed and playing is not None:
        midi_note_off(playing)
        playing = None

    # MIDI IN
    for msg in midi_read():
        if msg.kind == "note_on":
            held_in[msg.note] = msg.velocity
        elif msg.kind == "note_off":
            held_in.pop(msg.note, None)
        elif msg.kind == "control_change":
            cc = msg.value / 127

    background(0)

    # MIDI OUT dot visualizer
    for r in ripples:
        r[2] += 0.02
        x, y, age, color = r
        fill(*color, (1 - age) ** 2)
        circle(x, y, 24 + age * 220)
    ripples[:] = [r for r in ripples if r[2] < 1]

    # MIDI IN bar visualizer
    fill(cc * 0.4, cc * 0.4, cc * 0.4)
    rect(0, PLAY_H, width, height - PLAY_H)
    fill(0.3, 0.3, 0.3)
    rect(0, PLAY_H, width, 1)
    for n, velocity in held_in.items():
        h = velocity / 127 * (height - PLAY_H - 36)
        fill(*note_color(n))
        rect((n - LOW_NOTE) * key_w, height - h, key_w - 2, h)

    # PORTS DISPLAY
    fill(1.0)
    text_size(14)
    text(f"MIDI OUT: {out_port} (click to play)", 12, 22)
    text(f"MIDI IN: {in_port}", 12, PLAY_H + 22)


run()
