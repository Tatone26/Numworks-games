from menu import *

difficulty = 3
darkMode = False

white = color(255, 255, 255)
black = color(0, 0, 0)

red = color(200, 0, 0)
orange = color(255, 144, 0)
blue = color(0, 0, 200)
purple = color(120, 0, 200)
green = color(0, 82, 62)


class Card:
    def __init__(self, color, number):
        self.color = color
        self.number = number


def draw_card(card, pos):
    fill_rect(pos[0], pos[1], 35, 52, black)
    fill_rect(pos[0] + 1, pos[1] + 1, 33, 50, white)
    draw_string(str(card.color), pos[0]+1, pos[1]+1)
    fill_rect()
    display()

def solitaire():
    c = Card(1, 1)
    draw_card(c, (50, 50))


def menu_sol():
    global darkMode, difficulty, black, white

    def vis_add():
        pass

    list_opt = [["Mode sombre", ("Non", "Oui"), darkMode],
                ["Difficulté", ("Facile", "Normal", "Difficile"), difficulty]]
    modif_opt = menu("SOLITAIRE", vis_add, red, white, list_opt)
    if modif_opt[0] != darkMode: white, black = black, white
    darkMode = modif_opt[0]
    difficulty = modif_opt[1]
    if modif_opt[-1]: solitaire()


menu_sol()
