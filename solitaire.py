from random import choice
from time import *

from ion import *
from kandinsky import *

from menu import fill_screen, menu

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

select_colors = (color(18, 213, 7), color(12, 199, 150))

color_list = (red, orange, blue, purple)
letters_list = ("1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K")

placed_cards = [[], [], [], [], [], [], []]
final_stacks = [[], [], [], []]
deck = [[], 0]


# TECHNIQUE #


class Card:
    def __init__(self, number: int | None, suit: int | None, pos: int, shown=False):
        self.number = number
        self.suit = suit
        self.pos = pos
        self.shown = shown


def set_table():
    global deck, placed_cards
    temp = []
    for s in range(4):  # creates deck
        for n in range(13):
            temp.append(Card(n + 1, s + 1, 6))
    for i in range(52):  # randomizing
        c = choice(temp)
        deck[0].append(c)
        temp.remove(c)
    for j in range(7):  # placing table
        placed_cards[j] = deck[0][:j + 1]
        for k in range(j + 1):
            placed_cards[j][k].pos = j + 7
        placed_cards[j][0].shown = True
        deck[0] = deck[0][j + 1:]


def get_stack_from_pos(pos, empty=True) -> list[Card]:
    if 0 < pos < 5:
        if len(final_stacks[pos - 1]) <= 0 and not empty:
            return [Card(None, None, pos)]
        return final_stacks[pos - 1]
    elif pos == 5:
        if deck[1] > 0:
            return deck[0][:deck[1]]  # TODO
        else:
            return [Card(None, None, pos)]
    elif 6 < pos <= 13:
        if len(placed_cards[pos - 7]) <= 0 and not empty:
            return [Card(None, None, pos)]
        return placed_cards[pos - 7]
    else:
        return deck[0][deck[1]:]


def win():
    return len(deck) <= 0


def place_cards_there(cards, pos):
    stack = get_stack_from_pos(pos)
    if 0 < pos <= 4 and len(cards) <= 1:
        if len(stack) <= 0 and cards[0].number == 1 and cards[0].suit == pos:
            stack.insert(0, cards[0])
            return True
        elif len(stack) > 0 and cards[0].number == stack[-1].number + 1:
            stack.insert(0, cards[0])
            return True
    elif 7 <= pos <= 13:
        if len(stack) > 0:
            if cards[-1].number == stack[0].number - 1:
                if 0 < cards[-1].suit <= 2 and 3 <= stack[0].suit <= 4 \
                        or 3 <= cards[-1].suit <= 4 and 0 < stack[0].suit <= 2:
                    for i in reversed(cards):
                        stack.insert(0, i)
                    return True
        elif cards[-1].number == 13:
            for i in reversed(cards):
                stack.insert(0, i)
            return True
    return False


# GRAPHIQUE #


def draw_table():
    fill_screen(green)
    for i in placed_cards:
        for j in reversed(i):
            draw_card(j)
    for i in range(4):
        draw_card(Card(None, None, i + 1))
        fill_rect(22 + 38 * i, 5, 32, 3, color_list[i])
    draw_card(Card(None, None, 5))
    draw_card(deck[0][0])


def get_abs_pos(card) -> tuple[int, int]:
    stack = get_stack_from_pos(card.pos)
    card_index = 1
    if card.number is not None:
        card_index = stack.index(card)
    if 0 < card.pos <= 4:
        return 20 + (card.pos - 1) * 38, 10
    elif card.pos == 5:
        return 223, 10
    elif card.pos == 6:
        return 275, 10
    elif 13 >= card.pos > 6:
        return 15 + 42 * (card.pos - 7), 55 + 15 * max(len(stack) - card_index, 1)
    elif card.pos is None:
        return -100, -100


def draw_card(card: Card, outline=black, outline_size=1, clear=False, abs_pos=None):
    if abs_pos is None:
        abs_pos = get_abs_pos(card)
    if not clear:
        fill_rect(abs_pos[0], abs_pos[1], 35, 52, outline)
        if card.number is not None:
            if card.shown:
                fill_rect(abs_pos[0] + outline_size, abs_pos[1] + outline_size, 35 - 2 * outline_size,
                          52 - 2 * outline_size, white)
                fill_rect(abs_pos[0] + 17, abs_pos[1] + 3, 15, 15, color_list[card.suit - 1])
                fill_rect(abs_pos[0] + 3, abs_pos[1] + 34, 15, 15, color_list[card.suit - 1])
                draw_string(letters_list[card.number - 1], abs_pos[0] + 3, abs_pos[1] + 2, black, white)
            else:
                fill_rect(abs_pos[0] + outline_size, abs_pos[1] + outline_size, 35 - 2 * outline_size,
                          52 - 2 * outline_size, gray)
        else:
            fill_rect(abs_pos[0] + outline_size, abs_pos[1] + outline_size, 35 - 2 * outline_size,
                      52 - 2 * outline_size, green)
    else:
        fill_rect(abs_pos[0], abs_pos[1], 35, 52, green)


# MAIN #


def solitaire():
    set_table()
    draw_table()

    pos: int = 10
    selected_cards: list[Card] = []

    def clear_selection():
        for k in reversed(selected_cards):
            draw_card(k)
        selected_cards.clear()

    def draw_selection():
        for k in reversed(selected_cards):
            draw_card(k, outline=select_colors[1], outline_size=3)

    while not win():
        oldpos = pos
        if keydown(KEY_OK):
            if pos == 6:
                clear_selection()
                # TODO : turning the deck lol
            elif pos == 5:
                clear_selection()
                # TODO : récupérer la bonne carte
            elif len(selected_cards) <= 0 and len(get_stack_from_pos(pos)) > 0:
                selected_cards.append(get_stack_from_pos(pos)[0])
                draw_selection()
            elif len(selected_cards) > 0 and place_cards_there(selected_cards, pos):
                old_stack_pos = selected_cards[0].pos
                old_stack = get_stack_from_pos(old_stack_pos)
                for i in selected_cards:
                    draw_card(i, clear=True)
                    old_stack.remove(i)
                    i.pos = pos
                if len(old_stack) == 0:
                    old_stack.append(Card(None, None, old_stack_pos))
                old_stack[0].shown = True
                for i in reversed(old_stack):
                    draw_card(i)
                if old_stack[0].number is None:
                    old_stack.clear()
                for i in reversed(get_stack_from_pos(pos)):
                    if i.shown:
                        draw_card(i)
                selected_cards.clear()
            sleep(0.1)
        elif keydown(KEY_EXE):
            clear_selection()
        elif keydown(KEY_ALPHA) and len(selected_cards) > 1:
            draw_card(selected_cards[-1])
            selected_cards.pop()
            draw_selection()
        elif keydown(KEY_SHIFT) and len(get_stack_from_pos(pos)) > len(selected_cards) > 0:
            card = get_stack_from_pos(pos)[len(selected_cards)]
            if card.shown:
                selected_cards.append(get_stack_from_pos(pos)[len(selected_cards)])
                draw_selection()
        elif keydown(KEY_RIGHT) and pos < 13:
            pos += 1
        elif keydown(KEY_LEFT) and pos > 1:
            pos -= 1
        elif keydown(KEY_UP) and 7 <= pos <= 12:
            pos -= 6
        elif keydown(KEY_DOWN) and 1 <= pos <= 6:
            pos += 6
        if oldpos != pos:
            if len(selected_cards) == 0 or oldpos != selected_cards[0].pos:
                draw_card(get_stack_from_pos(oldpos, False)[0])
            draw_card(get_stack_from_pos(pos, False)[0],
                      outline=select_colors[int(len(selected_cards) > 0 and pos == selected_cards[0].pos)],
                      outline_size=3)
        sleep(0.1)


def menu_sol():
    global darkMode, difficulty, black, white, gray

    def vis_add():
        draw_card(Card(13, 1, 1, shown=True), abs_pos=(122, 70))
        draw_card(Card(1, 4, 1, shown=True), abs_pos=(172, 70))

    list_opt = [["Mode sombre", ("Non", "Oui"), darkMode],
                ["Difficulté", ("Facile", "Normal", "Difficile"), difficulty]]
    modif_opt = menu("SOLITAIRE", vis_add, red, green, list_opt)
    if modif_opt[0] != darkMode: white, black, gray = gray, white, black
    darkMode = modif_opt[0]
    difficulty = modif_opt[1]
    if modif_opt[-1]:
        solitaire()


menu_sol()
