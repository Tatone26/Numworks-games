from random import randint
from kandinsky import *
from ion import *
from time import *
from math import copysign
from menu import menu

killerSizes = True
obstacles = False
speed = 2
field = (32,22,1,0,0) #wdh, hgt, opt, xoff, yoff

posFruit = [-1, -1]
points = 0
posWall = []

white = color(255,255,255)
green = (0, 120, 0)
l_green = (40, 200, 120)

def box(x, y, color):
  fill_rect(10*x+field[3], 10*y+field[4], 10, 10, color)
def drawCase(x, y):
  if x>=field[0] or y>=field[1]:
    box(x, y, white)
  elif (x%2==0 and y%2!=0) or (y%2==0 and x%2!=0):
    box(x, y, (175, 175, 175))
  else:box(x, y, (225, 225, 225))

class snake():
  def __init__(self):
    self.pos = [[4, 1],[3, 1], [2, 1], [1, 1]]
    self.dir = [1, 0]

  def drawSelf(self):
    for p in self.pos[1:]:
      box(p[0], p[1], green)
    box(self.pos[0][0], self.pos[0][1], l_green)

  def move(self):
    next = wrap([self.pos[0][0]+self.dir[0], self.pos[0][1]+self.dir[1]])
    if next in self.pos or next in posWall:
      return False
    elif killerSizes and (not (0<=next[0]<field[0]) or not (0<=next[1]<field[1])):
      return False
    else :
      self.pos.insert(0, next)
      return True

  def checkKey(self):

    def check(di):
      # False if impossible action
      if not self.pos[1][abs(di[1])] != self.pos[0][abs(di[1])]+sum(di):
        return False
      elif not killerSizes :
        a = int((sum(di)-1)/-2)
        b = abs(di[1])
        if (self.pos[a][b]==field[b]-1 and self.pos[abs(a-1)][b]==0) :
          return False
      return True

    if keydown(KEY_UP) and check([0, -1]):
      self.dir = [0, -1]
    elif keydown(KEY_DOWN) and check([0, 1]):
      self.dir = [0, 1]
    elif keydown(KEY_RIGHT) and check([1, 0]):
      self.dir = [1, 0]
    elif keydown(KEY_LEFT)  and check([-1, 0]):
      self.dir = [-1, 0]
    elif keydown(KEY_OK):
      draw_string("PAUSE", 130, 90)
      draw_string("Points : "+str(points), 100, 120)
      sleep(0.3)
      while not keydown(KEY_OK):pass
      for x in range(round(9-field[3]/10), round(22-field[3]/10)):
        for y in range(round(9-field[4]/10), round(14-field[4]/10)):drawCase(x, y)
      drawFruit()
      for w in posWall:
        drawWall(w)
      self.drawSelf()
      sleep(0.3)

  def act(self):
    global points
    frameStart = monotonic()
    while monotonic()<frameStart+0.70-speed*0.20 :self.checkKey()
    drawCase(self.pos[-1][0], self.pos[-1][1])
    if self.move():
      if self.pos[0]==posFruit:
        points += 1
        if obstacles == True and points%2==0:
          placeWall(self.pos, self.dir)
        placeFruit(self.pos)
      else : self.pos.pop()
      self.drawSelf()
      return True
    return False

def wrap(pos):
  # wrap pos around field if necessary
  if not killerSizes:
    for i in range(2):
      if pos[i] < 0 or pos[i] > field[i]-1 :
        pos[i] -= int(copysign(field[i], pos[i]))
  return pos

def drawFruit():
  box(posFruit[0], posFruit[1], (255, 0, 0))
def placeFruit(bpos):
  global posFruit
  while True:
    posFruit = [randint(0, field[0]-1), randint(0, field[1]-1)]
    if posFruit not in bpos and posFruit not in posWall: break
  drawFruit()

def drawWall(p):
  box(p[0], p[1], (20, 20, 20))
def placeWall(bpos, dir):
  global posWall
  while True:
    newpos = [randint(0, field[0]-1), randint(0, field[1]-1)]
    bpostest = bpos.copy() + [[bpos[0][0]+dir[0], bpos[0][1]+dir[1]]]
    if (newpos not in bpostest) and (newpos not in posWall):
      posWall.append(newpos)
      break
  drawWall(newpos)

def snk():
  global points, posWall
  points=0
  posWall=[]
  color = (0, 0, 0)
  if not killerSizes : color = (150, 150, 150)
  fill_rect(field[3]-5, field[4]-5, 10*field[0]+10, 10*field[1]+10, color)
  for x in range(field[0]):
    for y in range(field[1]):drawCase(x, y)
  s = snake()
  placeFruit(s.pos)
  s.drawSelf()
  sleep(0.5)
  while s.act(): pass
  draw_string("Points : "+str(points), 100, 120)
  draw_string("REPLAY : <OK>", 10, 200)
  draw_string("MENU : <EXE>", 190, 200)
  draw_string("PERDU", 130, 90)
  while not (keydown(KEY_EXE) or keydown(KEY_OK)): pass
  if keydown(KEY_EXE): menu_s()
  else:
    fill_rect(0, 0, 320, 240, white)
    snk()

def menu_s():
  global killerSizes, speed, obstacles, field
  def addons():
    fill_rect(100, 90, 80, 10, green)
    fill_rect(180, 90, 10, 10, l_green)
    fill_rect(210, 90, 10, 10, (255, 0, 0))
  opt_list = [["Terrain continu", ("Oui", "Non"), killerSizes],
   ["Vitesse", ("Lent", "Moyen", "Rapide"), speed],
   ["Obstacles",("Non","Oui"),obstacles],
   ["Terrain",("Entier","Demi", "2/3"),field[2]]]
  opt_modif = menu("SNAKE", addons, green, white, opt_list)
  killerSizes = opt_modif[0]
  speed = opt_modif[1]
  obstacles = opt_modif[2]
  def calc_side(opt, size):
    if opt != 1:
      return round(size*((opt-1)/opt))
    else : return opt*size
  field = [calc_side(opt_modif[3], 32), calc_side(opt_modif[3], 22), opt_modif[3]]
  field = tuple(field + [160-10*round(field[0]/2), 110-10*round(field[1]/2)])
  if opt_modif[-1]==True : snk()

menu_s()