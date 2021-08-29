from math import *
from kandinsky import *
from ion import *
from random import *
from menu import *

xg = 120
yg = 20
sz = 4
pts = 0
best = 1112
lbx = []

def dwGrid():
  fill_rect(120,20,185,185,(180,180,180))
  for i in range(1,sz+1):
    for e in range(1,sz+1): dwNoBox(i,e)

def dwNoBox(x,y):
  fill_rect(xg+1+int(185/sz)*(x-1),yg+1+int(185/sz)*(y-1),int(185/sz)-1,int(185/sz)-1,(220,220,220))
def dwBox(box):
  fill_rect(xg+1+int(185/sz)*(box[0]-1),yg+1+int(185/sz)*(box[1]-1),int(185/sz)-1,int(185/sz)-1,(250-box[2]*10,250-box[2]*20,250-box[2]*20))
  draw_string(str(2**box[2]),xg+int(185/sz/2)+int(185/sz)*(box[0]-1)-int(len(str(2**box[2]))/2*8),yg+int(185/sz/3)-2+int(185/sz)*(box[1]-1))

def dwLbx():
  for i in lbx: dwBox(i)

def pinput():
  while True:
    if keydown(KEY_UP):return [0,-1]
    elif keydown(KEY_DOWN):return [0,1]
    elif keydown(KEY_RIGHT):return [1,0]
    elif keydown(KEY_LEFT):return [-1,0]

def fuseBoxes(d):
  nlb=[]
  fused=[]
  pts = 0
  for i in lbx:
    t = [i[0]+d[0]*(-1),i[1]+d[1]*(-1),i[2]]
    if t in lbx and (t[0], t[1]) not in fused:
      nlb.append([i[0],i[1],i[2]+1])
      fused.append((i[0]+d[0]*(-1), i[1]+d[1]*(-1)))
      pts += 2**(i[2]+1)
    elif (i[0], i[1]) not in fused:
      nlb.append(i)
  return nlb, pts

def moveBoxes(d):
  nlb=lbx
  def m(l,d):
    nl=[]
    add=False
    for i in range(len(l)):
      ept=True
      for e in l:
        if [e[0],e[1]]==[l[i][0]+d[0],l[i][1]+d[1]]:
          ept=False
      if ept==True and 0<l[i][0]+d[0]<=sz and 0<l[i][1]+d[1]<=sz:
        nl.append([l[i][0]+d[0],l[i][1]+d[1],l[i][2]])
        add = True
      else: nl.append(l[i])
    return add,nl
  mvd = False
  while True:
    t,nlb=m(nlb,d)
    if t==False: break
    mvd = True
  return nlb, mvd

def addBox():
  global lbx
  if len(lbx)<sz*sz:
    while True :
      turn = False
      nb = [randint(1, sz), randint(1, sz), 1]
      for e in lbx:
        if [e[0], e[1]] == [nb[0], nb[1]]:
          turn = True
          break
      if turn == False:
        lbx.append(nb)
        break
    return True
  return False

def vaStart():
  fill_rect(0,0,320,240,(255,255,255))
  dwGrid()
  dwLbx()
  draw_string("Record : \n    "+str(best), 5, 180)
  draw_string("Points :", 5, 20)
  draw_string("0", 35, 45)

def game():
  global lbx, pts
  lbx = [[randint(1, sz), randint(1, sz), 1]]
  vaStart()
  while True:
    d=pinput()
    lbx, mvd1 = moveBoxes(d)
    lbx, np = fuseBoxes(d)
    lbx = moveBoxes(d)[0]
    if mvd1 or np>0:
      pts += np
      draw_string(str(pts), 35, 45)
      live = addBox()
      if not live:
        if not end():break
      dwGrid()
      dwLbx()
    sleep(0.3)
  menu2048()

def end():
  global lbx
  draw_string("Perdu !", xg+70, yg+15)
  draw_string("Rejouer : <OK>", xg+5, yg+115)
  draw_string("Menu : <EXE>", xg+5, yg+160)
  if pts>best: draw_string("Nouveau \n record !", 5, 100)
  while True:
    if keydown(KEY_OK):
      lbx = [[randint(1, sz), randint(1, sz), 1]]
      vaStart()
      return True
    elif keydown(KEY_EXE):return False

def menu2048():
  global sz
  def va():
    fill_rect(0,75,320,20,(230,180,180))
  opt = menu("2048",va,(0,0,0),(255,255,255),[["Taille", ("3", "4", "5", "6", "7", "8"), sz-2]])
  sz = opt[0]+2
  if opt[-1]==True:game()

menu2048()