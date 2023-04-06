from num_to_wind import *

xgrid = 220
ygrid = 20
hgrid = 22
wgrid = 10

pts = 0
placed_blocs = []

'''
par ligne :
a1234b1234c1234d1234 (4 seulement pour I)
ordre : I, L, L2, T, S, S2, O
'''
listshape = [[["/1234", "3/3/3/3", "//1234", "2/2/2/2"], (250, 0, 0)],
[["1/123", "23/2/2", "/123/3", "2/2/12"], (0, 250, 0)],
[["3/123", "2/2/23", "/123/1", "12/2/2"], (0, 0, 250)],
[["2/123", "2/23/2", "/123/2", "2/12/2"], (250, 250, 0)],
[["23/12", "2/23/3", "/23/12", "1/12/2"], (250, 0, 250)],
[["12/23", "3/23/2", "/12/23", "2/12/1"], (0, 250, 250)],
[["12/12"], (0, 0, 0)]]

def make_shapes(): #[[form1, form2, form3, form4], color, nbform]
    shapes = []
    for i in listshape:
        sp = [[]]
        for text in i[0]:
            l = text.split("/")
            form = []
            for y in range(len(l)):
                pos = [int(i) for i in l[y]]
                form += [[x, y+1] for x in pos]
            sp[0].append(form)
        sp.append(i[-1])
        sp.append(0)
        print(sp)
        shapes.append(sp)
    return shapes

def blocs(shape):
    if len(shape)>2:
        return shape[0][shape[-1]]
    elif len(shape)==1:
        return shape
    else:
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
  color = shape[-2]
  for i in blocs(shape):
    fill_rect(xgrid+8*i[0],ygrid+8*i[1],8,8,color)

def clear_shape(shape):
    if len(shape)>1:
        list = blocs(shape)
    else :
        list = shape
    for i in list:
        draw_grid_pixel(i[0], i[1])

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

def sort_blocs():
  global placed_blocs
  placed_blocs = sorted(placed_blocs, key=lambda f: [f[1],f[0]])

def touch_side(shape, dir):
  for i in blocs(shape):
    if [i[0]+dir[0],i[1]] in placed_blocs:return True #si la shape touche une autre
    elif dir==[1,0]and i[0]>=9:return True  #si elle touche les bords
    elif dir==[-1,0] and i[0]<=0:return True
  return False #si elle ne touche rien

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

def bring_squares_down(shape):
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
        toDraw.append([[e], color])
    for i in toDraw :
      draw_shape(i)

def move_shape(shape):
  global placed_blocs
  while not touch_down(shape):
    clear_shape(shape)
    for i in shape[0]:
        for e in i:
            e[1]+=1
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
            for i in shape[0]:
                for e in i:
                    e[1]+=dir[1]
            draw_shape(shape)
        elif not touch_side(shape, dir):
          clear_shape(shape)
          for i in shape[0]:
              for e in i:
                  e[0]+=dir[0]
          draw_shape(shape)
  placed_blocs += blocs(shape)
  sort_blocs()
  bring_squares_down(shape)

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