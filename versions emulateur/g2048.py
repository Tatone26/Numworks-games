from num_to_wind import *
from menu import *

xg = 120
yg = 20
size = 4
pts = 0
best = 1112
lbx = []

def dwGrid():
    fill_rect(120,20,185,185,(180,180,180))
    for i in range(1,size+1):
        for e in range(1,size+1): dwNoBox(i,e)

def dwNoBox(x,y):
    rs = int(185/size)
    fill_rect(xg+1+rs*(x-1),yg+1+rs*(y-1),rs-1,rs-1,(220,220,220))
def dwBox(box):
    rs = int(185/size)
    fill_rect(xg+1+rs*(box[0]-1),yg+1+rs*(box[1]-1),rs-1,rs-1,(250-box[2]*10,250-box[2]*20,250-box[2]*20))
    draw_string(str(2**box[2]),xg+int(rs/2)+rs*(box[0]-1)-int(len(str(2**box[2]))/2*8),yg+int(rs/2)-8+rs*(box[1]-1))

def drawPts():
    fill_rect(30, 40, 50, 70, (255, 255, 255))
    draw_string(str(pts), 35, 45)

def pinput():
    while True:
        if keydown(KEY_UP):return [0,-1]
        elif keydown(KEY_DOWN):return [0,1]
        elif keydown(KEY_RIGHT):return [1,0]
        elif keydown(KEY_LEFT):return [-1,0]

def addBox():
    global lbx
    if randint(1, 9)==9: pw = 2
    else : pw = 1
    if len(lbx)<size*size:
        placed = False
        while not placed :
            nwbx = (randint(1, size), randint(1, size), pw)
            fullpos = [[x[0], x[1]] for x in lbx]
            if [nwbx[0], nwbx[1]] not in fullpos:
                lbx.append(nwbx)
                dwBox(nwbx)
                placed = True
        return True
    return False

def moveBoxes(drct):
    global lbx
    moved = True
    timesmoved = 0
    while moved :
        moved = False
        fullpos = [[x[0], x[1]] for x in lbx]
        newlbx = []
        for b in lbx:
            if 1<=b[0]+drct[0]<=size and 1<=b[1]+drct[1]<=size and [b[0]+drct[0], b[1]+drct[1]] not in fullpos:
                newlbx.append((b[0]+drct[0], b[1]+drct[1], b[2]))
                dwNoBox(b[0], b[1])
                dwBox(newlbx[-1])
                moved = True
            else :
                newlbx.append(b)
        lbx = newlbx.copy()
        if moved: timesmoved +=1
    if timesmoved >= 1 :return True
    else : return False

def fuseBoxes(drct):
    global lbx, pts
    newlbx = []
    used = []
    if drct[0]!=0:n = 0
    else : n = 1
    lbx.sort(key = lambda x : x[int(not bool(n))])
    lbx.sort(key = lambda x : x[n])
    if sum(drct)<0:
        it = range(len(lbx))
    else :
        it = range(len(lbx)-1, -1, -1)
    for i in it:
        b = lbx[i]
        if b not in used and (b[0]-drct[0], b[1]-drct[1], b[2]) in lbx and (b[0]-drct[0], b[1]-drct[1], b[2]) not in used:
            newlbx.append((b[0], b[1], b[2]+1))
            dwBox(newlbx[-1])
            used.append(b)
            dwNoBox(b[0]-drct[0], b[1]-drct[1])
            used.append((b[0]-drct[0], b[1]-drct[1], b[2]))
            pts += 2**(b[2]+1)
    for b in lbx:
        if b not in used:
            newlbx.append(b)
    lbx = newlbx.copy()
    return len(used)>0

def isdead():
    for b in lbx:
        if len(lbx)<size*size or (b[0]+1, b[1], b[2]) in lbx or (b[0]-1, b[1], b[2]) in lbx or (b[0], b[1]+1, b[2]) in lbx or (b[0], b[1]-1, b[2]) in lbx:
            return False
    return True

def game():
    global pts, lbx
    fill_screen((255, 255, 255))
    dwGrid()
    draw_string("Record :  \n    "+str(best), 5, 180)
    draw_string("Points :", 5, 20)
    drawPts()
    addBox()
    addBox()
    for i in lbx : dwBox(i)
    dead = False
    while not dead:
        drct = pinput()
        moved = moveBoxes(drct)
        sleep(0.08) #anim
        fused = fuseBoxes(drct)
        if fused:
            drawPts()
            sleep(0.08) #anim
            moveBoxes(drct)
        if moved or fused:
            addBox()
            if isdead():
                dead = True
        sleep(0.25)
    replay = False
    draw_string("Perdu !", xg+70, yg+15)
    draw_string("Rejouer : <OK>", xg+5, yg+115)
    draw_string("Menu : <EXE>", xg+5, yg+160)
    if pts>best: draw_string("Nouveau \n record !", 5, 100)
    print("Score : "+str(pts))
    while not keydown(KEY_EXE):
        if keydown(KEY_OK):
            replay = True
            break
    pts = 0
    lbx.clear()
    if replay : game()
    else : menu2048()

def menu2048():
    global size
    def va():
        fill_rect(0,75,320,20,(230,180,180))
    opt = menu("2048",va,(0,0,0),(255,255,255),[["Taille plateau", ("3x3", "4x4", "5x5", "6x6", "7x7", "8x8"), size-2]])
    size = opt[0]+2
    if opt[-1]==True:game()

menu2048()

ntw.mainloop()