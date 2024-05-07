from kandinsky import *
from ion import *
from time import *
from menu import menu, draw_centered_string, fill_screen

white = color(255, 255, 255)
black = color(0, 0, 0)
darkMode = False
gpos = ((20, 30), (170, 50))


class Player:
    def __init__(self):
        self.listBoats = []
        self.listTirs = []
        self.lastSel = [0, 0]


def color_case_grid(grid, pos, c):
    fill_rect(gpos[grid][0] + 1 + 14 * pos[0], gpos[grid][1] + 1 + 14 * pos[1], 10, 10, c)


def print_grid(grid):
    posx = gpos[grid][0]
    posy = gpos[grid][1]
    fill_rect(posx - 1, posy - 1, 126, 126, black)
    fill_rect(posx, posy, 124, 124, white)
    for x in range(8): fill_rect(posx + 12 + 14 * x, posy, 2, 124, black)
    for y in range(8): fill_rect(posx, posy + 12 + 14 * y, 124, 2, black)


def print_shot(grid, player, tir):
    c = white
    if tir + [0] in player.listTirs:
        c = (250, 250, 0)
    elif tir + [1] in player.listTirs:
        c = (250, 0, 0)
    color_case_grid(grid, tir, c)


def print_boat(grid, boat):
    for c in boat:
        color_case_grid(grid, c[:2], (250, abs(c[2] - 1) * 150, 0))


def select_pos(grid, sinit=(0, 0), opt=None, player=None):  # select initial, griPos, opt = [size, orient]

    if opt is None:
        opt = [5, (1, 0)]

    def print_cho():
        for i in range(opt[0]):
            color_case_grid(grid, [select[0] + 1 * i * opt[1][0], select[1] + 1 * i * opt[1][1]], (250, 0, 250))

    def clear_cho():
        for i in range(opt[0]):
            s = [select[0] + 1 * i * opt[1][0], select[1] + 1 * i * opt[1][1]]
            if player is None:
                color_case_grid(grid, s, white)
            else:
                print_shot(grid, player, s)

    def replace():
        if select[1] > 9 - opt[0] and opt[1] == (0, 1):
            select[1] = 9 - opt[0]
        elif select[0] > 9 - opt[0] and opt[1] == (1, 0):
            select[0] = 9 - opt[0]

    select = sinit
    replace()
    print_cho()
    sleep(0.5)
    while True:

        old_sel = select[:]
        turned = False
        if keydown(KEY_OK):
            clear_cho()
            return select, opt
        elif keydown(KEY_EXE):
            clear_cho()
            opt[1] = (opt[1][1], opt[1][0])
            replace()
            print_cho()
            turned = True

        elif keydown(KEY_RIGHT) and (select[0] < 9 - ((opt[0] - 1) * opt[1][0] + 1)):
            select[0] += 1
        elif keydown(KEY_LEFT) and (select[0] > 0):
            select[0] -= 1
        elif keydown(KEY_DOWN) and (select[1] < 9 - ((opt[0] - 1) * opt[1][1] + 1)):
            select[1] += 1
        elif keydown(KEY_UP) and (select[1] > 0):
            select[1] -= 1

        if old_sel != select and not turned:
            temp_sel = select[:]
            select = old_sel
            clear_cho()
            select = temp_sel[:]
            print_cho()
        sleep(0.05)


def change_screen(text, next_player):
    fill_screen(white)
    draw_centered_string("PLAYER " + str(next_player), 75, black, white)
    draw_centered_string(text, 100, black, white)
    draw_centered_string("Press <EXE> to continue.", 200, black, white)
    while not keydown(KEY_EXE):
        sleep(0.2)
    fill_screen(white)


def place_boats(player):
    print_grid(0)
    print_grid(1)
    sizes = [5, 4, 3, 3, 2]
    i = 0
    while i < len(sizes):
        opt = [sizes[i], (1, 0)]
        player.lastSel, opt = select_pos(0, player.lastSel[:], opt)
        boat = [[player.lastSel[0] + i * opt[1][0], player.lastSel[1] + i * opt[1][1], 0] for i in range(opt[0])]
        verif = False
        for b in player.listBoats:
            for c in boat:
                if c in b: verif = True
        if not verif:
            player.listBoats.append(
                [[player.lastSel[0] + i * opt[1][0], player.lastSel[1] + i * opt[1][1], 0] for i in range(opt[0])])
            print_boat(1, player.listBoats[-1])
            i += 1


def turn(player, target):
    print_grid(0)
    draw_string("V--TARGET--V", 23, 2, black, white)
    print_grid(1)
    draw_string("^-YOUR BOATS-^", 155, 185, black, white)
    for b in player.listBoats: print_boat(1, b)
    for t in player.listTirs: print_shot(0, player, [t[0], t[1]])
    while True:
        pos: list
        pos, o = select_pos(0, player.lastSel, [1, (1, 1)], player)
        player.lastSel = pos[:]
        if pos in [[i[0], i[1]] for i in player.listTirs]:
            continue
        elif pos + [1] in [i for b in target.listBoats for i in b]:
            continue
        elif pos + [0] not in [i for b in target.listBoats for i in b]:
            player.listTirs.append(pos + [0])
            draw_string("Missed...", 10, 175, black, white)
            sleep(1)
            break
        else:
            player.listTirs.append(pos + [1])
            draw_string("Hit!", 10, 175)
            for b in target.listBoats:
                if pos + [0] in b:
                    b[b.index(pos + [0])][2] = 1
                    if False not in [i[2] == 1 for i in b]:
                        draw_string("Hit and sunk!", 10, 175, black, white)
            sleep(1)
            break


def game():
    p1 = Player()
    p2 = Player()
    change_screen("is going to place his boats.", 1)
    place_boats(p1)
    change_screen("is going to place his boats.", 2)
    place_boats(p2)
    while True:
        change_screen("is going to attack.", 1)
        turn(p1, p2)
        if False not in [i[2] == 1 for b in p2.listBoats for i in b]:
            win = 1
            break
        change_screen("is going to attack.", 2)
        turn(p2, p1)
        if False not in [i[2] == 1 for b in p1.listBoats for i in b]:
            win = 2
            break
    change_screen("WON", win)
    menu_bn()


def menu_bn():
    global darkMode, white, black

    def vis_add():
        print_boat(0, [[8, 5, 0], [9, 5, 0], [10, 5, 1], [11, 5, 0], [12, 5, 0]])

    mod_opt = menu("BATTLESHIP", vis_add, (250, 150, 0), white, [["Dark mode", ("No", "Yes"), darkMode]], black)
    if darkMode != mod_opt[0]:
        darkMode = mod_opt[0]
        white, black = black, white
    if mod_opt[-1]: game()


menu_bn()
