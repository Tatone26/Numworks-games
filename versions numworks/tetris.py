from math import *
from kandinsky import *
from random import *
from ion import *
from time import *

xgrid = 220
ygrid = 20
hgrid = 22
wgrid = 10

pts = 0
placed_blocs = []

listshape = [[["/1234", "3/3/3/3", "//1234", "2/2/2/2"], (250, 0, 0)],
[["1/123", "23/2/2", "/123/3", "2/2/12"], (0, 250, 0)],
[["3/123", "2/2/23", "/123/1", "12/2/2"], (0, 0, 250)],
[["2/123", "2/23/2", "/123/2", "2/12/2"], (250, 250, 0)],
[["23/12", "2/23/3", "/23/12", "1/12/2"], (250, 0, 250)],
[["12/23", "3/23/2", "/12/23", "2/12/1"], (0, 250, 250)],
[["12/12"], (0, 0, 0)]]

def make_shapes(): #[[form1, form2, form3, form4], nbform, color]
    shapes = []
    for i in listshape:
        sp = [[]]
        for text in i[0]:
            l = text.split("/")
            form = []
            for y in range(len(l)):
                pos = [int(i) for i in l[y]]
                form += [[x, y] for x in pos]
            sp[0].append(form)
        clear = len([x for x in i[0][0].split("/") if len(x)==0])
        for x in sp[0]:
          for e in x:
              e[1]-=clear
        sp.append(0)
        sp.append(i[-1])
        shapes.append(sp)
    return shapes

def blocs(shape):
  if len(shape)>=3 and type(shape[-2])==type(1):
    return shape[0][shape[-2]]
  elif len(shape)==1 and type(shape[0])==type([]):
    return shape
  elif len(shape)==2 and type(shape[1])==type((0, 0)):
    return shape[0]

def random_shape():
  list = make_shapes()
  return list[randint(0, len(list)-1)]

def draw_grid():
  fill_rect(xgrid-2,ygrid-2,8*wgrid+4,8*hgrid+4,(0,0,0))
  for i in range(hgrid):
    for e in range(wgrid):
      draw_grid_pixel(e, i)

def draw_grid_pixel(x, y):
  c = (100,100,100)
  if x%2==0:
    c = (150,150,150)
  fill_rect(xgrid+8*x,ygrid+8*y,8,8,c)

def draw_ui():
  draw_string("Points : "+str(pts),5,20)

def draw_shape(shape):
  color = shape[-1]
  for i in blocs(shape):
    fill_rect(xgrid+8*i[0],ygrid+8*i[1],8,8,color)

def clear_shape(shape):
  for i in blocs(shape):
    draw_grid_pixel(i[0], i[1])

def check_keys():
  if keydown(KEY_LEFT):return [-1,0]
  elif keydown(KEY_RIGHT):return [1,0]
  elif keydown(KEY_DOWN):return [0,1]
  elif keydown(KEY_EXE):return True
  else:return [0,0]

def touch_down(shape):
  for i in blocs(shape):
    if i[1] >= hgrid-1:return True
    elif [i[0],i[1]+1] in placed_blocs:return True
  return False

def touch_side(shape, dir):
  for i in blocs(shape):
    if [i[0]+dir[0],i[1]] in placed_blocs:return True #si la shape touche une autre
    elif dir==[1,0]and i[0]>=wgrid-1:return True  #si elle touche les bords
    elif dir==[-1,0] and i[0]<=0:return True
  return False #si elle ne touche rien

def sort_blocs():
  global placed_blocs
  placed_blocs = sorted(placed_blocs, key=lambda f: [f[1],f[0]])

def check_lines(shape):
  hgt = list(set([i[1] for i in blocs(shape)]))
  lines_check = [0 for i in range(len(hgt))]
  for i in placed_blocs:
    if i[1] in hgt:
      lines_check[hgt.index(i[1])] += 1
  broken = []
  for i in lines_check :
    if i >= wgrid:
      broken.append(hgt[lines_check.index(i)])
  broken.sort()
  broken.reverse()
  return broken

def bring_squares_down(shape):
  global placed_blocs, pts
  broken_lines = check_lines(shape)
  used_score = []
  for i in broken_lines:
    if i not in used_score:
      score = 1
      used_score.append(i)
      while i+1 in broken_lines and i+1 not in used_score:
        score+=1
        used_score.append(i+1)
      if score == 1 : pts+=40
      elif score == 2 : pts+=100
      elif score == 3 : pts+=300
      elif score == 4 : pts+=1200
  toRemove = []
  for i in placed_blocs:
    if i[1] in broken_lines:
      clear_shape([i])
      toRemove.append(i)
  toRemove.reverse()
  for i in toRemove:
    placed_blocs.remove(i)
  for i in broken_lines:
    toDraw = []
    for e in placed_blocs:
      if e[1]<i:
        color = get_pixel(xgrid+8*e[0]+1,ygrid+8*e[1]+1)
        clear_shape([e])
        e[1]+=1
        toDraw.append([[e], color])
    for i in toDraw :
      draw_shape(i)

def move(shape, dir):
  for i in shape[0]:
    for e in i:
      e[1]+=dir[1]
      e[0]+=dir[0]

def move_shape(shape):
  global placed_blocs

  while not touch_down(shape):
    clear_shape(shape)
    move(shape, [0, 1])
    draw_shape(shape)
    start = monotonic()
    turndown = 0
    while monotonic()<start+0.3:
      sleep(0.02)
      if turndown != 0 and monotonic()>=turndown+0.10:
        turndown = False
      dir = check_keys()
      if type(dir)==type(True) and not turndown:
        clear_shape(shape)
        turn_shape(shape)
        draw_shape(shape)
        turndown = monotonic()
      elif dir!=[0,0] and type(dir)==type([]):
        if dir==[0,1]:
          if touch_down(shape):break
          else:
            clear_shape(shape)
            move(shape, dir)
            draw_shape(shape)
        elif not touch_side(shape, dir):
          clear_shape(shape)
          move(shape, dir)
          draw_shape(shape)
  placed_blocs += blocs(shape)
  #sort_blocs()
  bring_squares_down(shape)
  draw_ui()

def turn_shape(shape):

  def check_pos():
    for i in blocs(shape):
      if i in placed_blocs or i[0]<0 or i[0]>wgrid-1 or i[1]>hgrid:
        return False
    return True

  if len(shape)>=3 and type(shape[-2])==type(1):
    max = len(shape[0])-1
    old = shape[-2]
    if shape[-2]+1>max:
      shape[-2]=0
    else:
      shape[-2]+=1
    movedright = False
    movedleft = False
    movedup = False
    for i in blocs(shape):
      while i[0]<0:
        move(shape, [1, 0])
        movedright = True
      while i[0]>wgrid-1:
        move(shape, [-1, 0])
        movedleft = True
      while i[1]>hgrid-1:
        move(shape, [0, -1])
        movedup = True
      if i in placed_blocs and [i[0]+1, i[1]] not in placed_blocs and not movedright:
        move(shape, [1, 0])
        if not check_pos():
          move(shape, [-1, 0])
        else :
          movedright = True
      if i in placed_blocs and [i[0]-1, i[1]] not in placed_blocs and not movedleft:
        move(shape, [-1, 0])
        if not check_pos():
          move(shape, [1, 0])
        else :
          movedleft = True
    if not check_pos():
        shape[-2] = old
  return shape

def lose():
  for i in placed_blocs:
    if i[1]<=0:return True

def tetris():
  draw_grid()
  while not lose():
    draw_ui()
    shape = random_shape()
    draw_shape(shape)
    sleep(0.2)
    move_shape(shape)
  draw_string("PERDU", 0, 0)

tetris()