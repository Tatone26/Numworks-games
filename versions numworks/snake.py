from random import randint
from kandinsky import *
from ion import *
from time import *
from menu import menu

posFruit = [-1, -1]
points = 0
killerSizes = True
speed = 2
white = color(255,255,255)
record = 157
green = color(0, 120, 0)
l_green = color(40, 200, 120)

def drawCase(x, y):
  if (x%2==0 and y%2!=0) or (y%2==0 and x%2!=0):
    fill_rect(10*x, 10*y, 10, 10, (175, 175, 175))
  else:fill_rect(10*x, 10*y, 10, 10, (225, 225, 225))

class snake():
  def __init__(self):
    self.pos = [[3, 0],[2, 0], [1, 0], [0, 0]]
    self.dir = [1, 0]

  def drawSelf(self, t):
    for pos in self.pos[1:]:
      fill_rect(int(10*pos[0]+(t-1)*10), int(10*pos[1]+(t-1)*10), 10, 10, green)
    fill_rect(int(10*self.pos[0][0]+10*(t-1)), int(self.pos[0][1]*10+10*(t-1)), 10, 10, l_green)

  def checkPos(self):
    nex = [self.pos[0][0]+self.dir[0], self.pos[0][1]+self.dir[1]]
    if nex in self.pos:return False
    elif killerSizes and (not (0<=nex[0]<32) or not (0<=nex[1]<22)):return False
    return True

  def move(self):
    self.pos.insert(0, [self.pos[0][0]+self.dir[0], self.pos[0][1]+self.dir[1]])
    if not killerSizes:
      if self.pos[0][0]<0: self.pos[0][0]=31
      elif self.pos[0][0]>31: self.pos[0][0]=0
      if self.pos[0][1]<0: self.pos[0][1]=21
      elif self.pos[0][1]>21: self.pos[0][1]=0

  def checkKey(self):
    if keydown(KEY_UP) and self.pos[1][1] != self.pos[0][1]-1:
      self.dir = [0, -1]
    elif keydown(KEY_DOWN) and self.pos[1][1] != self.pos[0][1]+1:
      self.dir = [0, 1]
    elif keydown(KEY_RIGHT) and self.pos[1][0] != self.pos[0][0]+1:
      self.dir = [1, 0]
    elif keydown(KEY_LEFT)  and self.pos[1][0] != self.pos[0][0]-1:
      self.dir = [-1, 0]
    elif keydown(KEY_OK):
      draw_string("PAUSE", 120, 90)
      draw_string("Points : "+str(points), 95, 120)
      sleep(0.3)
      while not keydown(KEY_OK):pass
      for x in range(9, 22):
        for y in range(9, 16):drawCase(x, y)
      drawFruit()
      sleep(0.1)

  def act(self):
    global points
    frameStart = monotonic()
    while monotonic()<frameStart+0.70-speed*0.20 :self.checkKey()
    drawCase(self.pos[-1][0], self.pos[-1][1])
    if self.checkPos():
      self.move()
      if self.pos[0]==posFruit:
        points += 1
        placeFruit(self.pos)
      else : self.pos.pop()
      self.drawSelf(1)
      return True
    return False

def drawFruit():
  fill_rect(posFruit[0]*10, posFruit[1]*10, 10, 10, (255, 0, 0))

def placeFruit(bpos):
  global posFruit
  while True:
    posFruit = [randint(0, 31), randint(0, 21)]
    if posFruit not in bpos: break
  drawFruit()

def snk():
  for x in range(32):
    for y in range(22):drawCase(x, y)
  a = snake()
  placeFruit(a.pos)
  while a.act(): pass
  draw_string("Points : "+str(points), 95, 110)
  draw_string("REPLAY : <OK>", 10, 200)
  draw_string("MENU : <EXE>", 190, 200)
  draw_string("PERDU", 120, 80)
  while not (keydown(KEY_EXE) or keydown(KEY_OK)): pass
  if keydown(KEY_EXE): menu_s()
  else:snk()

def menu_s():
  global killerSizes, speed
  def addons():
    fill_rect(100, 90, 80, 10, green)
    fill_rect(180, 90, 10, 10, l_green)
    fill_rect(210, 90, 10, 10, (255, 0, 0))
    draw_string("Record : "+str(record), 190, 10)
  opt_list = [["Terrain continu", ("Oui", "Non"), killerSizes], ["Vitesse", ("Lent", "Moyen", "Rapide"), speed]]
  opt_modif = menu("SNAKE", addons, green, white, opt_list)
  killerSizes = opt_modif[0]
  speed = opt_modif[1]
  if opt_modif[-1]==True : snk()

menu_s()
#157