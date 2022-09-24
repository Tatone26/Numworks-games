from kandinsky import *
from ion import *
from menu import menu
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


def auto_draw_box(pos, c=None):
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
    auto_draw_box(oldpos)
    auto_draw_box(newpos)


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
    fill_rect(0, 0, 320, 222, green)
    fill_rect(70, 20, 180, 180, black)
    for i in range(8):
        draw_string(str(i + 1), i * 22 + 78, 2, black, green)
        draw_string(str(i + 1), i * 22 + 78, 202, black, green)
        for j in range(8):
            auto_draw_box((i, j))
    letters = ["A", "B", "C", "D", "E", "F", "G", "H"]
    for i in range(8):
        draw_string(letters[i], 56, i * 22 + 24, black, green)
        draw_string(letters[i], 254, i * 22 + 24, black, green)


def selection(init_pos):
    pos = init_pos.copy()

    def draw_selection():
        auto_draw_box(pos, c=(255, 0, 0))

    draw_selection()
    while True:
        if keydown(KEY_RIGHT) and pos[0] < 7:
            auto_draw_box(pos)
            pos[0] += 1
            draw_selection()
        elif keydown(KEY_LEFT) and pos[0] > 0:
            auto_draw_box(pos)
            pos[0] -= 1
            draw_selection()
        elif keydown(KEY_DOWN) and pos[1] < 7:
            auto_draw_box(pos)
            pos[1] += 1
            draw_selection()
        elif keydown(KEY_UP) and pos[1] > 0:
            auto_draw_box(pos)
            pos[1] -= 1
            draw_selection()
        elif keydown(KEY_OK):
            moves = get_possible_moves(pos)
            for i in moves:
                auto_draw_box(i, c=(0, 255, 0))
        sleep(0.1)


def get_possible_moves(pos):
    p = board[pos[0]][pos[1]]
    if p is None:
        return []
    else:
        moves = []
        if p[0] == 0:  # PIONS
            if board[pos[0] + 1 - 2 * p[1]][pos[1]] is None:
                moves.append([pos[0] + 1 - 2 * p[1], pos[1]])
                if pos[0] == 1 + 5 * p[1] and board[pos[0] + 2 - 4 * p[1]][pos[1]] is None:
                    moves.append([pos[0] + 2 - 4 * p[1], pos[1]])
            if board[pos[0] + 1 - 2 * p[1]][pos[1] + 1] is not None and board[pos[0] + 1 - 2 * p[1]][pos[1] + 1][1] != \
                    p[1]:
                moves.append([pos[0] + 1 - 2 * p[1], pos[1] + 1])
            if board[pos[0] + 1 - 2 * p[1]][pos[1] - 1] is not None and board[pos[0] + 1 - 2 * p[1]][pos[1] - 1][1] != \
                    p[1]:
                moves.append([pos[0] + 1 - 2 * p[1], pos[1] - 1])

        if p[0] == 1 or p[0] == 4:  # LIGNES DROITES
            i = pos[0] + 1
            while i <= 7 and (board[i][pos[1]] is None or board[i][pos[1]][1] != p[1]):
                moves.append([i, pos[1]])
                if board[i][pos[1]] is not None and board[i][pos[1]][1] != p[1]: break
                i += 1
            j = pos[0] - 1
            while j >= 0 and (board[j][pos[1]] is None or board[j][pos[1]][1] != p[1]):
                moves.append([j, pos[1]])
                if board[j][pos[1]] is not None and board[j][pos[1]][1] != p[1]: break
                j -= 1
            k = pos[1] + 1
            while k <= 7 and (board[pos[0]][k] is None or board[pos[0]][k][1] != p[1]):
                moves.append([pos[0], k])
                if board[pos[0]][k] is not None and board[pos[0]][k][1] != p[1]: break
                k += 1
            e = pos[1] - 1
            while e >= 0 and (board[pos[0]][e] is None or board[pos[0]][e][1] != p[1]):
                moves.append([pos[0], e])
                if board[pos[0]][e] is not None and board[pos[0]][e][1] != p[1]: break
                e -= 1

        if p[0] == 4 or p[0] == 3:  # DIAGONALES
            i = 1
            while pos[0] + i <= 7 and pos[1] + i <= 7 and (
                    board[pos[0] + i][pos[1] + i] is None or board[pos[0] + i][pos[1] + i][1] != p[1]):
                moves.append([pos[0] + i, pos[1] + i])
                if board[pos[0] + i][pos[1] + i] is not None and board[pos[0] + i][pos[1] + i][1] != p[1]: break
                i += 1
            j = -1
            while pos[0] + j >= 0 and pos[1] + j >= 0 and (
                    board[pos[0] + j][pos[1] + j] is None or board[pos[0] + j][pos[1] + j][1] != p[1]):
                moves.append([pos[0] + j, pos[1] + j])
                if board[pos[0] + j][pos[1] + j] is not None and board[pos[0] + j][pos[1] + j][1] != p[1]: break
                j -= 1
            k = 1
            while pos[0] + k <= 7 and pos[1] - k >= 0 and (
                    board[pos[0] + k][pos[1] - k] is None or board[pos[0] + k][pos[1] - k][1] != p[1]):
                moves.append([pos[0] + k, pos[1] - k])
                if board[pos[0] + k][pos[1] - k] is not None and board[pos[0] + k][pos[1] - k][1] != p[1]: break
                k += 1
            e = -1
            while pos[0] + e >= 0 and pos[1] - e <= 7 and (
                    board[pos[0] + e][pos[1] - e] is None or board[pos[0] + e][pos[1] - e][1] != p[1]):
                moves.append([pos[0] + e, pos[1] - e])
                if board[pos[0] + e][pos[1] - e] is not None and board[pos[0] + e][pos[1] - e][1] != p[1]: break
                e -= 1
        if p[0] == 2:  # CHEVAL
            a = [(2, 1), (2, -1), (-2, 1), (-2, -1), (1, -2), (-1, -2), (1, 2), (-1, -2)]
            for i in a:
                if 0 <= pos[0] + i[0] <= 7 and 0 <= pos[1] + i[1] <= 7:
                    if board[pos[0] + i[0]][pos[1] + i[1]] is None or board[pos[0] + i[0]][pos[1] + i[1]][1] != p[1]:
                        moves.append([pos[0] + i[0], pos[1] + i[1]])
        return moves


def chess():
    create_initial_board()
    move_piece((1, 2), (5, 3))
    move_piece((1, 0), (5, 2))
    move_piece((6, 0), (5, 1))
    move_piece((1, 3), (2, 1))
    move_piece((0, 3), (1, 2))
    move_piece((6, 6), (4, 6))
    move_piece((6, 1), (4, 1))
    selection([0, 0])


def menu_chess():
    def vis_add():
        pass

    list_opt = []
    modif_opt = menu("CHESS", vis_add, black, green, list_opt, black)
    if modif_opt[-1]: chess()


menu_chess()
display()
