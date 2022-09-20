from random import choice
from time import *

from ion import *
from kandinsky import *

from menu import fill_screen, menu, draw_centered_string

difficulty = 3

white, black, gray = color(255, 255, 255), color(0, 0, 0), color(80, 80, 80)
red = color(220, 0, 0)
orange = color(220, 120, 0)
blue = color(0, 0, 220)
purple = color(120, 0, 220)
green = color(0, 82, 62)

select_colors = (color(18, 213, 7), color(220, 100, 0))
color_list = (red, orange, blue, purple)
letters_list = ("1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K")

table = []


class Card:
    def __init__(self, number: int | None, suit: int | None, pos: int, shown=False):
        self.number = number
        self.suit = suit
        self.pos = pos
        self.shown = shown


def set_table():
    global table
    table = [[] for i in range(13)]
    temp = []
    fill_screen(green)
    draw_card(Card(None, None, 4))
    for s in range(4):
        for n in range(13):
            temp.append(Card(n + 1, s, 5))
        draw_card(Card(None, None, s))
        fill_rect(22 + 38 * s, 5, 32, 3, color_list[s])
    for i in range(52):
        c = choice(temp)
        table[5].append(c)
        temp.remove(c)
    for j in range(6, 13):
        table[j] = table[5][:j - 5]
        for k in reversed(range(j - 5)):
            table[j][k].pos, table[j][k].shown = j, k == 0
            draw_card(table[j][k])
            sleep(0.05)
        table[5] = table[5][j - 5:]
    draw_card(table[5][0])


def get_stack_from_pos(pos, empty=True) -> list[Card]:
    return [Card(None, None, pos)] if len(table[pos]) <= 0 and not empty else table[pos]


def place_cards_there(cards, pos):
    stack = get_stack_from_pos(pos)

    def insertion():
        for i in reversed(cards):
            stack.insert(0, i)
        return True

    if 0 <= pos < 4 and len(cards) == 1 and cards[0].suit == pos:
        if (len(stack) <= 0 and cards[0].number == 1) or (len(stack) > 0 and cards[0].number == stack[0].number + 1):
            return insertion()
    elif 6 <= pos < 13:
        if len(stack) > 0:
            if cards[-1].number == stack[0].number - 1 and ((0 <= cards[-1].suit < 2 and 2 <= stack[0].suit < 4)
                                                            or (2 <= cards[-1].suit < 4 and 0 <= stack[0].suit < 2)):
                return insertion()
        elif cards[-1].number == 13:
            return insertion()
    return False


def turning_deck():
    fill_rect(189, 10, 69, 52, green)
    draw_card(Card(None, None, 4))
    if len(table[5]) > 0:
        for k in range(min(difficulty, len(table[5]))):
            table[5][0].pos, table[5][0].shown = 4, True
            table[4].insert(0, table[5][0])
            table[5].remove(table[5][0])
    else:
        while len(table[4]) > 0:
            table[4][-1].pos, table[4][-1].shown = 5, False
            table[5].append(table[4][-1])
            table[4].pop()
    for k in reversed(range(min(3, len(table[4])))):
        draw_card(table[4][k])
    draw_card(get_stack_from_pos(5, empty=False)[0], outline=select_colors[0], outline_size=3)


def get_abs_pos(card) -> tuple[int, int] | tuple[int, int, bool]:
    stack = get_stack_from_pos(card.pos, empty=False)
    card_index = 1
    if card.number is not None:
        card_index = stack.index(card)
    if 0 <= card.pos < 4:
        return 20 + card.pos * 38, 10
    elif card.pos == 4:
        return 223 - 15 * (card_index % 3), 10
    elif card.pos == 5:
        return 275, 10
    elif 13 > card.pos >= 6:
        if len(stack) < 10:
            return 15 + 42 * (card.pos - 6), 55 + 15 * max(len(stack) - card_index, 1)
        else:
            return 15 + 42 * (card.pos - 6), 60 + 10 * max(len(stack) - card_index, 1)
    elif card.pos is None:
        return -100, -100


def draw_card(card: Card, outline=black, outline_size=1, clear=False, abs_pos=None):
    if abs_pos is None:
        abs_pos = get_abs_pos(card)
    fill_rect(abs_pos[0], abs_pos[1], 35, 52, outline if not clear else green)
    if not clear:
        fill_rect(abs_pos[0] + outline_size, abs_pos[1] + outline_size, 35 - 2 * outline_size,
                  52 - 2 * outline_size, green if card.number is None else (white if card.shown else gray))
        if card.shown and card.number is not None:
            fill_rect(abs_pos[0] + 17, abs_pos[1] + 3, 15, 15, color_list[card.suit])
            fill_rect(abs_pos[0] + 3, abs_pos[1] + 34, 15, 15, color_list[card.suit])
            draw_string(letters_list[card.number - 1], abs_pos[0] + 3, abs_pos[1] + 3, black, white)


def win():
    for i in table:
        for j in i:
            if not j.shown:
                return False
    return len(table[4]) == 0


def solitaire():
    set_table()

    pos: int = 9
    selected_cards: list[Card] = []

    def clear_selection():
        for k in reversed(selected_cards):
            draw_card(k)
        if len(selected_cards) > 0 and selected_cards[0].pos == pos:
            draw_card(selected_cards[0], outline=select_colors[0], outline_size=3)
        selected_cards.clear()

    def draw_selection():
        for k in reversed(selected_cards):
            draw_card(k, outline=select_colors[1], outline_size=3)

    def draw_cursor():
        draw_card(get_stack_from_pos(pos, empty=False)[0],
                  outline=select_colors[int(len(selected_cards) > 0 and pos == selected_cards[0].pos)],
                  outline_size=3)

    draw_cursor()
    while not win():
        oldpos = pos
        if keydown(KEY_OK):
            if pos == 5:
                clear_selection()
                turning_deck()
            elif pos == 4 and len(selected_cards) <= 0 and len(table[4]) > 0:
                clear_selection()
                selected_cards.append(table[4][0])
                draw_selection()
            elif len(selected_cards) <= 0 and len(get_stack_from_pos(pos)) > 0:
                selected_cards.append(get_stack_from_pos(pos)[0])
                draw_selection()
            elif len(selected_cards) > 0:
                draw_card(get_stack_from_pos(pos, empty=False)[0], clear=True)
                if place_cards_there(selected_cards, pos):
                    old_stack_pos = selected_cards[0].pos
                    for i in selected_cards:
                        draw_card(i, clear=True)
                        i.pos = pos
                        get_stack_from_pos(old_stack_pos).remove(i)
                    for i in reversed(get_stack_from_pos(pos)):
                        draw_card(i)
                    if old_stack_pos == 4:
                        if len(table[4]) < 3:
                            fill_rect(189, 10, 69, 52, green)
                        for p in reversed(range(min(3, len(table[4])))):
                            draw_card(table[4][p])
                    else:
                        get_stack_from_pos(old_stack_pos, empty=False)[0].shown = True
                        for i in reversed(get_stack_from_pos(old_stack_pos, empty=False)):
                            draw_card(i)
                    selected_cards.clear()
                draw_cursor()
            sleep(0.1)
        elif keydown(KEY_BACKSPACE):
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
        elif keydown(KEY_RIGHT) and pos < 12:
            pos += 1
        elif keydown(KEY_LEFT) and pos > 0:
            pos -= 1
        elif keydown(KEY_UP) and 6 <= pos <= 11:
            pos -= 6
        elif keydown(KEY_DOWN) and 0 <= pos <= 5:
            pos += 6
        elif keydown(KEY_ZERO):
            solitaire()
        if oldpos != pos:
            if len(selected_cards) == 0 or oldpos != selected_cards[0].pos:
                draw_card(get_stack_from_pos(oldpos, False)[0])
            draw_cursor()
        sleep(0.1)
    draw_centered_string(" BRAVO !! ", 100, white, green)
    draw_string("REJOUER : <0>", 10, 200, white, green)
    draw_string("MENU : <ANS>", 190, 200, white, green)
    while True:
        if keydown(KEY_ANS):
            menu_sol()
        elif keydown(KEY_ZERO):
            solitaire()
        sleep(0.1)


def menu_sol():
    global difficulty

    def vis_add():
        draw_card(Card(13, 0, 1, shown=True), abs_pos=(122, 70))
        draw_card(Card(1, 3, 1, shown=True), abs_pos=(172, 70))

    list_opt = [["Difficulté", ("Facile", "Normal", "Difficile"), difficulty]]
    modif_opt = menu("SOLITAIRE", vis_add, red, green, list_opt)
    difficulty = modif_opt[0]
    if modif_opt[-1]:
        solitaire()


menu_sol()
