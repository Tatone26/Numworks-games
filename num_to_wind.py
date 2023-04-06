import math
import random as rd
import time
from tkinter import *

import keyboard as kb


class Ntw(Tk):

    def __init__(self, debug):
        Tk.__init__(self)
        self.title("NUMWORK TO WINDOWS")
        self.geometry("320x220")
        self.resizable(False, False)
        self.canvas = Canvas(self, bg="white", height=220, width=320, bd=0, highlightthickness=0)
        self.canvas.pack(side=LEFT)
        self.bind("<KeyPress-Delete>", self.exit)
        self.debug = debug
        if debug:
            self.geometry("470x220")
            self.debug_text = StringVar(self, value="DEBUG")
            self.debug_label = Label(self, bg="light gray", height=220, width=140, anchor="ne",
                                     textvariable=self.debug_text)
            self.debug_label.pack(side=RIGHT)

    def my_update(self):
        self.update()
        if self.debug:  # writing the debug label. Not really useful for numworks programming right now.
            self.debug_text.set(
                "---DEBUG---\n-> canvas item nb : {0}\n-> key pressed : {1}".format(len(self.canvas.find_all()), None))

    def exit(self, event):
        self.destroy()

    def get_pixel(self, posx, posy):  # complicated because of tk
        list_touched = self.canvas.find_overlapping(posx, posy, posx, posy)
        precise_touched = max(list_touched)
        return tuple(int(self.canvas.itemcget(precise_touched, "fill").strip('#')[i:i + 2], 16) for i in (0, 2, 4))

    def fill_rect(self, posx, posy, width, height, color):
        self.canvas.create_rectangle(posx, posy, posx + width, posy + height,
                                     fill='#{:02x}{:02x}{:02x}'.format(color[0], color[1], color[2]), outline='',
                                     tags="rect")
        self.clean_canvas()
        self.my_update()

    def draw_string(self, string, posx, posy):
        string_list = string.split("\n")
        for i in range(len(string_list)):
            self.canvas.create_rectangle(posx - 2, posy - 1 + 20 * i, posx + len(string_list[0]) * 7 + 4, posy + 20,
                                         outline="white", fill="white", tags="rect")
        self.canvas.create_text(posx, posy, text=string, anchor="nw", font=("Arial", 12))
        self.my_update()

    def sleep(self, sec):  # have to put it there because of tkinter
        time.sleep(sec)
        self.my_update()

    def clean_canvas(self):  # clear the canvas of everything that is not showing
        listrect = self.canvas.find_withtag("rect")
        todelete = []
        for b in listrect:
            c = self.canvas.coords(b)
            enclosed = self.canvas.find_enclosed(c[0], c[1], c[2] + 1, c[3] + 1)
            for i in enclosed:
                if i in listrect and listrect.index(i) < listrect.index(b):
                    todelete.append(i)
        for i in todelete:
            self.canvas.delete(i)


ntw = Ntw(True)  # Launch main windows. Programs should call the functions below. True means with debug menu.


## KANDINSKY
'''
def fill_rect(posx, posy, width, height, color):
    ntw.fill_rect(posx, posy, width, height, color)


def set_pixel(x, y, color):
    ntw.fill_rect(x, y, 1, 1, color)


def draw_string(string, posx, posy):
    ntw.draw_string(string, posx, posy)


def get_pixel(posx, posy):
    return ntw.get_pixel(posx, posy)


def color(a, b, c):
    return (a, b, c)
'''

## RANDOM

def randint(a, b):
    return rd.randint(a, b)


def choice(seq):
    return rd.choice(seq)


def getrandbits(k):
    return rd.getrandbits(k)


def seed(k):
    return rd.seed(k)


def randrange(start, stop):
    return rd.randrange(start, stop)


def random():
    return rd.random()


def uniform(a, b):
    return rd.uniform(a, b)


## TIME

def monotonic():
    return time.time()


def sleep(sec):
    ntw.sleep(sec)


## MATH

def copysign(x, y):
    return math.copysign(x, y)


## ION

def keydown(key):
    if ntw.focus_displayof() != None and kb.is_pressed(key):
        return True
    else:
        return False


# ions keys translated to keyboard module... don't know if there are the good names though (no doc and no testing)
KEY_EXE = "space"
KEY_UP = "up"
KEY_DOWN = "down"
KEY_RIGHT = "right"
KEY_LEFT = "left"
KEY_OK = "return"
KEY_BACK = "*"
KEY_HOME = "windows"
KEY_ONOFF = "echap"
KEY_SHIFT = "shift"
KEY_ALPHA = "ctrl"
KEY_XNT = "n"
KEY_VAR = "F1"
KEY_TOOLBOX = "F2"
KEY_EXP = "e"
KEY_LN = "z"
KEY_LOG = "l"
KEY_IMAGINARY = "i"
KEY_COMMA = ","
KEY_POWER = "^"
KEY_SINE = "s"
KEY_COSINE = "c"
KEY_TANGENT = "t"
KEY_PI = "p"
KEY_SQRT = "_"
KEY_SQUARE = "²"
KEY_EIGHT = "eight"
KEY_SEVEN = "seven"
KEY_NINE = "nine"
KEY_ONE = "one"
KEY_TWO = "two"
KEY_THREE = "three"
KEY_RIGHTPARENTHESIS = ")"
KEY_LEFTPARENTHESIS = "("
KEY_FOUR = "four"
KEY_FIVE = "five"
KEY_SIX = "six"
KEY_MULTIPLICATION = "multiplication"
KEY_DIVISION = "division"
KEY_MINUS = "minus"
KEY_PLUS = "plus"
KEY_ZERO = "zero"
KEY_DOT = "dot"
KEY_EE = "!"
KEY_ANS = "a"
