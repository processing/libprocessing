# Datatype Conversion.
#
# It is sometimes beneficial to convert a value from one type of
# data to another. Each of the conversion functions converts its parameter
# to an equivalent representation within its datatype.
# The conversion functions include int(), float(), str(), ord(), chr(), and others.
from mewnala import *

size(640, 360)
background(0.0)
no_stroke()

text_font(load_font("data/SourceCodePro-Regular.ttf"))
text_size(24)

# Strings are used for storing alphanumeric symbols
# Floats are decimal numbers
# Integers are whole numbers

c = "A"
f = float(ord(c))  # Sets f = 65.0
i = int(f * 1.4)  # Sets i to 91
b = ord(c) // 2  # Sets b to 32

# print(f)
# print(i)
# print(b)

text("The value of variable c is " + c, 50, 100)
text("The value of variable f is " + str(f), 50, 150)
text("The value of variable i is " + str(i), 50, 200)
text("The value of variable b is " + str(b), 50, 250)

run()
