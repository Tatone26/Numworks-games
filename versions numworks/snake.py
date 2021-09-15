from random import randint
from kandinsky import *
from ion import *
from time import *
from menu import menu

killerSizes = True
obstacles = False
speed = 2
field = (32,22,1,0,0) #width, height, size (1, 2 ou 3), xoffset, yoffset

posFruit = [-1, -1]
points = 0
posWall = []

white = color(255,255,255)
green = color(0, 120, 0)
l_green = color(40, 200, 120)

def drawCase(x, y):
  if x>=field[0] or y>=field[1]:
    fill_rect(10*x+field[3], 10*y+field[4], 10, 10, white)
  elif (x%2==0 and y%2!=0) or (y%2==0 and x%2!=0):
    fill_rect(10*x+field[3], 10*y+field[4], 10, 10, (175, 175, 175))
  else:fill_rect(10*x+field[3], 10*y+field[4], 10, 10, (225, 225, 225))

class snake():
  def __init__(self):
    self.pos = [[4, 1],[3, 1], [2, 1], [1, 1]]
    self.dir = [1, 0]

  def drawSelf(self):
    for pos in self.pos[1:]:
      fill_rect(int(10*pos[0])+field[3], int(10*pos[1])+field[4], 10, 10, green)
    fill_rect(int(10*self.pos[0][0])+field[3], int(self.pos[0][1]*10)+field[4], 10, 10, l_green)

  def checkPos(self):
    nex = [self.pos[0][0]+self.dir[0], self.pos[0][1]+self.dir[1]]
    if nex in self.pos or nex in posWall:return False
    elif killerSizes and (not (0<=nex[0]<field[0]) or not (0<=nex[1]<field[1])):return False
    return True

  def move(self):
    self.pos.insert(0, [self.pos[0][0]+self.dir[0], self.pos[0][1]+self.dir[1]])
    if not killerSizes:
      if self.pos[0][0]<0: self.pos[0][0]=field[0]-1
      elif self.pos[0][0]>(field[0]-1): self.pos[0][0]=0
      if self.pos[0][1]<0: self.pos[0][1]=field[1]-1
      elif self.pos[0][1]>(field[1]-1): self.pos[0][1]=0

  def checkKey(self):
    if keydown(KEY_UP) and self.pos[1][1] != self.pos[0][1]-1 and not (not killerSizes and self.pos[0][1]==0 and self.pos[1][1]==field[1]-1):
      self.dir = [0, -1]
    elif keydown(KEY_DOWN) and self.pos[1][1] != self.pos[0][1]+1 and not (not killerSizes and self.pos[0][1]==field[1]-1 and self.pos[1][1]==0):
      self.dir = [0, 1]
    elif keydown(KEY_RIGHT) and self.pos[1][0] != self.pos[0][0]+1 and not (not killerSizes and self.pos[0][0]==field[0]-1 and self.pos[1][0]==0):
      self.dir = [1, 0]
    elif keydown(KEY_LEFT)  and self.pos[1][0] != self.pos[0][0]-1 and not (not killerSizes and self.pos[0][0]==0 and self.pos[1][0]==field[0]-1):
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
    if self.checkPos():
      self.move()
      if self.pos[0]==posFruit:
        points += 1
        if obstacles == True and points%2==0:
          placeWall(self.pos)
        placeFruit(self.pos)
      else : self.pos.pop()
      self.drawSelf()
      return True
    return False

def drawFruit():
  fill_rect(posFruit[0]*10+field[3], posFruit[1]*10+field[4], 10, 10, (255, 0, 0))
def placeFruit(bpos):
  global posFruit
  while True:
    posFruit = [randint(0, field[0]-1), randint(0, field[1]-1)]
    if posFruit not in bpos and posFruit not in posWall: break
  drawFruit()

def drawWall(pos):
  fill_rect(pos[0]*10+field[3], pos[1]*10+field[4], 10, 10, (20, 20, 20))
def placeWall(bpos):
  global posWall
  while True:
    newpos = [randint(0, field[0]-1), randint(0, field[1]-1)]
    bpostest = bpos.copy()
    bpostest += [[bpos[0][0]+1, bpos[0][1]], [bpos[0][0]-1, bpos[0][1]], [bpos[0][0], bpos[0][1]+1], [bpos[0][0], bpos[0][1]-1]]
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
  a = snake()
  placeFruit(a.pos)
  a.drawSelf()
  sleep(0.5)
  while a.act(): pass
  draw_string("Points : "+str(points), 95, 110)
  draw_string("REPLAY : <OK>", 10, 200)
  draw_string("MENU : <EXE>", 190, 200)
  draw_string("PERDU", 120, 80)
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
