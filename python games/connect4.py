from kandinsky import *
from ion import *
from time import *
from menu import menu, draw_centered_string, fill_screen, move_select

white = color(255, 255, 255)
black = color(0, 0, 0)
blue = color(0, 0, 255)
red = color(255, 0, 0)
green = color(0, 255, 0)
placedCoins = []

darkMode = False
visGrid = True
visChoice = True
title = "CONNECT 4"
nbPlayers = 2


def check():
    if hAlign() or vAlign() or dhAlign() or dbAlign():
        return True
    return False


def hAlign():
    for x in range(5):
        for y in range(6):
            co = placedCoins[x][y]
            if co is not None and placedCoins[x + 1][y] == co and placedCoins[x + 2][y] == co and (
                    nbPlayers >= 3 or (x <= 3 and placedCoins[x + 3][y] == co)):
                return True
    return False


def vAlign():
    for x in range(7):
        for y in range(4):
            co = placedCoins[x][y]
            if co is not None and placedCoins[x][y + 1] == co and placedCoins[x][y + 2] == co and (
                    nbPlayers >= 3 or (y <= 2 and placedCoins[x][y + 3] == co)):
                return True
    return False


def dhAlign():
    for x in range(5):
        for y in range(4):
            co = placedCoins[x][y]
            if co is not None and placedCoins[x + 1][y + 1] == co and placedCoins[x + 2][y + 2] == co and (
                    nbPlayers >= 3 or (x <= 3 and y <= 2 and placedCoins[x + 3][y + 3] == co)):
                return True
    return False


def dbAlign():
    for x in range(5):
        for y in range(2, 6):
            co = placedCoins[x][y]
            if co is not None and placedCoins[x + 1][y - 1] == co and placedCoins[x + 2][y - 2] == co and (
                    nbPlayers >= 3 or (x <= 3 <= y and placedCoins[x + 3][y - 3] == co)):
                return True
    return False


def place_coin(posx, color):
    h = len([f for f in placedCoins[posx] if f is not None])
    placedCoins[posx][h] = color
    return h


def print_grid():
    fill_rect(50, 50, 212, 170, black)
    for x in range(7):
        fill_rect(52 + x * 30, 50, 28, 168, white)
        for y in range(6):
            if not darkMode:
                print_coin(x, y, (240, 240, 240))
            else:
                print_coin(x, y, (30, 30, 30))


def print_coin(posx, posy, color): fill_rect(53 + 30 * posx, 191 - 28 * posy, 26, 26, color)


def clear_coin(pos): fill_rect(53 + 30 * pos, 23, 26, 26, white)


def select_pos_coin(color):
    def vis_function(pos, new_pos):
        clear_coin(pos)
        if visChoice: print_coin(new_pos, 6, color)
    fill_rect(0, 0, 350, 10, color)
    if visChoice: print_coin(3, 6, color)
    select = move_select(7, 3, vis_function)
    if select < 0 or len([f for f in placedCoins[select] if f is not None]) > 5:
        fill_rect(53, 23, 270, 26, white)
        select_pos_coin(color)
    else:
        h = place_coin(select, color)
        if visGrid: print_coin(select, h, color)
        clear_coin(select)
        sleep(0.3)


def p4():
    global placedCoins
    fill_screen(white)
    placedCoins = [[None] * 6 for k in range(0, 7)]
    if visGrid: print_grid()
    sleep(0.4)
    while not check():
        select_pos_coin(blue)
        if check(): break
        select_pos_coin(red)
        if check(): break
        if nbPlayers >= 3:
            select_pos_coin(green)
        if check(): break
    print_grid()
    for x in range(7):
        for y in range(6):
            if placedCoins[x][y] is not None:
                print_coin(x, y, placedCoins[x][y])
    draw_centered_string(title + " !", 100, black, white)
    draw_string("MENU : EXE", 180, 200, black, white)
    draw_string("REMATCH : OK", 20, 200, black, white)
    while not keydown(KEY_EXE) and not keydown(KEY_OK):
        pass
    if keydown(KEY_EXE):
        menu_p4()
    else:
        p4()


def menu_p4():
    global darkMode, white, black, visGrid, visChoice, nbPlayers, title

    def vis_add():
        print_coin(2, 4, red)
        print_coin(4, 4, blue)
        if nbPlayers >= 3:
            print_coin(3, 4, green)

    list_opt = [["Dark Mode", ("No", "Yes"), darkMode], ["Visible Grid", ("No", "Yes"), visGrid],
                ["Visible choice", ("No", "Yes"), visChoice], ["Nb Players", ("2", "3"), nbPlayers - 1]]
    modif_opt = menu(title, vis_add, blue, white, list_opt, black)
    if modif_opt[0] != darkMode: white, black = black, white
    darkMode = modif_opt[0]
    visGrid = modif_opt[1]
    visChoice = modif_opt[2]
    nbPlayers = modif_opt[3] + 1
    title = "CONNECT " + str(6 - nbPlayers)
    if modif_opt[-1]: p4()


menu_p4()