# Numworks Games
Some simple games for numworks calculator, to run in python or as applications !
I make these game for fun and to learn Rust and give myself a challenge by programming on a very limited device.

Most of the games here should work as perfectly as I could make them but don't expect anything crazy !

## Python Games

  **Everything needs the menu.py script downloaded as well.**

- Snake : An incredible classic, a lot of fun if you are motivated. The default settings are the hardest one, try to beat 20 or so points! **See the Apps for a better one!**
- Connect4 : Simple but efficient and fun to play with friends. You can even play with strange rules or at 3 players! **See the Apps for a better one!**
- 2048 : Works but pretty bad looking compared to the original.
- Solitaire : A fully functional Classic Solitaire! **See the Apps for a better one!**

Thanks to [ZetaMap](https://github.com/ZetaMap/ZetaMap), you can run these python scripts on your pc with the Kandinksy and the Ion-Numworks modules installed to test them.
You can find a lot more programs on numworks.com, where anyone can submit their own.

## Applications

**NEW : Data can now be saved, so games now store High Scores and settings.**

**NEW BIS : All games can now be downloaded as a single package with "alltheapps.nwa" ! Saves a lot of space thanks to compiler optimisations.**

> Be warned : resetting the console will remove any app and all the data too -> you can enter GodMode to edit the highscores by pressing 'shift' while clicking on settings.

*Feel free to download and copy the code it if you want, but it's quite messy as I work alone.*

I'm also using these projects to learn some Rust, as I am already used to code in C. Read it at your own eyes' risk !

PS : These games are tested on a real N0110 and the latest version of the Epsilon software.

**Instructions and controls are included in the games themselves!**

- [All the Apps](./apps/alltheapps) : This is a single application combining all the games with a simple menu. The data is not shared with the normal apps!
- [Snake](./apps/snake/) : My take on the classic game!
- [Connect Four](./apps/connectfour/) : You can play against your friend or **an AI**, and even try some (strange) 3 players games !
- [Solitaire](./apps/solitaire/) : A Classic Solitaire, with classic rules.
- [Tetris](./apps/tetris/) : Yeah. **Tetris!** I feel like it is almost as good as it can be :) I *think* I followed every rule of the original game.
- [Flappy Bird](./apps/flappybird/) : Everybody knows Flappy Bird. As of today, it is by far the most **technically advanced game** I made. (and the one I worked the most on)

- [Numworks_utils](./apps/numworks_utils/) contains a lot of the utility code I use : the numworks default functions, the entire menu code, graphical tools and more.
- [Model](./apps/model/) is a basic repo you can copy to start making a game using my template.
- [Nppm_decoder](./apps/nppm_decoder/) is a build utility I made to process the images at compile time. Necessary to use my graphical modules, like *tiling*.

The official software comes with a lot of limitations, but that's what makes it interesting too !

If you want to create some apps that run on Epsilon, see the official [Rust-based template](https://github.com/numworks/epsilon-sample-app-rust), [C++](https://github.com/numworks/epsilon-sample-app-cpp) and [C](https://github.com/numworks/epsilon-sample-app-c).

### Installation instructions :
  - go to the "apps" folder
  - download the .nwa file(s) of the game(s) you want
  - go to my.numworks.com/apps (on a chromium navigator like Chrome or Edge)
  - follow the instructions on the website, and put the file(s) you downloaded.
  - click DOWNLOAD and you're good to go !


### Some technical details of the Numworks calculator
#### Or what make these games more difficult to make than you think

As I'm not an expert at all, I won't talk about the *truly* technical stuff. You can start [here](https://www.numworks.com/engineering/software/#read-our-coding-guidelines) if you want more details.

- The RAM is only 256kb, and the stack is only 32kb. I know you can use around 125kb + 32kb of this RAM in the applications. It is really small but by being careful, it does not cause a lot of problems.
- There is no heap and no standard libray, so no malloc to use. This changes a lot of things, but the **heapless crate** can solve most problems. I still had to rewrite some very basic functions, as I didn't want to import another crate (to save space).
- The screen is 320x240 pixels and uses RGB565 (16 bits) colors. That means you need 2 bytes / pixel when drawing anything, which is a lot in this context.
- The refresh rate is not bad (45Hz) **BUT** it works from the top-left to the bottom-right. This means that rectangles drawn too late in the frame can look *really* bad in the bottom-right corner. It seems to be a good pratice to draw from right to left if possible.
- The VBlank time is pretty small too, so it is next to impossible to draw the entire screen. It is probably able to, but I couldn't find how with the tools we have.


### TODOS

- [ ] "Admin" editor to manually set highscore after a reset (which means creating a number input widget D: )
- [ ] NEW GAMES
