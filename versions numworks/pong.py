from kandinsky import *
from ion import *
from time import *
from menu import menu
from random import choice

black = color(0, 0, 0)
white = color(255, 255, 255)
darkMode = False
maxPoints = 10

def pong():
  global white, black, darkMode, maxPoints
  p1=player(KEY_SEVEN, KEY_ONE, 20, 100, 45)
  p2=player(KEY_RIGHTPARENTHESIS, KEY_MINUS, 300,100, 45)
  balle = ball(158, 118, p1, p2, 2)
  def vis_add():
    balle.printSelf()
    p1.printSelf()
    p2.printSelf()
  list_opt = [["Mode sombre", ("Non", "Oui"), darkMode], ["Points pour vic", ("1","2","3","4","5","6","7","8","9","10"), maxPoints]]
  mod_opt = menu("PONG", vis_add, black, white, list_opt)
  if mod_opt[0]!=darkMode:
      white, black = black, white
      darkMode = mod_opt[0]
  maxPoints = mod_opt[1]
  if mod_opt[-1]==True: play(p1, p2, balle)

def play(p1, p2, ba):
  fill_rect(0, 0, 350, 350, white)
  p1.drawPoints()
  p2.drawPoints()
  p1.posy = 100
  p2.posy = 100
  p1.printSelf()
  p2.printSelf()
  sleep(0.3)
  rd1 = choice([-1, 1])
  rd2 = choice([-1, 1])
  ba.posx = 158
  ba.posy = 118
  ba.vel = [ba.vel[0]*rd1, ba.vel[1]*rd2]
  while True:
    p1.mvt()
    p2.mvt()
    ba.act()

def point(p1, p2, ba):
  p1.addPoint()
  if p1.points == maxPoints:
    draw_string("VICTOIRE", 100, 100)
    draw_string("<OK> pour retourner au menu", 20, 190)
    while not keydown(KEY_OK):pass
    pong()
  else:
    draw_string("POINT", 100, 100)
    sleep(1)
    play(p1, p2, ba)

class player:
  def __init__(self, up, down, posx, posy, size):
    self.kup = up
    self.kdown = down
    self.posx = posx
    self.posy = posy
    self.size = size
    self.points = 0

  def addPoint(self):
    self.points += 1
    self.drawPoints()

  def drawPoints(self):
    if self.posx < 150:
      draw_string(str(self.points), self.posx+75, 25)
    else: draw_string(str(self.points), self.posx-75, 25)

  def mvt(self):
    if keydown(self.kup) and not keydown(self.kdown) and self.posy>0:
      self.clearSelf()
      self.posy-=3
      self.printSelf()
    elif keydown(self.kdown) and not keydown(self.kup) and self.posy<220-self.size:
      self.clearSelf()
      self.posy+=3
      self.printSelf()

  def printSelf(self):
    fill_rect(self.posx,self.posy,5,self.size,black)
  def clearSelf(self):
    fill_rect(self.posx,self.posy,5,self.size, white)

class ball:
  def __init__(self, posx, posy, player1, player2, speed):
    self.posx=posx
    self.posy=posy
    self.pOne = player1
    self.pTwo = player2
    self.vel = [speed, speed]

  def act(self):
    self.clearSelf()
    self.posx += self.vel[0]
    self.posy += self.vel[1]
    self.checkContact()
    self.printSelf()

  def checkContact(self):
    if self.posy <=5 or self.posy >= 220:
      self.vel[1] = self.vel[1]*(-1)
    if self.posx <= self.pOne.posx+5 and (self.pOne.posy<=self.posy<=self.pOne.posy+self.pOne.size):
      self.vel[0] = self.vel[0]*(-1)
    elif self.posx >= self.pTwo.posx and(self.pTwo.posy<=self.posy<=self.pTwo.posy+self.pTwo.size):
      self.vel[0] = self.vel[0]*(-1)
    elif self.posx <= self.pOne.posx+4:
      point(self.pTwo, self.pOne, self)
    elif self.posx >= self.pTwo.posx:
      point(self.pOne, self.pTwo, self)

  def printSelf(self):
    fill_rect(self.posx-2, self.posy-2, 4, 4, black)
  def clearSelf(self):
    fill_rect(self.posx-2, self.posy-2, 4, 4, white)

pong()