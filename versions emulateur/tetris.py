from num_to_wind import *

xgrid = 220
ygrid = 20
hgrid = 22
wgrid = 10

pts = 0
placed_blocs = []

def blocs(shape):
  '''Récupère uniquement les blocs de la shape (exclus la couleur notamment)'''
  for i in shape:
    if type(i) == type([]):
      yield i

def draw_grid():
  '''Dessine la grille.'''
  fill_rect(xgrid-2,ygrid-2,8*wgrid+4,8*hgrid+4,(0,0,0))
  for i in range(hgrid):
    for e in range(wgrid):
      draw_grid_pixel(e, i)

def draw_grid_pixel(x, y):
  '''Dessine un pixel de la grille'''
  c = (100,100,100)
  if x%2==0:
    c = (150,150,150)
  fill_rect(xgrid+8*x,ygrid+8*y,8,8,c)

def draw_ui():
  '''Dessine tout ce qu'il y a à gauche (points etc)'''
  draw_string("Points : "+str(pts),5,20)

def draw_shape(shape):
  '''Dessine une shape sur la grille.'''
  color = [e for e in shape if type(e)==type((0, 0)) and len(e)==3]
  for i in blocs(shape):
    fill_rect(xgrid+8*i[0],ygrid+8*i[1],8,8,color[0])

def clear_shape(shape):
  '''Efface une shape.'''
  for i in blocs(shape):
    draw_grid_pixel(i[0], i[1])

def move_shape(shape): #TODO : améliorer algo
  '''Gère le déplacements d'une shape (chute) et vérification de toutes les collisions et input.'''
  global placed_blocs
  while not touch_down(shape):
    clear_shape(shape)
    for i in blocs(shape):
      i[1]+=1
    draw_shape(shape)
    start = monotonic()
    while monotonic()<start+0.3:
      sleep(0.02)
      dir = check_keys()
      if dir!=[0,0]:
        if dir==[0,1]:
          if touch_down(shape):break
          else:
            clear_shape(shape)
            for i in blocs(shape):
              i[1]+=1
            draw_shape(shape)
        elif not touch_side(shape, dir):
          clear_shape(shape)
          for i in blocs(shape):
            i[0]+=dir[0]
          draw_shape(shape)
  placed_blocs += blocs(shape)
  sort_blocs()
  bring_squares_down(shape)

def turn_shape(shape):
  #pos pivot : shape[5]
  # 1 : trouver le centre.
  # 2 : pivoter. (bon courage)
  # 3 : floor kicks
  # 4 : si floor kicks fonctionnent pas alors rotation annulée
  # 5 : sinon rotation effectuée et return nouvelle shape
  pass

def sort_blocs():
  global placed_blocs
  placed_blocs = sorted(placed_blocs, key=lambda f: [f[1],f[0]])

def touch_side(shape, dir):
  for i in blocs(shape):
    if [i[0]+dir[0],i[1]] in placed_blocs:return True #si la shape touche une autre
    elif dir==[1,0]and i[0]>=9:return True  #si elle touche les bords
    elif dir==[-1,0] and i[0]<=0:return True
  return False #si elle ne touche rien

def check_keys():
  if keydown(KEY_LEFT):return [-1,0]
  elif keydown(KEY_RIGHT):return [1,0]
  elif keydown(KEY_DOWN):return [0,1]
  else:return [0,0]

def touch_down(shape):
  for i in blocs(shape):
    if i[1] >= hgrid-1:return True
    elif [i[0],i[1]+1] in placed_blocs:return True
  return False

def check_lines(shape):
  hgt = list(set([i[1] for i in blocs(shape)]))
  lines_check = [0 for i in range(len(hgt))]
  for i in placed_blocs:
    if i[1] in hgt:
      lines_check[hgt.index(i[1])] += 1
  broken = []
  for i in lines_check :
    if i >= wgrid - 3: #enlever le -3 quand pièces pourront tourner sans pb
      broken.append(hgt[lines_check.index(i)])
  return broken

def bring_squares_down(shape): #ça marche ???? check bugs pls
  global placed_blocs
  broken_lines = check_lines(shape)
  toRemove = []
  for i in range(len(placed_blocs)):
    if placed_blocs[i][1] in broken_lines:
      clear_shape([placed_blocs[i]])
      toRemove.append(i)
  toRemove.reverse()
  for i in toRemove :
    placed_blocs.pop(i)
  for i in broken_lines:
    toDraw = []
    for e in placed_blocs:
      if e[1]<i: #ça me paraît le pb
        color = get_pixel(xgrid+8*e[0]+1,ygrid+8*e[1]+1)
        clear_shape([e])
        e[1]+=1
        toDraw.append([e, color])
    for i in toDraw :
      draw_shape(i)

def random_shape(): #[bloc1, bloc2, bloc3, bloc4, color, pospivotdansliste ?]
  list = [[[3,0],[4,0],[5,0],[5,1], (250, 0, 0), 2],
  [[3,0],[3,1],[4,1],[4,0], (0, 250, 0), 3],
  [[3,0],[3,1],[4,0],[5,0], (0, 0, 250), 1],
  [[3,0],[4,0],[5,0],[6,0], (250, 250, 0), 1],
  [[3,0],[4,0],[4,1],[5,1], (250, 0, 250), 1],
  [[3,1],[4,1],[4,0],[5,0], (0, 250, 250), 2],
  [[3,0],[4,0],[5,0],[4,1], (0, 0, 0), 1]]
  return list[randint(0, len(list)-1)]

def lose():
  for i in placed_blocs:
    if i[1]<=0:return True

def tetris():
  draw_grid()
  while not lose():
    draw_ui()
    shape = random_shape()
    draw_shape(shape)
    move_shape(shape)
  draw_string("PERDU", 0, 0)

tetris()