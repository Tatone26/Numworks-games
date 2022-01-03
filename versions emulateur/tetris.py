from num_to_wind import *

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
  else:
    return shape

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
  if keydown(KEY_LEFT):return -1,0
  elif keydown(KEY_RIGHT):return 1,0
  elif keydown(KEY_DOWN):return 0,1
  elif keydown(KEY_OK):return True
  elif keydown(KEY_BACK):return False
  else:return 0,0

def touch_down(shape):
  for i in blocs(shape):
    if i[1] >= hgrid-1:return True
    elif [i[0],i[1]+1] in placed_blocs:return True
  return False

def touch_side(shape, dir):
  for i in blocs(shape):
    if [i[0]+dir[0],i[1]] in placed_blocs:return True #si la shape touche une autre
    elif dir[0]==1 and i[0]>=wgrid-1:return True  #si elle touche les bords
    elif dir[0]==-1 and i[0]<=0:return True
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
  return shape

def move_shape(shape):
  global placed_blocs

  while not touch_down(shape):
    clear_shape(shape)
    shape = move(shape, (0, 1))
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
        shape = turn_shape(shape, dir)
        draw_shape(shape)
        turndown = monotonic()
      elif dir!=(0,0) and type(dir)==type((0, 0)):
        if dir==(0,1):
          if touch_down(shape):break
          else:
            clear_shape(shape)
            shape = move(shape, dir)
            draw_shape(shape)
        elif not touch_side(shape, dir):
          clear_shape(shape)
          shape = move(shape, dir)
          draw_shape(shape)
  placed_blocs += blocs(shape)
  bring_squares_down(shape)
  draw_ui()

def turn_shape(shape, right):

  def check_pos(sh):
    for i in blocs(sh):
      if (i in placed_blocs) or (i[0]<0) or (i[0]>wgrid-1) or (i[1]>hgrid-1):
        return False
    return True

  def test_rotation(dir, sh):
    testsh = blocs(sh)[:]
    for i in testsh:
      i[0]-=dir[0]
      i[1]-=dir[1] #the rotation table was oriented the opposite
    if check_pos(testsh):
      return True
    else:
      return False

  if len(shape)>=3 and type(shape[-2])==type(1): #check if good shape to turn (complete shape)

    maxi = len(shape[0])-1 #maximum value of nbform
    new = shape[:] #conserving the initial shape and everything

    if right:
      if new[-2]+1>maxi:
        new[-2]=0
      else:
        new[-2]+=1
    else:
      if new[-2]-1<0:
        new[-2]=maxi
      else:
        new[-2]-=1

    nrot = new[-2] #new rotation nb
    orot = shape[-2] #old rotation nb

    if not check_pos(new) and maxi != 0: #the square doesn't turn, if that test is passed -> maxi = 3
      testI = max(len(set([i[1] for i in blocs(new)])), len(set([i[0] for i in blocs(new)])))
      success = False
      test = []
      if testI < 4: #if it is not the I
        if orot == 0:
          if nrot == 1:
            test = [(-1, 0), (-1, 1), (0, -2), (-1, -2)]
          elif nrot == 3:
            test = [(1, 0), (1, 1), (0, -2), (1, -2)]
        elif orot == 1:
          if nrot == 0 or nrot == 2:
            test = [(1, 0), (1, -1), (0, 2), (1, 2)]
        elif orot == 2:
          if nrot == 1 :
            test = [(-1, 0), (-1, 1), (0, -2), (-1, -2)]
          elif nrot == 3:
            test = [(1, 0), (1, 1), (0, -2), (1, -2)]
        elif orot == 3:
          if nrot == 0 or nrot == 2:
            test = [(1, 0), (-1, 1), (0, 2), (-1, 2)]
      else :
        if orot == 0:
          if nrot == 1:
            test = [(-2, 0), (1, 0), (-2, -1), (1, 2)]
          elif nrot == 3:
            test = [(-1, 0), (2, 0), (-1, 2), (2, -1)]
        elif orot == 1:
          if nrot == 2:
            test =[(-1, 0), (2, 0), (-1, 2), (2, -1)]
          elif nrot == 0:
            test = [(2, 0), (-1, 0), (2, 1), (-1, -2)]
        elif orot == 2:
          if nrot == 1 :
            test = [(1, 0), (-2, 0), (1, -2), (-2, 1)]
          elif nrot == 3:
            test = [(2, 0), (-1, 0), (2, 1), (-1, -2)]
        elif orot == 3:
          if nrot == 0:
            test = [(1, 0), (-2, 0), (1, -2), (-2, 1)]
          elif nrot == 2:
            test = [(-2, 0), (1, 0), (-2, -1), (1, 2)]
      for t in test:
        success = test_rotation(t, new)
        if success:
          new = move(new, (-t[0], -t[1]))
          return new
        else :
          new = shape[:]
          new[-2] = orot
          return new
      return shape
    else : return new


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