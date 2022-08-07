from time import sleep

from ion import *
from kandinsky import *


def menu(title, visible_addons, select_col, bkgd_col, list_opt, text_col=(0, 0, 0)):
    modif_opt = [i[2] for i in list_opt]

    def vis_fonc(pos1, pos2):
        fill_rect(125, 200 - 30 * pos2, 80, 2, select_col)
        fill_rect(125, 200 - 30 * pos1, 80, 2, bkgd_col)

    def play():
        fill_screen(bkgd_col)
        draw_centered_string(title, 40, text_col, bkgd_col)
        draw_centered_string("Commencer", 150, text_col, bkgd_col)
        draw_centered_string("Options", 180, text_col, bkgd_col)
        draw_string("Quitter:<EXE>", 192, 202, text_col, bkgd_col)
        fill_rect(125, 170, 80, 2, select_col)
        visible_addons()
        return move_select(2, 1, vis_fonc)

    while True:
        ch = play()
        if ch == 1 or ch == -1:
            fill_screen(bkgd_col)
            return modif_opt + [ch > 0]
        elif ch == 0:
            modif_opt = options(list_opt, select_col, bkgd_col, text_col)


def options(olist, select_col, bkgd_col, text_col):
    fill_rect(0, 0, 320, 240, bkgd_col)
    draw_centered_string("OPTIONS", 10, text_col, bkgd_col)
    firsty = 130 - 20 * (4 - len(olist))
    for e in range(len(olist)):
        draw_string(olist[e][0] + " : ", 30, firsty - 30 * e, text_col, bkgd_col)
    draw_string("Retour au menu", 30, 170, text_col, bkgd_col)

    def draw_choices():
        for e in range(len(olist)):
            opt = olist[e]
            fill_rect(200, firsty + 30 * e, 140, 20, bkgd_col)
            if type(opt[2]) is bool:
                draw_string(opt[1][int(opt[2])], 200, firsty - 30 * e, text_col, bkgd_col)
            elif type(opt[2]) is int:
                draw_string(opt[1][opt[2] - 1], 200, firsty - 30 * e, text_col, bkgd_col)

    def draw_selected(last, new):
        if last == 0:
            fill_rect(35, 190, 130, 2, bkgd_col)
        else:
            fill_rect(200, firsty + 50 - 30 * last, 30, 2, bkgd_col)
        if new == 0:
            fill_rect(35, 190, 130, 2, select_col)
        else:
            fill_rect(200, firsty + 50 - 30 * new, 30, 2, select_col)

    draw_choices()
    fill_rect(35, 190, 130, 2, select_col)
    pos = move_select(len(olist) + 1, 0, draw_selected)
    while pos > 0:
        opt = olist[pos - 1]
        if type(opt[2]) is bool:
            fill_rect(200, firsty - 30 * (pos - 1), len(opt[1][int(opt[2])]) * 15, 18, bkgd_col)
            opt[2] = not opt[2]
        elif type(opt[2]) is int:
            fill_rect(200, firsty - 30 * (pos - 1), len(opt[1][opt[2] - 1]) * 15, 18, bkgd_col)
            if opt[2] < len(opt[1]):
                opt[2] += 1
            else:
                opt[2] = 1
        draw_choices()
        pos = move_select(len(olist) + 1, pos, draw_selected)
    return [i[2] for i in olist]


def move_select(size, pos, vis_fonc):
    sleep(0.2)
    while not keydown(KEY_OK):
        if (keydown(KEY_DOWN) or keydown(KEY_LEFT)) and pos > 0:
            vis_fonc(pos, pos - 1)
            pos -= 1
        elif (keydown(KEY_UP) or keydown(KEY_RIGHT)) and pos < size - 1:
            vis_fonc(pos, pos + 1)
            pos += 1
        elif keydown(KEY_EXE):
            pos = -1
            break
        sleep(0.1)
    return pos


def draw_centered_string(text, posy, color=(0, 0, 0), background=(255, 255, 255)):
    draw_string(text, 159 - 10 * int(len(text) / 2), posy, color, background)


def fill_screen(color):
    fill_rect(0, 0, 320, 240, color)
