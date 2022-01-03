from kandinsky import *
from ion import *
from time import *
from menu import menu, draw_centered_string, fill_screen

white = color(255,255,255)
black = color(0, 0, 0)
darkMode = False

class Player:
  def __init__(self):
    self.listBoats = []
    self.listTirs = []
    self.lastSel = [0, 0]

def colorCaseGri(griPos, pos, color):
  fill_rect(griPos[0]+1+14*pos[0], griPos[1]+1+14*pos[1], 10, 10, color)

def printGri(griPos):
  posx = griPos[0]
  posy = griPos[1]
  fill_rect(posx-1,posy-1,126,126,black)
  fill_rect(posx,posy,124,124,white)
  for x in range(8):fill_rect(posx+12+14*x, posy, 2, 124, black)
  for y in range(8):fill_rect(posx, posy+12+14*y, 124, 2, black)

def printTir(griPos, player, tir):
  c = white
  if tir+[0] in player.listTirs:
    c = (250, 250, 0)
  elif tir+[1] in player.listTirs:
    c = (250, 0, 0)
  colorCaseGri(griPos, tir, c)

def printBoat(griPos, boat):
  for i in range(len(boat)):
    colorCaseGri(griPos, boat[i][:2], (250, abs(boat[i][2]-1)*150, 0))

def selectPos(griPos, sInit = [0, 0], opt = [5, (1, 0)], player = None) : #select initial, griPos, opt = [size, orient]

  def printCho():
    for i in range(opt[0]):
      colorCaseGri(griPos, [select[0]+1*i*opt[1][0], select[1]+1*i*opt[1][1]], (250, 0, 250))

  def clearCho():
    for i in range(opt[0]):
      s = [select[0]+1*i*opt[1][0], select[1]+1*i*opt[1][1]]
      if player == None:
        colorCaseGri(griPos, s, white)
      else:
        printTir(griPos, player, s)

  def replace():
    if select[1]>9-opt[0] and opt[1]==(0, 1):
        select[1] = 9-opt[0]
    elif select[0]>9-opt[0] and opt[1]==(1, 0):
        select[0] = 9-opt[0]

  select = sInit
  replace()
  printCho()
  sleep(0.5)
  while True:

    oldSel = select[:]
    turned = False
    if keydown(KEY_OK):
      clearCho()
      return select, opt
    elif keydown(KEY_EXE):
      clearCho()
      opt[1] = (opt[1][1], opt[1][0])
      replace()
      printCho()
      turned = True

    elif keydown(KEY_RIGHT) and (select[0]<9-((opt[0]-1)*opt[1][0]+1)):
      select[0] += 1
    elif keydown(KEY_LEFT) and (select[0]>0):
      select[0] -= 1
    elif keydown(KEY_DOWN) and (select[1]<9-((opt[0]-1)*opt[1][1]+1)):
      select[1] += 1
    elif keydown(KEY_UP) and (select[1]>0):
      select[1] -= 1

    if oldSel != select and not turned:
      tempSel = select[:]
      select = oldSel
      clearCho()
      select = tempSel[:]
      printCho()
    sleep(0.05)

def changeScreen(text, nextPlayer):
  fill_screen(white)
  draw_centered_string("PLAYER "+str(nextPlayer), 75)
  draw_centered_string(text, 100)
  draw_centered_string("Press <EXE> to continue.", 200)
  while not keydown(KEY_EXE):
    sleep(0.2)
  fill_screen(white)

def placeBoats(player):
  printGri((20, 20))
  printGri((170, 50))
  sizes = [5, 4, 3, 3, 2]
  i = 0
  while i<len(sizes):
    opt = [sizes[i], (1, 0)]
    player.lastSel, opt = selectPos((20,20), player.lastSel[:], opt)
    boat = [[player.lastSel[0]+i*opt[1][0], player.lastSel[1]+i*opt[1][1], 0] for i in range(opt[0])]
    verif = False
    for b in player.listBoats :
      for c in boat:
        if c in b:verif = True
    if not verif:
      player.listBoats.append([[player.lastSel[0]+i*opt[1][0], player.lastSel[1]+i*opt[1][1], 0] for i in range(opt[0])])
      printBoat((170, 50), player.listBoats[-1])
      i += 1

def turn(player, cible):
  printGri((20, 20))
  draw_string("V--CIBLE--V", 15, 2)
  printGri((170, 50))
  draw_string("^-VOS BATEAUX-^", 155, 180)
  for b in player.listBoats : printBoat((170, 50), b)
  for t in player.listTirs : printTir((20, 20), player, [t[0], t[1]])
  while True:
    pos, o = selectPos((20, 20), player.lastSel, [1, (1, 1)], player)
    player.lastSel = pos[:]
    if pos in [[i[0], i[1]] for i in player.listTirs]:
      continue
    elif pos+[1] in [i for b in cible.listBoats for i in b]:
      continue
    elif pos+[0] not in [i for b in cible.listBoats for i in b]:
      player.listTirs.append(pos+[0])
      draw_string("Dans l'eau...", 10, 175)
      sleep(1)
      break
    else :
      player.listTirs.append(pos+[1])
      draw_string("Touché !", 10, 175)
      for b in cible.listBoats:
        if pos+[0] in b:
          b[b.index(pos+[0])][2] = 1
          if False not in [i[2] == 1 for i in b]:
            draw_string("Touché coulé !", 10, 175)
      sleep(1)
      break


def game():
  p1 = Player()
  p2 = Player()
  changeScreen("is going to place his boats.", 1)
  placeBoats(p1)
  changeScreen("is going to place his boats.", 2)
  placeBoats(p2)
  win = None
  while True:
    changeScreen("is going to attack.", 1)
    turn(p1, p2)
    if False not in [i[2]==1 for b in p2.listBoats for i in b]:
      win = 1
      break
    changeScreen("is going to attack.", 2)
    turn(p2, p1)
    if False not in [i[2]==1 for b in p1.listBoats for i in b]:
      win = 2
      break
  changeScreen("WON", win)
  menu_bn()

def menu_bn():
  global darkMode, white, black
  def vis_add():
    printBoat((120, 75), [[0, 0, 0], [1, 0, 0], [2, 0, 1], [3, 0, 0], [4, 0, 0]])
  mod_opt = menu("BATAILLE NAVALE", vis_add, (250, 150, 0), white, [["Mode sombre", ("Non", "Oui"), darkMode]])
  if darkMode != mod_opt[0]:
    darkMode = mod_opt[0]
    white, black = black, white
  if mod_opt[-1]==True:game()

menu_bn()