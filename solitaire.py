from random import shuffle
from kandinsky import *
from ion import *
from time import *

from menu import menu, fill_screen, move_select

difficulty = 3
darkMode = False

white = color(255, 255, 255)
black = color(0, 0, 0)
gray = color(100, 100, 100)

red = color(200, 0, 0)
orange = color(255, 144, 0)
blue = color(0, 0, 200)
purple = color(120, 0, 200)
green = color(0, 82, 62)

color_list = (red, orange, blue, purple)
letters_list = ("1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K")

placed_cards = []
deck = []

class Card:
    def __init__(self, number, suit):
        self.suit = suit
        self.color = color_list[suit - 1]
        self.number = number
        self.letter = letters_list[number - 1]


def draw_card(card, pos, hidden=False):
    fill_rect(pos[0], pos[1], 35, 52, black)
    if not hidden:
        fill_rect(pos[0] + 1, pos[1] + 1, 33, 50, white)
        fill_rect(pos[0] + 17, pos[1] + 3, 15, 15, card.color)
        fill_rect(pos[0] + 3, pos[1] + 34, 15, 15, card.color)
        draw_string(card.letter, pos[0] + 3, pos[1] + 2)
        return True
    else:
        fill_rect(pos[0] + 1, pos[1] + 1, 33, 50, gray)
        return False


def clear_card(pos):
    fill_rect(pos[0], pos[1], 35, 52, green)


def draw_start():
    for i in range(4):
        fill_rect(20+i*38, 10, 35, 52, black)
        fill_rect(21+i*38, 11, 33, 50, green)
        fill_rect(30+i*38, 27, 15, 15, color_list[i])
    for j in placed_cards:
        for c in range(len(j)):
            draw_card(j[c], (15+42*(len(j)-1), 70+15*c), c < len(j)-1)
    draw_card(deck[difficulty], (275, 10), True)
    print(deck[0].letter)
    print(deck[2].letter)
    for k in range(difficulty):
        draw_card(deck[k], (240+15*(k-difficulty), 10))


def create_deck():
    global deck
    for s in range(4):
        for n in range(13):
            deck.append(Card(n + 1, s + 1))
    shuffle(deck)


def solitaire():
    global placed_cards, deck
    fill_screen(green)
    create_deck()
    for i in range(7):
        placed_cards.append([deck[f] for f in range(i+1)])
        deck = deck[i+1:]
    draw_start()
    display()


def menu_sol():
    global darkMode, difficulty, black, white

    def vis_add():
        draw_card(Card(13, 1), (122, 70))
        draw_card(Card(1, 4), (172, 70))

    list_opt = [("Mode sombre", ("Non", "Oui"), darkMode),
                ("Difficulté", ("Facile", "Normal", "Difficile"), difficulty)]
    modif_opt = menu("SOLITAIRE", vis_add, red, white, list_opt)
    if modif_opt[0] != darkMode: white, black = black, white
    darkMode = modif_opt[0]
    difficulty = modif_opt[1]
    if modif_opt[-1]: solitaire()


menu_sol()
