from tkinter import *
import random as rd
import keyboard as kb
import time

class Ntw(Tk):

    def __init__(self):
        Tk.__init__(self)
        self.title("NUMWORK TO WINDOWS")
        self.geometry("320x220")
        self.resizable(False, False)
        self.canvas = Canvas(self, bg="white", height=220, width=320, bd=0, highlightthickness=0)
        self.canvas.pack()
        self.last_update = time.time()
        self.bind("<KeyPress-Delete>", self.exit)

    def exit(self, event):
        self.destroy()

    def get_pixel(self, posx, posy): #complicated because of tk
        list_touched = self.canvas.find_overlapping(posx, posy, posx, posy)
        precise_touched = max(list_touched)
        return tuple(int(self.canvas.itemcget(precise_touched, "fill").strip('#')[i:i+2], 16) for i in (0, 2, 4))

    def set_pixel(self, x, y, color):
        self.canvas.create_rectangle(x+1, y+1, x+2, y+2, fill='#{:02x}{:02x}{:02x}'.format(color[0], color[1], color[2]), outline='', tags="rect")
        self.clean_canvas()
        self.canvas.update()

    def fill_rect(self, posx, posy, width, height, color):
        self.canvas.create_rectangle(posx, posy, posx+width, posy+height, fill='#{:02x}{:02x}{:02x}'.format(color[0], color[1], color[2]), outline='', tags="rect")
        self.clean_canvas()
        self.canvas.update()

    def draw_string(self, string, posx, posy):
        self.canvas.create_rectangle(posx-2, posy-1, posx+len(string)*7+4, posy+20, outline="white", fill="white", tags="rect")
        self.canvas.create_text(posx, posy, text=string, anchor="nw", tags="rect")
        self.canvas.update()

    def sleep(self, sec):
        time.sleep(sec)
        self.update()

    def clean_canvas(self): # clean the canvas of everything that is not showing
        listrect = self.canvas.find_withtag("rect")
        todelete = []
        for b in listrect:
            c = self.canvas.coords(b)
            enclosed = self.canvas.find_enclosed(c[0], c[1], c[2]+1, c[3]+1)
            for i in enclosed:
                if i in listrect and listrect.index(i) < listrect.index(b):
                    todelete.append(i)
        for i in todelete:
            self.canvas.delete(i)


ntw = Ntw() #Launch main windows. Programs should call the functions below.

## KANDINSKY

def fill_rect(posx, posy, width, height, color):
    ntw.fill_rect(posx, posy, width, height, color)

def draw_string(string, posx, posy):
    ntw.draw_string(string, posx, posy)

def set_pixel(x,y,color):
    ntw.set_pixel(x, y, color)

def get_pixel(posx, posy):
    return ntw.get_pixel(posx, posy)

def color(a, b, c):
    return (a, b, c)

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

## ION

def keydown(key):
    if ntw.focus_displayof() != None and kb.is_pressed(key):
        return True
    else : return False

KEY_EXE = "space"
KEY_UP = "up"
KEY_DOWN = "down"
KEY_RIGHT = "right"
KEY_LEFT = "left"
KEY_OK = "return"
KEY_SHIFT = "shift"
KEY_SEVEN = "seven"
KEY_ONE = "one"
KEY_RIGHTPARENTHESIS = "nine"
KEY_MINUS = "three"