from num_to_wind import *
from menu import menu

white = color(255,255,255)
black = color(0, 0, 0)
darkMode = False

class Boat:

  def __init__(self, size, posx, posy, orient):
    self.boat = [0]*size
    self.pos = []
    self.orient = orient
    self.size = size
    if(self.orient == "h"):
      for p in range(posx, posx+size):
        self.pos.append([p, posy])
    else :
      for p in range(posy, posy+size):
        self.pos.append([posx, p])

  def check(self):
    return(not 0 in self.boat)

class Player:

  def __init__(self):
    self.listBoat = []
    self.listTirs = []
    self.lastSelect = [0,0]

  def choosePos(self, griPosX, griPosY, boatSize, orient):
    select = replaceBoatSelect(boatSize, orient, self.lastSelect)

    def printCho():
      colorCaseGri(griPosX, griPosY, select, (250, 0, 250))
      if boatSize > 0:
        printBoat(Boat(boatSize, select[0], select[1], orient), griPosX, griPosY)

    def clearCho(oldSelect):
      if boatSize > 0:
        for pos in Boat(boatSize, oldSelect[0], oldSelect[1], orient).pos:
          colorCaseGri(griPosX, griPosY, pos, white)
      else:
        c = white
        if oldSelect+[0] in self.listTirs:
          c = (250,250,0)
        elif oldSelect+[1] in self.listTirs:
          c = (250,0,0)
        colorCaseGri(griPosX, griPosY, oldSelect, c)

    printCho()

    while True:
      vSelect = select[:]
      turned = False

      if keydown(KEY_OK) and ( (boatSize > 0 and self.checkBoatPlcmt(Boat(boatSize, select[0], select[1], orient))) or (boatSize<=0 and select+[0] not in self.listTirs and select+[1] not in self.listTirs)):
        clearCho(select)
        self.lastSelect = select
        return select, orient

      elif keydown(KEY_RIGHT) and (select[0]<9-boatSize if (boatSize >0 and orient=="h") else select[0]<8):
        select[0] += 1
      elif keydown(KEY_LEFT) and select[0]>0:
        select[0] -= 1
      elif keydown(KEY_DOWN) and (select[1]<9-boatSize if (boatSize >0 and orient=="v") else select[1]<8):
        select[1] += 1
      elif keydown(KEY_UP) and select[1]>0:
        select[1] -= 1
      elif keydown(KEY_SHIFT) and boatSize>0:
        clearCho(vSelect)
        if(orient=="h") :
          orient="v"
        else:
          orient="h"
        select = replaceBoatSelect(boatSize, orient, select)
        printCho()
        turned = True

      if vSelect != select and not turned:
        clearCho(vSelect)
        printCho()
      sleep(0.08)

  def checkBoatPlcmt(self, boat):
    for b in self.listBoat:
      for p in b.pos:
        if p in boat.pos: return False
    return True

  def fire(self, player2):
    pos, orient = self.choosePos(20, 30, 0, None)
    for boat in player2.listBoat :
      t, c = False, False
      if pos in boat.pos:
        boat.boat[boat.pos.index(pos)] = 1
        t, c = True, boat.check()
      if t : break
    fill_rect(0,175,150,75,white)
    if t:
        self.listTirs += [pos+[1]]
        if c : draw_string("Bateau coulé !", 10, 175)
        else:  draw_string("Touché !", 10, 175)
        sleep(1.5)
    else:
        self.listTirs += [pos+[0]]
        draw_string("Dans l'eau...", 10, 175)
        sleep(1.5)

  def check(self):
    for i in self.listBoat:
      if not i.check(): return False
    return True

def chooseBoats(pl):
  clearScreen()
  printGri(20, 30) #cho
  printGri(170, 50) #boats
  draw_string("Pivoter : bouton \"shift\"", 10, 180)
  list = [5, 4, 3, 3, 2]
  for i in range(len(list)) :
    draw_string("Restants : "+str(len(list)-i)+"   ", 10, 2)
    pos, orient = pl.choosePos(20, 30, list[i], "h")
    bo = Boat(list[i], pos[0], pos[1], orient)
    pl.listBoat.append(bo)
    printBoat(bo, 170, 50)

def replaceBoatSelect(boatSize, orient, select):
  if select[1]>9-boatSize and orient=="v":
      select[1] = 9-boatSize
  elif select[0]>9-boatSize and orient=="h":
      select[0] = 9-boatSize
  return select

def colorCaseGri(posxgri, posygri, pos, color):
  fill_rect(posxgri+1+14*pos[0], posygri+1+14*pos[1], 10, 10, color)

def printGri(posx, posy):
  fill_rect(posx-1,posy-1,126,126,black)
  fill_rect(posx,posy,124,124,white)
  for x in range(8):fill_rect(posx+12+14*x, posy, 2, 124, black)
  for y in range(8):fill_rect(posx, posy+12+14*y, 124, 2, black)

def printBoat(boat, posxgri, posygri):
  for i in range(boat.size):
      c = boat.pos[i]
      if boat.boat[i] == 0:
        colorCaseGri(posxgri, posygri, c, (250, 150, 0))
      else:
        colorCaseGri(posxgri, posygri, c, (250, 0, 0))

def clearScreen():fill_rect(0, 0, 320, 240, white)

def changingPlayer():
  clearScreen()
  draw_string("JOUEUR SUIVANT",100, 100)
  draw_string("Appuie sur EXE pour commencer ton tour.", 20, 200)
  while not keydown(KEY_EXE):pass

def turn(p1, p2):
  clearScreen()
  printGri(20,30)
  for pos in p1.listTirs:
    if pos[2] == 0:
      colorCaseGri(20, 30, pos, (250, 250, 0))
    else:colorCaseGri(20, 30, pos, (250, 0, 0))
  draw_string("V--CIBLE--V", 15, 2)
  printGri(170, 50)
  for b in p1.listBoat : printBoat(b,170,50)
  draw_string("^-VOS BATEAUX-^",155, 180)
  p1.fire(p2)

def game():
  sleep(0.5)
  clearScreen()
  a = Player()
  chooseBoats(a)
  changingPlayer()
  b = Player()
  chooseBoats(b)
  changingPlayer()
  while True:
    turn(a, b)
    if b.check():
      clearScreen()
      draw_string("VICTOIRE DE J1 !",50,100)
      break
    changingPlayer()
    turn(b, a)
    if a.check():
      clearScreen()
      draw_string("VICTOIRE DE J2 !",50,100)
      break
    changingPlayer()

def menu_bn():
  global darkMode, white, black
  def vis_add():
    printBoat(Boat(5, 0, 0, "h"), 120, 75)
  mod_opt = menu("BATAILLE NAVALE", vis_add, (250, 150, 0), white, [["Mode sombre", ("Non", "Oui"), darkMode]])
  if darkMode != mod_opt[0]:
    darkMode = mod_opt[0]
    white, black = black, white
  if mod_opt[-1]==True:game()

menu_bn()