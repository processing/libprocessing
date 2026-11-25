from processing_py import *

# OSC out and in example
# OSC OUT: moving the mouse sends /mouse x y (0-1). Pressing and releasing the mouse sends /click 1 and 0.
# OSC IN: latest message is shown at the bottom.
#
# To try it with another program:
#   - receive on port 12000
#   - send to localhost (127.0.0.1) on port 10000

LISTEN_PORT = 10000
SEND_HOST = "127.0.0.1"   # this computer
SEND_PORT = 12000

latest_in = f"nothing received on port {LISTEN_PORT}"
latest_out = "nothing sent"
was_pressed = False


def setup():
    size(640, 480)
    no_stroke()
    port = osc_listen(LISTEN_PORT)
    print(f"listening on port {port}, sending to {SEND_HOST}:{SEND_PORT}")


def draw():
    global latest_in, latest_out, was_pressed

    # OSC IN
    for msg in osc_read():
        latest_in = f"{msg}  from {msg.host}:{msg.port}"

    # OSC OUT
    if mouse_x != pmouse_x or mouse_y != pmouse_y:
        sent = osc_send(SEND_HOST, SEND_PORT, "/mouse", mouse_x / width, mouse_y / height)
        latest_out = f"{sent}  to {sent.host}:{sent.port}"
    if mouse_is_pressed != was_pressed:
        sent = osc_send(SEND_HOST, SEND_PORT, "/click", 1 if mouse_is_pressed else 0)
        latest_out = f"{sent}  to {sent.host}:{sent.port}"
        was_pressed = mouse_is_pressed

    background(0)

    # OSC OUT visualizer
    stroke(0.5)
    if mouse_is_pressed:
        fill(1.0,1.0,0.0)
    else:
        no_fill()
    circle(mouse_x, mouse_y, 16)
    no_stroke()

    # OSC IN/OUT DISPLAY
    fill(1.0)
    text_size(14)
    text(f"OSC OUT: {SEND_HOST}:{SEND_PORT}", 12, 22)
    text(f"OSC IN: port {LISTEN_PORT}", 12, 42)
    text("OSC OUT: " + latest_out, 12, height - 22)
    text("OSC IN: " + latest_in, 12, height - 42)

run()