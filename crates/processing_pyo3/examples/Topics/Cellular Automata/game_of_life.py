# Game of Life
# by Joan Soler-Adillon.
#
# Press SPACE BAR to pause and change the cell's values
# with the mouse. On pause, click to activate/deactivate
# cells. Press 'R' to randomly reset the cells' grid.
# Press 'C' to clear the cells' grid. The original Game
# of Life was created by John Conway in 1970.
from mewnala import *

# Size of cells
cell_size = 5

# How likely for a cell to be alive at start (in percentage)
probability_of_alive_at_start = 15

# Variables for timer
interval = 100
last_recorded_time = 0

# Colors for active/inactive cells
alive = color(0.0, 0.78, 0.0)
dead = color(0.0)

# Array of cells
cells = []
# Buffer to record the state of the cells and use this
# while changing the others in the interations
cells_buffer = []

# Pause
pause = False


def setup():
    size(640, 360)

    # Instantiate arrays
    for x in range(width // cell_size):
        cells.append([0] * (height // cell_size))
        cells_buffer.append([0] * (height // cell_size))

    # This stroke will draw the background grid
    stroke(0.19)

    # Initialization of cells
    for x in range(width // cell_size):
        for y in range(height // cell_size):
            state = random(100)
            if state > probability_of_alive_at_start:
                state = 0
            else:
                state = 1
            cells[x][y] = int(state)  # Save state of each cell
    # Fill in black in case cells don't cover all the windows
    background(0.0)


def draw():
    global last_recorded_time

    # Draw grid
    for x in range(width // cell_size):
        for y in range(height // cell_size):
            if cells[x][y] == 1:
                fill(alive)  # If alive
            else:
                fill(dead)  # If dead
            rect(x * cell_size, y * cell_size, cell_size, cell_size)
    # Iterate if timer ticks
    if millis() - last_recorded_time > interval:
        if not pause:
            iteration()
            last_recorded_time = millis()

    # Create new cells manually on pause
    if pause and mouse_is_pressed:
        # Map and avoid out of bound errors
        x_cell_over = int(remap(mouse_x, 0, width, 0, width // cell_size))
        x_cell_over = constrain(x_cell_over, 0, width // cell_size - 1)
        y_cell_over = int(remap(mouse_y, 0, height, 0, height // cell_size))
        y_cell_over = constrain(y_cell_over, 0, height // cell_size - 1)

        # Check against cells in buffer
        if cells_buffer[x_cell_over][y_cell_over] == 1:  # Cell is alive
            cells[x_cell_over][y_cell_over] = 0  # Kill
            fill(dead)  # Fill with kill color
        else:  # Cell is dead
            cells[x_cell_over][y_cell_over] = 1  # Make alive
            fill(alive)  # Fill alive color
    elif pause and not mouse_is_pressed:  # And then save to buffer once mouse goes up
        # Save cells to buffer (so we opeate with one array keeping the other intact)
        for x in range(width // cell_size):
            for y in range(height // cell_size):
                cells_buffer[x][y] = cells[x][y]


def iteration():  # When the clock ticks
    # Save cells to buffer (so we opeate with one array keeping the other intact)
    for x in range(width // cell_size):
        for y in range(height // cell_size):
            cells_buffer[x][y] = cells[x][y]

    # Visit each cell:
    for x in range(width // cell_size):
        for y in range(height // cell_size):
            # And visit all the neighbours of each cell
            neighbours = 0  # We'll count the neighbours
            for xx in range(x - 1, x + 2):
                for yy in range(y - 1, y + 2):
                    if 0 <= xx < width // cell_size and 0 <= yy < height // cell_size:  # Make sure you are not out of bounds
                        if not (xx == x and yy == y):  # Make sure to to check against self
                            if cells_buffer[xx][yy] == 1:
                                neighbours += 1  # Check alive neighbours and count them
            # We've checked the neigbours: apply rules!
            if cells_buffer[x][y] == 1:  # The cell is alive: kill it if necessary
                if neighbours < 2 or neighbours > 3:
                    cells[x][y] = 0  # Die unless it has 2 or 3 neighbours
            else:  # The cell is dead: make it live if necessary
                if neighbours == 3:
                    cells[x][y] = 1  # Only if it has 3 neighbours


def key_pressed():
    global pause
    if key == "r" or key == "R":
        # Restart: reinitialization of cells
        for x in range(width // cell_size):
            for y in range(height // cell_size):
                state = random(100)
                if state > probability_of_alive_at_start:
                    state = 0
                else:
                    state = 1
                cells[x][y] = int(state)  # Save state of each cell
    if key == " ":  # On/off of pause
        pause = not pause
    if key == "c" or key == "C":  # Clear all
        for x in range(width // cell_size):
            for y in range(height // cell_size):
                cells[x][y] = 0  # Save all to zero


run()
