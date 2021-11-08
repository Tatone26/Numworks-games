# Numworks
Some simple games for numworks calculator, to run in python.
The emulator is not perfect, a lot of functions are still missing (I'm adding them when I need them).
It has difficulties to work with a lot of Tkinter.canvas items, so complex drawings can be really slow to complete. You may need to prefer calling the least possible drawing fonctions in your programs ! (like I'm doing)

Graphically, only support kandinsky for now (turtle isn't great for games). Ion and Time are implemented, but ion keys have been implemented by hand very fast and not tested until useful. Random is also supported, via the most simple implementation. Math and cmath are not supported, you have to manually add them in both versions (numworks and emulator) to use them. For matplotlib.pyplot, I don't know for now if it can work with that emulator.

To get the emulator version of any program, just exchange all the numworks-specific imports (like kandinsky) by "import path/num_to_wind", and vice-versa. On numworks, I'm using "from {} import *" but you are free to change that of course. Just be careful, you may need to do some changes when going from emulator to calculator.

To get a program from the calculator or to the calculator, go to numworks.com (with chrome), connect to your account, go to "python" -> "your scripts" and connect your numworks to your pc with a usb cable. Then, you have some big buttons to get scripts from or to your numworks.

I don't think this github page will ever interest anyone but I made it public anyway... On numworks.com, you have a repository of lots of programs (and you can submit your owns apparently).
