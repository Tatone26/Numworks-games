# Numworks Games 🎮

Some simple games for numworks calculator, to run in python or as external applications! I'm proud of them, but don't expect anything crazier than classic games remakes :)

I make these game for fun, to learn Rust and to give myself a challenge by programming on a very limited device : the glorious NumWorks Calculator !

> These games are tested on a real N0110 and the latest version of the Epsilon software.
> *Feel free to download and copy the code if you want, but keep in mind it's tailored specifically for tight hardware hacks rather than standard textbook code.*

**Instructions and controls are included in the games themselves!**

> **AI USAGE NOTE** : Yes, I use LLMs to help write code. No, I don't use autonomous agents: I do all the testing on hardware myself, profile the performance, understand every line, and dictate the low-level optimizations.
> It still takes tens of hours of manual engineering to hit this level of performance under extreme hardware constraints.
> Art is completely human made, with zero AI generated images. Yes, my drawing skills are pretty limited.

---

## Applications

**All games can be downloaded as a single package with `alltheapps.nwa`!** The only downside is that save data isn't shared between games launched with `alltheapps` and games launched with standalone apps.

> **Warning:** Resetting or crashing the calculator *may* wipe any installed app and all saved data. You can enter **GodMode** to edit your high scores by pressing Shift while clicking on "settings".

**NEW : A 2D Engine !**

### Games

| Game & Links | Description |
|------------------|-------------|
| **[All the Apps](./apps/alltheapps)** - [Download](https://raw.githubusercontent.com/Tatone26/Numworks-games/main/apps/alltheapps.nwa) | This is a single application combining all games with a simple menu. |
| **[Flappy Bird](./apps/flappybird)** - [Download](https://raw.githubusercontent.com/Tatone26/Numworks-games/main/apps/flappybird.nwa) | Everybody knows Flappy Bird. It is by far the most technically advanced game I made. *It has been remade to be the first to use my new engine!* |
| **[Pacman](./apps/pacman)** - [Download](https://raw.githubusercontent.com/Tatone26/Numworks-games/main/apps/pacman.nwa) | A pretty correct recreation of the original arcade Pacman game. You won't believe how many lines of code this needs. |
| **[Tetris](./apps/tetris)** - [Download](https://raw.githubusercontent.com/Tatone26/Numworks-games/main/apps/tetris.nwa) | Yeah. **Tetris!** I feel like it is almost as good as it can be :) I *think* I followed every rule of the original game. |
| **[Snake](./apps/snake)** - [Download](https://raw.githubusercontent.com/Tatone26/Numworks-games/main/apps/snake.nwa) | My take on the classic game! |
| **[Connect Four](./apps/connectfour)** - [Download](https://raw.githubusercontent.com/Tatone26/Numworks-games/main/apps/connectfour.nwa) | You can play against your friend or **an AI** (sort of), and even try some 3 players games! |
| **[Solitaire](./apps/solitaire)** - [Download](https://raw.githubusercontent.com/Tatone26/Numworks-games/main/apps/solitaire.nwa) | A Classic Solitaire, with classic rules. |

### Engine & Utilities

- **[Num_engine](./apps/num_engine)** : A lightweight 2D engine made to simplify game development. It works in parallel to the graphical utilities below, specialized for 2D sprites moving around.
- **[Numworks_utils](./apps/numworks_utils/)** contains a lot of utility code I use: the numworks default functions, entire menu code, graphical tools and more.
- **[Model](./apps/model/)** is a basic repo you can copy to start making a game using my template. It doesn't show how to use the Engine for now.
- **[Nppm_decoder](./apps/nppm_decoder/)** is a build utility I made to process images at compile time. Necessary to use graphical modules like *tiling*.

The official software comes with pretty intense limitations, but that's what makes it interesting! If you want to create apps that run on Epsilon, see the official [Rust-based template](https://github.com/numworks/epsilon-sample-app-rust), [C++](https://github.com/numworks/epsilon-sample-app-cpp) and [C](https://github.com/numworks/epsilon-sample-app-c).

---

## Installation Instructions

1. Download the `.nwa` file(s) of the game(s) you want from the links above or the [apps](./apps) folder.
2. Go to [my.numworks.com/apps](https://my.numworks.com/apps) (on a Chromium browser like Chrome or Edge)
3. Connect your NumWorks calculator via USB
4. Follow the instructions on the website, and upload the file(s) you downloaded
5. Click **DOWNLOAD** and you're good to go!

---

## Technical Details

The NumWorks calculator has limitations that make these games more challenging to create:

- **RAM constraints:** Total RAM is only 256 KB, with a stack of just 32 KB. Applications seem to get roughly ~125 KB + 32 KB stack. Using static arrays avoids major stack issues.
- **No dynamic allocator (`no_std`):** There's no heap or standard library, so no `malloc`. The `heapless` crate solves most problems, but I still hand-wrote basic utilities to avoid extra dependencies and waste binary space.
- **Display buffer:** Screen is 320×240 pixels using RGB565 (16 bits / 2 bytes per pixel). Maintaining a full-screen framebuffer in RAM is mathematically impossible with the memory budget creating the need for lots of workarounds.
- **Slow data bus:** Refresh rate around 40 Hz, but the bus transferring pixels to screen controller is slow. Cannot redraw entire display in single frame. Drawing right-to-left helps preventing tearing against scanline.

---

## Python Games (Old) ⚠️

> *Note: These are older projects and I don't work on them anymore.*
> **Everything needs the menu.py script downloaded as well.**

Find all the files in the `python games` folder.

- Snake : An incredible classic, a lot of fun if you are motivated. The default settings are the hardest one, try to beat 20 or so points! See [the Apps for a better version](./apps/snake/).
- Connect4 : Simple but efficient and fun to play with friends. You can even play with strange rules or at 3 players! See [the Apps for a better version](./apps/connectfour/).
- 2048 : Works but pretty bad looking compared to the original.
- Solitaire : A fully functional Classic Solitaire! See [the Apps for a better version](./apps/solitaire/).

Thanks to [ZetaMap](https://github.com/ZetaMap/ZetaMap), you can run these python scripts on your PC with Kandinsky and Ion-Numworks modules installed. You can find a lot more programs on numworks.com, where anyone can submit their own.

---

### TODOs 📝

- [x] Update Menu to add lot-of-options support with multi page
- [ ] Update Flappy Bird to be better !!
- [ ] NEW GAMES IDEAS : crossy-road like, platformer
