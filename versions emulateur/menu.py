from num_to_wind import *

def menu(titre, visible_addons, select_col, bkgd_col, list_opt):
  modif_opt = [i[2] for i in list_opt]
  def vis_fonc(pos1, pos2):
    fill_rect(105, 200-30*pos2, 90, 2, select_col)
    fill_rect(105, 200-30*pos1, 90, 2, bkgd_col)
  def play():
    fill_rect(0,0,320,240,bkgd_col)
    draw_string(titre, 160-10*int(len(titre)/2), 40)
    draw_string("Commencer", 105, 150)
    draw_string("Options", 115, 180)
    draw_string("Quitter:<EXE>",192,202)
    fill_rect(105, 170, 90, 2, select_col)
    visible_addons()
    return move_select(2, 1, vis_fonc)
  while True:
    ch = play()
    if ch==1:return modif_opt+[True]
    elif ch==-1:return modif_opt+[False]
    elif ch==0:
      modif_opt = options(list_opt, select_col, bkgd_col)

def options(opt_list, select_col, bkgd_col):
  fill_rect(0,0,320,240,bkgd_col)
  draw_string("OPTIONS", 125, 40)
  for e in range(len(opt_list)):
    draw_string(opt_list[e][0]+" : ", 30, 110-30*e)
  draw_string("Retour au menu", 30, 170)
  def draw_choices():
    for e in range(len(opt_list)):
      fill_rect(200, 110-30*e, 140, 20, bkgd_col)
      if type(opt_list[e][2]) is bool:
        draw_string(opt_list[e][1][int(opt_list[e][2])], 200, 110-30*e)
      elif type(opt_list[e][2]) is int:
        draw_string(opt_list[e][1][opt_list[e][2]-1], 200, 110-30*e)
  def draw_selected(nb1, nb2):
    if nb1 == 0 : fill_rect(35, 190, 130, 2, bkgd_col)
    else:fill_rect(200, 160-30*nb1, 30, 2, bkgd_col)
    if nb2 == 0: fill_rect(35, 190, 130, 2, select_col)
    else:fill_rect(200, 160-30*nb2, 30, 2, select_col)
  draw_choices()
  fill_rect(35, 190, 130, 2, select_col)
  pos = move_select(len(opt_list)+1, 0, draw_selected)
  while pos != 0:
    if type(opt_list[pos-1][2]) is bool:
      opt_list[pos-1][2] = not opt_list[pos-1][2]
    elif type(opt_list[pos-1][2]) is int:
      if opt_list[pos-1][2] < len(opt_list[pos-1][1]):
        opt_list[pos-1][2] += 1
      else: opt_list[pos-1][2] = 1
    draw_choices()
    pos = move_select(len(opt_list)+1, pos, draw_selected)
  return [i[2] for i in opt_list]

def move_select(size, pos, vis_fonc):
  sleep(0.2)
  while not keydown(KEY_OK):
    if keydown(KEY_DOWN) and pos>0:
      vis_fonc(pos, pos-1)
      pos-=1
    elif keydown(KEY_UP) and pos<size-1:
      vis_fonc(pos, pos+1)
      pos+=1
    elif keydown(KEY_EXE):
      pos = -1
      break
    sleep(0.1)
  return pos