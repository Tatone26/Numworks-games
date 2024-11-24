# NUMWORKS UTILS
This lib contains a lot of code I use in all the Rust apps. 

### EADK
Eadk.rs contains all the functions we can call on the Epsilon ecosystem, and some basic implementations.
It is kindly given by the Numworks team, and I made minimal changes to it.

### Menu
Menu.rs contains all the code necessary to make a fully functional menu.
It can take care of a very simple selection menu, or a fully complete menu with options and all. 

### Utils
Utils.rs contains some useful functions which we don't have access to since we are coding in no_std. 

### Graphical
Graphical.rs contains some functions useful for everything UI-related. It allows drawing images or do some fading.

### Tiling
Tiling.rs is a more precise graphical lib. It allows using a simple Tileset and draw those tiles. Useful to make code simpler.