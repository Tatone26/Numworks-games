from kandinsky import *
from ion import *
from menu import menu, fill_screen
from time import *

white = color(255, 255, 255)
black = color(0, 0, 0)
green = color(0, 82, 62)

board = []


def get_box_color(pos):
    if pos[0] % 2 + pos[1] % 2 == 1:
        return 225, 225, 225
    else:
        return 175, 175, 175


def draw_box(pos, c=None):
    fill_rect(pos[0] * 22 + 72, pos[1] * 22 + 22, 22, 22, get_box_color(pos) if c is None else c)
    n = board[pos[0]][pos[1]]
    if n is not None:
        lp = ["P", "R", "N", "B", "Q", "K"]
        draw_string(lp[n[0]], pos[0] * 22 + 78, pos[1] * 22 + 24, black if n[1] else green,
                    get_box_color(pos) if c is None else c)


def move_piece(oldpos, newpos):
    global board
    piece = board[oldpos[0]][oldpos[1]]
    board[oldpos[0]][oldpos[1]] = None
    board[newpos[0]][newpos[1]] = piece
    draw_box(oldpos)
    draw_box(newpos)


def create_initial_board():
    global board

    # Board initialisation
    for i in range(8):
        board.append([])
        for j in range(8):
            board[i].append(None)

    # Placing pieces
    start_pos = [[1, 2, 3, 4, 5, 3, 2, 1], [0, 0, 0, 0, 0, 0, 0, 0]]
    for i in range(len(start_pos)):
        for j in range(len(start_pos[i])):
            piece1 = (start_pos[i][j], 0)
            piece2 = (start_pos[i][j], 1)
            board[i][j] = piece1
            board[7 - i][j] = piece2

    # Drawing board
    fill_screen(green)
    fill_rect(70, 20, 180, 180, black)
    fill_rect(0, 0, 10, 222, white)
    fill_rect(310, 0, 320, 222, black)
    for i in range(8):
        draw_string(str(i + 1), i * 22 + 78, 2, black, green)
        draw_string(str(i + 1), i * 22 + 78, 202, black, green)
        for j in range(8):
            draw_box((i, j))
    letters = ["A", "B", "C", "D", "E", "F", "G", "H"]
    for i in range(8):
        draw_string(letters[i], 56, i * 22 + 24, black, green)
        draw_string(letters[i], 254, i * 22 + 24, black, green)


def selection(init_pos, player):
    pos = init_pos.copy()
    moves = []
    selected_piece = None

    def draw_selection():
        draw_box(pos, c=(255, 0, 0))

    def clear_selection():
        draw_box(pos, c=(0, 255, 0) if pos in moves else None)

    draw_selection()
    while True:
        mvt = (0, 0)
        if keydown(KEY_RIGHT):
            mvt = (1, 0)
        elif keydown(KEY_LEFT):
            mvt = (-1, 0)
        elif keydown(KEY_DOWN):
            mvt = (0, 1)
        elif keydown(KEY_UP):
            mvt = (0, -1)
        elif keydown(KEY_OK):
            for p in moves:
                draw_box(p)
            if pos in moves and selected_piece is not None and board[selected_piece[0]][selected_piece[1]][1] == player:
                if board[pos[0]][pos[1]] is not None and board[pos[0]][pos[1]][1] == 5:
                    draw_string("VICTOIRE", 0, 0)
                move_piece(selected_piece, pos)
                # MAX DE TRUCS A VERIFIER ICI
                return pos
            else:
                if selected_piece != pos and board[pos[0]][pos[1]] is not None:
                    selected_piece = pos.copy()
                    moves = get_possible_moves(pos)
                    for i in moves:
                        draw_box(i, c=(0, 255, 0))
                else:
                    selected_piece = None
                    moves.clear()
                draw_selection()
            sleep(0.1)
        if mvt != (0, 0) and 0 <= pos[0] + mvt[0] <= 7 and 0 <= pos[1] + mvt[1] <= 7:
            clear_selection()
            pos[0] += mvt[0]
            pos[1] += mvt[1]
            draw_selection()
        sleep(0.1)


def get_possible_moves(pos):
    p = board[pos[0]][pos[1]]
    if p is None:
        return []
    else:
        moves = []
        if p[0] == 0:  # PIONS
            if board[pos[0] + 1 - 2 * p[1]][pos[1]] is None:  # TOUT DROIT
                moves.append([pos[0] + 1 - 2 * p[1], pos[1]])
                if pos[0] == 1 + 5 * p[1] and board[pos[0] + 2 - 4 * p[1]][pos[1]] is None:  # DOUBLE SI PAS BOUGE
                    moves.append([pos[0] + 2 - 4 * p[1], pos[1]])
            for i in [1, -1]:  # BOUFFER LES AUTRES
                if 0<= pos[1]+i <= 7 and board[pos[0] + 1 - 2 * p[1]][pos[1] + i] is not None and \
                        board[pos[0] + 1 - 2 * p[1]][pos[1] + i][1] != p[1]:
                    moves.append([pos[0] + 1 - 2 * p[1], pos[1] + i])
        d = []
        if p[0] == 1 or p[0] == 4:  # LIGNES DROITES
            d += [(1, 0), (-1, 0), (0, 1), (0, -1)]
        if p[0] == 4 or p[0] == 3:  # DIAGONALES
            d += [(1, 1), (-1, 1), (1, -1), (-1, -1)]
        for e in d:  # CHECK INFINI (LA PIECE PEUT AVANCER TANT QU'ELLE N'EST PAS BLOQUEE)
            i, j = pos[0] + e[0], pos[1] + e[1]
            while 0 <= i <= 7 and 0 <= j <= 7 and (board[i][j] is None or board[i][j][1] != p[1]):
                moves.append([i, j])
                if board[i][j] is not None and board[i][j][1] != p[1]: break
                i += e[0]
                j += e[1]
        d = []
        if p[0] == 2:
            d = [(2, 1), (2, -1), (-2, 1), (-2, -1), (1, -2), (-1, -2), (1, 2), (-1, -2)]
        if p[0] == 5:
            d = [(1, 0), (1, 1), (0, 1), (-1, 0), (0, -1), (-1, -1), (-1, 1), (1, -1)]
        for i in d:  # CHECK LIMITE (SEULEMENT MVT PRECISES DANS d)
            if 0 <= pos[0] + i[0] <= 7 and 0 <= pos[1] + i[1] <= 7:
                if board[pos[0] + i[0]][pos[1] + i[1]] is None or board[pos[0] + i[0]][pos[1] + i[1]][1] != p[1]:
                    moves.append([pos[0] + i[0], pos[1] + i[1]])
        return moves


def chess():
    def draw_turn(p):
        fill_rect(15 + 265 * p, 5, 25, 10, (255, 0, 0))
        fill_rect(15 + 265 * (0 ** p), 5, 25, 10, green)

    create_initial_board()
    pos1 = [1, 3]
    pos2 = [6, 4]
    while True:
        draw_turn(0)
        pos1 = selection(pos1, 0)
        sleep(0.2)
        draw_turn(1)
        pos2 = selection(pos2, 1)
        sleep(0.2)


def menu_chess():
    def vis_add():
        pass

    list_opt = []
    modif_opt = menu("CHESS", vis_add, black, green, list_opt, black)
    if modif_opt[-1]: chess()


menu_chess()
display()
