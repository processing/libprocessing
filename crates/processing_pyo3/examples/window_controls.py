from mewnala import *

TITLES = ["window controls", "🪟 hello", "still alive"]
POSITIONS = [(100, 100), (300, 200), (50, 50)]
SIZES = [(640, 480), (800, 600), (320, 240)]
OPACITIES = [1.0, 0.85, 0.5]

state = {
    "title": 0,
    "position": 0,
    "size": 0,
    "opacity": 0,
    "resizable": True,
    "decorated": True,
    "on_top": False,
    "fullscreen": False,
    "show_at": -1,
}


def setup():
    size(640, 480)
    window_title(TITLES[0])


def log(label):
    print(f"[frame {frame_count}] {label}")


def cycle(key, label):
    state[key] = (state[key] + 1) % len(label)
    return state[key]


def draw():
    if 0 <= state["show_at"] <= frame_count:
        window_visible(True)
        state["show_at"] = -1
        log("re-shown")

    if key_just_pressed(KEY_T):
        idx = cycle("title", TITLES)
        window_title(TITLES[idx])
        log(f"title -> {TITLES[idx]!r}")

    if key_just_pressed(KEY_M):
        idx = cycle("position", POSITIONS)
        x, y = POSITIONS[idx]
        window_move(x, y)
        log(f"moved to ({x}, {y})")

    if key_just_pressed(KEY_W):
        idx = cycle("size", SIZES)
        w, h = SIZES[idx]
        window_resize(w, h)
        log(f"resized to {w}x{h}")

    if key_just_pressed(KEY_O):
        idx = cycle("opacity", OPACITIES)
        window_opacity(OPACITIES[idx])
        log(f"opacity -> {OPACITIES[idx]}")

    if key_just_pressed(KEY_R):
        state["resizable"] = not state["resizable"]
        window_resizable(state["resizable"])
        log(f"resizable -> {state['resizable']}")

    if key_just_pressed(KEY_D):
        state["decorated"] = not state["decorated"]
        window_decorated(state["decorated"])
        log(f"decorated -> {state['decorated']}")

    if key_just_pressed(KEY_A):
        state["on_top"] = not state["on_top"]
        window_always_on_top(state["on_top"])
        log(f"always-on-top -> {state['on_top']}")

    if key_just_pressed(KEY_V):
        window_visible(False)
        state["show_at"] = frame_count + 60
        log("hidden for ~1s")

    if key_just_pressed(KEY_I):
        window_iconify()
        log("iconified")

    if key_just_pressed(KEY_X):
        window_maximize()
        log("maximized")

    if key_just_pressed(KEY_N):
        window_restore()
        log("restored")

    if key_just_pressed(KEY_F):
        state["fullscreen"] = not state["fullscreen"]
        full_screen(primary_monitor() if state["fullscreen"] else None)
        log(f"fullscreen -> {state['fullscreen']}")

    if key_just_pressed(KEY_C):
        if (m := primary_monitor()) is not None:
            window_center_on(m)
            log(f"centered on {m.name!r}")

    if key_just_pressed(KEY_P):
        if (m := primary_monitor()) is not None:
            window_position_on(m, 10, 10)
            log(f"workarea +(10, 10) — workarea={m.workarea}")

    background(24)
    no_stroke()
    fill(80, 200, 200)
    rect(20, 20, width - 40, 40)
    fill(200, 80, 120)
    rect(20, 80, width - 40, 60)
    fill(120, 80, 200)
    rect(20, 160, width - 40, 60)
    fill(80, 200, 120)
    rect(20, 240, width - 40, 60)

    # Yellow dot tracks window_x / window_y wrapped into the canvas.
    fill(255, 220, 60)
    if width > 0 and height > 0:
        circle(window_x % width, window_y % height, 12)


run()
