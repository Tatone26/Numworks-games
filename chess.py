from kandinsky import *
from ion import *
from menu import *
from time import *

white = color(255, 255, 255)
black = color(0, 0, 0)
green = color(0, 82, 62)
"""
-> stocker les positions de chaque pièce, même celles sorties du terrain
-> afficher le plateau, uniquement les pièces qui ont bougé (deux cases à chaque tour du coup)
-> afficher les coups possibles de chaque pion (ne pas oublier d'effacer cet affichage)
-> prendre en charge roque et en passant
-> faire confiance aux joueurs pour le mat ? 
"""

board = [[None]*8]*8
print(len(board[7]))

def draw_case(pos):
    fill_rect(pos[0] * 22 + 72, pos[1] * 22 + 22, 22, 22, get_box_color(pos))


def draw_piece(pos, n):
    """n being the type of piece, as the index in this list : [pawn, rook, knight, bishop, queen, king]"""
    lp = ["P", "R", "N", "B", "Q", "K"]
    draw_string(lp[n], pos[0] * 22 + 78, pos[1] * 22 + 24, black, get_box_color(pos))


def get_box_color(pos):
    if pos[0] % 2 + pos[1] % 2 == 1:
        return 225, 225, 225
    else:
        return 175, 175, 175


def chess():
    fill_rect(0, 0, 320, 222, green)
    fill_rect(70, 20, 180, 180, black)
    for i in range(8):
        draw_string(str(i+1), i*22 + 78, 2, black, green)
        draw_string(str(i+1), i*22 + 78, 202, black, green)
        for j in range(8):
            draw_case((i, j))
    ltrs = ["A", "B", "C", "D", "E", "F", "G", "H"]
    for i in range(8):
        draw_string(ltrs[i], 56, i*22+24, black, green)

    start_pos = [[1, 2, 3, 4, 5, 3, 2, 1], [0, 0, 0, 0, 0, 0, 0, 0]]
    for i in range(len(start_pos)):
        for j in range(len(start_pos[i])):
            draw_piece((i, j), start_pos[i][j])
            draw_piece((7-i, j), start_pos[i][j])


def menu_chess():
    def vis_add():
        pass

    list_opt = []
    modif_opt = menu("CHESS", vis_add, black, green, list_opt, black)
    if modif_opt[-1]: chess()


menu_chess()
display()
