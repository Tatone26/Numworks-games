from kandinsky import *
from ion import *
from time import *
from menu import menu, draw_centered_string, fill_screen

white = color(255, 255, 255)
black = color(0, 0, 0)
blue = color(0, 0, 255)
red = color(255, 0, 0)
green = color(0, 255, 0)
placedCoins = []

darkMode = False
visGrid = True
visChoice = True
title = "PUISSANCE 4"
nbPlayers = 2

def check():
  if hAlign() or vAlign() or dhAlign() or dbAlign():
    return True
  return False
def hAlign():
  for x in range(5):
    for y in range(6):
      co = placedCoins[x][y]
      if co != None and placedCoins[x+1][y] == co and placedCoins[x+2][y]==co and (placedCoins[x+3][y]==co or nbPlayers>=3):
        return True
  return False
def vAlign():
  for x in range(7):
    for y in range(4):
      co = placedCoins[x][y]
      if co != None and placedCoins[x][y+1] == co and placedCoins[x][y+2]==co and (placedCoins[x][y+3]==co or nbPlayers>=3):
        return True
  return False
def dhAlign():
  for x in range(5):
    for y in range(3):
      co = placedCoins[x][y]
      if co != None and placedCoins[x+1][y+1] == co and placedCoins[x+2][y+2]==co and (placedCoins[x+3][y+3]==co or nbPlayers>=3):
        return True
  return False
def dbAlign():
  for x in range(5):
    for y in range(2, 6):
      co = placedCoins[x][y]
      if co != None and placedCoins[x+1][y-1] == co and placedCoins[x+2][y-2]==co and (placedCoins[x+3][y-3]==co or nbPlayers>=3):
        return True
  return False

def placeCoin(posx, color):
  h = len([f for f in placedCoins[posx] if f!=None])
  placedCoins[posx][h] = color
  return h

def printGrid():
  fill_rect(50, 50, 212, 170, black)
  for x in range(7):
    fill_rect(52+x*30, 50, 28, 168, white)
    for y in range(6):
        if not darkMode: printCoin(x,y,(240,240,240))
        else: printCoin(x,y,(30,30,30))
def printCoin(posx,posy,color) : fill_rect(53+30*posx, 191-28*posy, 26, 26, color)
def clearCoin(pos) : fill_rect(53+30*pos, 23, 26, 26, white)

def selectPosCoin(color):
  select = 3
  fill_rect(0, 0, 350, 10, color)
  if visChoice : printCoin(select, 6, color)
  while True:
    if keydown(KEY_LEFT) and select>0:
      clearCoin(select)
      select -= 1
      if visChoice : printCoin(select, 6, color)
    elif keydown(KEY_RIGHT) and select<6:
      clearCoin(select)
      select +=1
      if visChoice : printCoin(select, 6, color)
    elif (keydown(KEY_OK)or keydown(KEY_DOWN)) and len([f for f in placedCoins[select] if f!=None])<=5:
      clearCoin(select)
      break
    sleep(0.1)
  h = placeCoin(select, color)
  if visGrid : printCoin(select, h, color)
  sleep(0.3)

def p4():
  global placedCoins
  fill_screen(white)
  placedCoins = [[None]*6 for k in range(0,7)]
  if visGrid:printGrid()
  sleep(0.4)
  while not check():
    selectPosCoin(blue)
    if check(): break
    selectPosCoin(red)
    if check():break
    if nbPlayers>=3:
      selectPosCoin(green)
    if check(): break
  printGrid()
  for x in range(7):
    for y in range(6):
      if placedCoins[x][y]!=None:
        printCoin(x, y, placedCoins[x][y])
  draw_centered_string(title+" !",100)
  draw_string("MENU : EXE", 180, 200)
  draw_string("REJOUER : OK",20,200)
  while not keydown(KEY_EXE) and not keydown(KEY_OK):
    pass
  if keydown(KEY_EXE):menu_p4()
  else:p4()

def menu_p4():
  global darkMode, white, black, visGrid, visChoice, nbPlayers, title
  def vis_add():
    printCoin(2, 4, red)
    printCoin(4, 4, blue)
    if nbPlayers >=3:
      printCoin(3,4, green)
  list_opt = [["Mode sombre", ("Non", "Oui"), darkMode], ["Grille visible", ("Non", "Oui"), visGrid], ["Choix visible", ("Non", "Oui"), visChoice], ["Nb de joueurs", ("2", "3"), nbPlayers-1]]
  modif_opt = menu(title, vis_add, blue, white, list_opt)
  if modif_opt[0]!=darkMode:white, black = black, white
  darkMode = modif_opt[0]
  visGrid = modif_opt[1]
  visChoice = modif_opt[2]
  nbPlayers = modif_opt[3]+1
  title = "PUISSANCE "+str(6-nbPlayers)
  if modif_opt[-1]==True:p4()

menu_p4()