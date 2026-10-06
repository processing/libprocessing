# Penrose Snowflake L-System
# by Geraldine Sarmiento.
#
# This example was based on Patrick Dwyer's L-System class.
from mewnala import *

ps = None


def setup():
    global ps
    size(640, 360)
    stroke(1.0)
    no_fill()
    ps = PenroseSnowflakeLSystem()
    ps.simulate(4)


def draw():
    background(0.0)
    ps.render()


class LSystem:
    def __init__(self):
        self.steps = 0
        self.axiom = "F"
        self.rule = "F+F-F"
        self.production = ""
        self.start_length = 90.0
        self.draw_length = 0.0
        self.theta = radians(120.0)
        self.generations = 0
        self.reset()

    def reset(self):
        self.production = self.axiom
        self.draw_length = self.start_length
        self.generations = 0

    def get_age(self):
        return self.generations

    def render(self):
        translate(width / 2, height / 2)
        self.steps += 5
        if self.steps > len(self.production):
            self.steps = len(self.production)
        for i in range(self.steps):
            step = self.production[i]
            if step == "F":
                rect(0, 0, -self.draw_length, -self.draw_length)
                no_fill()
                translate(0, -self.draw_length)
            elif step == "+":
                rotate(self.theta)
            elif step == "-":
                rotate(-self.theta)
            elif step == "[":
                push_matrix()
            elif step == "]":
                pop_matrix()

    def simulate(self, gen):
        while self.get_age() < gen:
            self.production = self.iterate(self.production, self.rule)

    def iterate(self, prod_, rule_):
        self.draw_length = self.draw_length * 0.6
        self.generations += 1
        new_production = prod_
        new_production = new_production.replace("F", rule_)
        return new_production


class PenroseSnowflakeLSystem(LSystem):
    def __init__(self):
        super().__init__()
        self.axiom = "F3-F3-F3-F3-F"
        self.rule_f = "F3-F3-F45-F++F3-F"
        self.start_length = 450.0
        self.theta = radians(18)
        self.reset()

    def use_rule(self, r_):
        self.rule = r_

    def use_axiom(self, a_):
        self.axiom = a_

    def use_length(self, l_):
        self.start_length = l_

    def use_theta(self, t_):
        self.theta = radians(t_)

    def reset(self):
        self.production = self.axiom
        self.draw_length = self.start_length
        self.generations = 0

    def get_age(self):
        return self.generations

    def render(self):
        translate(width, height)
        repeats = 1

        self.steps += 3
        if self.steps > len(self.production):
            self.steps = len(self.production)

        for i in range(self.steps):
            step = self.production[i]
            if step == "F":
                for j in range(repeats):
                    line(0, 0, 0, -self.draw_length)
                    translate(0, -self.draw_length)
                repeats = 1
            elif step == "+":
                for j in range(repeats):
                    rotate(self.theta)
                repeats = 1
            elif step == "-":
                for j in range(repeats):
                    rotate(-self.theta)
                repeats = 1
            elif step == "[":
                push_matrix()
            elif step == "]":
                pop_matrix()
            elif "0" <= step <= "9":
                repeats += ord(step) - ord("0")

    def iterate(self, prod_, rule_):
        new_production = ""
        for i in range(len(prod_)):
            step = self.production[i]
            if step == "F":
                new_production = new_production + self.rule_f
            else:
                if step != "F":
                    new_production = new_production + step
        self.draw_length = self.draw_length * 0.4
        self.generations += 1
        return new_production


run()
