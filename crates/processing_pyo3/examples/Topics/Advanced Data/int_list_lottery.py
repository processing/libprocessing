# IntList Lottery example
# by Daniel Shiffman.
#
# This example demonstrates how a list can be used to store a list of numbers.
# A list can easily have values added or deleted and it can also be
# shuffled and sorted.
#
# In this example, three lists of integers are created.  One is a pool of numbers
# that is shuffled and picked randomly from.  One is the list of "picked" numbers.
# And one is a lottery "ticket" which includes 5 numbers that are trying to be matched.
from mewnala import *
from random import shuffle

# Three lists of integers
lottery = []
results = []
ticket = []


def setup():
    size(640, 360)
    frame_rate(30)

    # Add 20 integers in order to the lottery list
    for i in range(20):
        lottery.append(i)

    # Pick five numbers from the lottery list to go into the ticket list
    for i in range(5):
        index = int(random(len(lottery)))
        ticket.append(lottery[index])


def draw():
    background(0.2)

    # shuffle() randomly shuffles the order of the values in the list
    shuffle(lottery)

    # Call a function that will display the integers in the list at an x,y location
    show_list(lottery, 16, 48)
    show_list(results, 16, 100)
    show_list(ticket, 16, 140)

    # This loop checks if the picked numbers (results)
    # match the ticket numbers
    for i in range(len(results)):
        # Are the integers equal?
        if results[i] == ticket[i]:
            fill(0.0, 1.0, 0.0, 0.39)  # if so green
        else:
            fill(1.0, 0.0, 0.0, 0.39)  # if not red
        ellipse(16 + i * 32, 140, 24, 24)

    # One every 30 frames we pick a new lottery number to go in results
    if frame_count % 30 == 0:
        if len(results) < 5:
            # Get the first value in the lottery list and remove it
            val = lottery.pop(0)
            # Put it in the results
            results.append(val)
        else:
            # Ok we picked five numbers, let's reset
            for i in range(len(results)):
                # Put the picked results back into the lottery
                lottery.append(results[i])
            # Clear the results and start over
            results.clear()


# Draw a list of numbers starting at an x,y location
def show_list(lst, x, y):
    for i in range(len(lst)):
        # Pull a value from the list at the specified index
        val = lst[i]
        stroke(1.0)
        no_fill()
        ellipse(x + i * 32, y, 24, 24)
        text_align(CENTER)
        fill(1.0)
        text(str(val), x + i * 32, y + 6)


run()
