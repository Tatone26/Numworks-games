from math import *
from kandinsky import *
from ion import *
from random import *
from menu import menu, fill_screen
from time import *
xg = 120
yg = 20
size = 4
pts = 0
best = 26372
lbx = []


def dw_grid():
    fill_rect(120, 20, 185, 185, (180, 180, 180))
    for i in range(1, size + 1):
        for e in range(1, size + 1): dw_no_box(i, e)


def dw_no_box(x, y):
    rs = int(185 / size)
    fill_rect(xg + 1 + rs * (x - 1), yg + 1 + rs * (y - 1), rs - 1, rs - 1, (220, 220, 220))


def dw_box(box):
    rs = int(185 / size)
    c = (250 - box[2] * 10, 250 - box[2] * 20, 250 - box[2] * 20)
    fill_rect(xg + 1 + rs * (box[0] - 1), yg + 1 + rs * (box[1] - 1), rs - 1, rs - 1,
              (250 - box[2] * 10, 250 - box[2] * 20, 250 - box[2] * 20))
    draw_string(str(2 ** box[2]), xg + int(rs / 2) + rs * (box[0] - 1) - int(len(str(2 ** box[2])) / 2 * 8),
                yg + int(rs / 2) - 8 + rs * (box[1] - 1), (0, 0, 0), c)


def draw_pts():
    fill_rect(30, 40, 50, 70, (255, 255, 255))
    draw_string(str(pts), 35, 45)


def pinput():
    while True:
        if keydown(KEY_UP):
            return [0, -1]
        elif keydown(KEY_DOWN):
            return [0, 1]
        elif keydown(KEY_RIGHT):
            return [1, 0]
        elif keydown(KEY_LEFT):
            return [-1, 0]


def add_box():
    global lbx
    if randint(1, 9) == 9:
        pw = 2
    else:
        pw = 1
    if len(lbx) < size * size:
        placed = False
        while not placed:
            nwbx = (randint(1, size), randint(1, size), pw)
            fullpos = [[x[0], x[1]] for x in lbx]
            if [nwbx[0], nwbx[1]] not in fullpos:
                lbx.append(nwbx)
                dw_box(nwbx)
                placed = True
        return True
    return False


def move_boxes(drct):
    global lbx
    moved = True
    times_moved = 0
    while moved:
        moved = False
        full_pos = [[x[0], x[1]] for x in lbx]
        new_lbx = []
        for b in lbx:
            if 1 <= b[0] + drct[0] <= size and 1 <= b[1] + drct[1] <= size and [b[0] + drct[0],
                                                                                b[1] + drct[1]] not in full_pos:
                new_lbx.append((b[0] + drct[0], b[1] + drct[1], b[2]))
                dw_no_box(b[0], b[1])
                dw_box(new_lbx[-1])
                moved = True
            else:
                new_lbx.append(b)
        lbx = new_lbx.copy()
        if moved: times_moved += 1
    if times_moved >= 1:
        return True
    else:
        return False


def fuse_boxes(drct):
    global lbx, pts
    new_lbx = []
    used = []
    n = int(drct[0] == 0)
    lbx.sort(key=lambda x: x[int(not bool(n))])
    lbx.sort(key=lambda x: x[n])
    if sum(drct) < 0:
        it = range(len(lbx))
    else:
        it = range(len(lbx) - 1, -1, -1)
    for i in it:
        b = lbx[i]
        if b not in used and (b[0] - drct[0], b[1] - drct[1], b[2]) in lbx and (
                b[0] - drct[0], b[1] - drct[1], b[2]) not in used:
            new_lbx.append((b[0], b[1], b[2] + 1))
            dw_box(new_lbx[-1])
            used.append(b)
            dw_no_box(b[0] - drct[0], b[1] - drct[1])
            used.append((b[0] - drct[0], b[1] - drct[1], b[2]))
            pts += 2 ** (b[2] + 1)
    for b in lbx:
        if b not in used:
            new_lbx.append(b)
    lbx = new_lbx.copy()
    return len(used) > 0


def is_dead():
    for b in lbx:
        if len(lbx) < size * size or (b[0] + 1, b[1], b[2]) in lbx or (b[0] - 1, b[1], b[2]) in lbx or (
                b[0], b[1] + 1, b[2]) in lbx or (b[0], b[1] - 1, b[2]) in lbx:
            return False
    return True


def game():
    global pts, lbx
    fill_screen((255, 255, 255))
    dw_grid()
    draw_string("High Score:\n    " + str(best), 5, 180)
    draw_string("Points :", 5, 20)
    draw_pts()
    add_box()
    add_box()
    for i in lbx: dw_box(i)
    dead = False
    while not dead:
        drct = pinput()
        moved = move_boxes(drct)
        sleep(0.08)  # anim
        fused = fuse_boxes(drct)
        if fused:
            draw_pts()
            sleep(0.08)  # anim
            move_boxes(drct)
        if moved or fused:
            add_box()
            if is_dead():
                dead = True
        sleep(0.25)
    replay = False
    draw_string("You Lost !", xg + 70, yg + 15)
    draw_string("Replay : <OK>", xg + 5, yg + 115)
    draw_string("Menu : <EXE>", xg + 5, yg + 160)
    if pts > best: draw_string("New \n High Score !", 5, 100)
    print("Score : " + str(pts))
    while not keydown(KEY_EXE):
        if keydown(KEY_OK):
            replay = True
            break
    pts = 0
    lbx.clear()
    if replay:
        game()
    else:
        menu2048()


def menu2048():
    global size

    def va():
        fill_rect(0, 75, 320, 20, (230, 180, 180))

    opt = menu("2048", va, (0, 0, 0), (255, 255, 255),
               [["Size", ("3x3", "4x4", "5x5", "6x6", "7x7", "8x8"), size - 2]])
    size = opt[0] + 2
    if opt[-1]: game()


menu2048()