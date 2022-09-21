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

board = []


def draw_case(pos):
    fill_rect(pos[0] * 22 + 72, pos[1] * 22 + 22, 22, 22, get_box_color(pos))


def get_box_color(pos):
    if pos[0] % 2 + pos[1] % 2 == 1:
        return 225, 225, 225
    else:
        return 175, 175, 175


def draw_piece(pos, n):
    """n being the type and color of the piece, as the index in this list : [pawn, rook, knight, bishop, queen, king]
    and 0 for white and 1 for black (everything in a tuple)"""
    lp = ["P", "R", "N", "B", "Q", "K"]
    draw_string(lp[n[0]], pos[0] * 22 + 78, pos[1] * 22 + 24, black if n[1] else green, get_box_color(pos))


def move_piece(oldpos, newpos):
    global board
    piece = board[oldpos[0]][oldpos[1]]
    board[oldpos[0]][oldpos[1]] = None
    print(piece)
    draw_case(oldpos)
    board[newpos[0]][newpos[1]] = piece
    draw_piece(newpos, piece)


def create_initial_board():
    global board
    for i in range(8):
        board.append([])
        for j in range(8):
            board[i].append(None)
    fill_rect(0, 0, 320, 222, green)
    fill_rect(70, 20, 180, 180, black)
    for i in range(8):
        draw_string(str(i + 1), i * 22 + 78, 2, black, green)
        draw_string(str(i + 1), i * 22 + 78, 202, black, green)
        for j in range(8):
            draw_case((i, j))
    ltrs = ["A", "B", "C", "D", "E", "F", "G", "H"]
    for i in range(8):
        draw_string(ltrs[i], 56, i * 22 + 24, black, green)
        draw_string(ltrs[i], 254, i * 22 + 24, black, green)

    start_pos = [[1, 2, 3, 4, 5, 3, 2, 1], [0, 0, 0, 0, 0, 0, 0, 0]]
    for i in range(len(start_pos)):
        for j in range(len(start_pos[i])):
            piece1 = (start_pos[i][j], 0)
            piece2 = (start_pos[i][j], 1)
            draw_piece((i, j), piece1)
            draw_piece((7 - i, j), piece2)
            board[i][j] = piece1
            board[7-i][j] = piece2


def selection(pos):
    pass


def chess():
    create_initial_board()
    move_piece((1, 2), (3, 2))


def menu_chess():
    def vis_add():
        pass

    list_opt = []
    modif_opt = menu("CHESS", vis_add, black, green, list_opt, black)
    if modif_opt[-1]: chess()


menu_chess()
display()
