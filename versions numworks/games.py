from kandinsky import *
from ion import *
from time import *
from snake import menu_s
from puissance4 import menu_p4
from pong import pong

white = color(255,255,255)
black = color(0,0,0)
listg = [["SNAKE",menu_s],["PUISSANCE 4", menu_p4],
["PONG",pong]]

def menug():
  fill_rect(0,0,340,250,white)
  draw_string("JEUX",132,10)
  for i in range(len(listg)):
    draw_string(listg[i][0],100,80+i*25)
  listg[ch_pos()][1]()
  menug()

def ch_pos():
  pos = 0
  draw_pos(pos)
  while not keydown(KEY_OK):
    if keydown(KEY_DOWN) and pos < len(listg)-1:
      clear_pos(pos)
      pos += 1
      draw_pos(pos)
      sleep(0.3)
    elif keydown(KEY_UP) and pos > 0:
      clear_pos(pos)
      pos -= 1
      draw_pos(pos)
      sleep(0.3)
  return pos

def draw_pos(x):
  fill_rect(100,95+x*25,100,3,black)
def clear_pos(x):
  fill_rect(100,95+x*25,100,3,white)

menug()
