# Numworks Games

Some simple games for numworks calculator, to run in python or as external applications ! I'm proud of them, but don't expect anything crazier than classic games remakes :)

I make these game for fun, to learn Rust and to give myself a challenge by programming on a very limited device : the glorious Numworks Calculator !

> These games are tested on a real N0110 and the latest version of the Epsilon software.

*Feel free to download and copy the code it if you want, but keep in mind it's tailored specifically for tight hardware hacks rather than standard textbook code.*

**Instructions and controls are included in the games themselves!**

> **AI USAGE NOTE** : Yes, I use LLMs to help write code. No, I don't use autonomous agents: I do all the testing on hardware myself, profile the performance, understand every line, and dictate the low-level optimizations.
> Art is completely human made, with zero AI generated images. Yes, my drawing skills are pretty limited.
> It still takes tens of hours of manual engineering to hit this level of performance under extreme hardware constraints.

---

## Applications

**All games can be downloaded as a single package with `alltheapps.nwa` ! It even saves a lot of space thanks to compiler optimisations.** The only downside is that save data isn't shared between games launched with `alltheapps` and games launched with standalone apps.

**NEW : A 2D Engine !**

> **Warning:** Resetting the calculator will wipe any installed app and all saved data. You can enter **GodMode** to edit your high scores by pressing Shift while clicking on "settings".

### Games

* **[All the Apps](/apps/alltheapps)** : This is a single application combining all the games with a simple menu. The data is not shared with the normal apps, so you can't really use it in parallel of isolated apps.
* **[Flappy Bird](./apps/flappybird/)** : Everybody knows Flappy Bird. As of today, it is by far the most **technically advanced game** I made. *It has been remade to be the first to use my new engine !*
* **[Pacman](./apps/pacman/)** : A pretty correct recreation of the original arcade Pacman game. You won't believe how many lines of code this needs.
* **[Tetris](./apps/tetris/)** : Yeah. **Tetris!** I feel like it is almost as good as it can be :) I *think* I followed every rule of the original game.
* **[Snake](./apps/snake/)** : My take on the classic game!
* **[Connect Four](./apps/connectfour/)** : You can play against your friend or **an AI**, and even try some (strange) 3 players games !
* **[Solitaire](./apps/solitaire/)** : A Classic Solitaire, with classic rules.

### Engine & Utilities

* **[Num_engine](./apps/num_engine)** : A lightweight 2D engine made to simplify the game development. It works in parallel to the graphical utilities below, specialized for 2D sprites moving around.
* **[Numworks_utils](./apps/numworks_utils/)** contains a lot of the utility code I use : the numworks default functions, the entire menu code, graphical tools and more.
* **[Model](./apps/model/)** is a basic repo you can copy to start making a game using my template. It doesn't show how to use the Engine for now.
* **[Nppm_decoder](./apps/nppm_decoder/)** is a build utility I made to process the images at compile time. Necessary to use my graphical modules, like *tiling*.

The official software comes with a lot of limitations, but that's what makes it interesting too !

If you want to create some apps that run on Epsilon, see the official [Rust-based template](https://github.com/numworks/epsilon-sample-app-rust), [C++](https://github.com/numworks/epsilon-sample-app-cpp) and [C](https://github.com/numworks/epsilon-sample-app-c).

---

### Installation instructions

* Go to the [`apps/`](./apps/) folder
* Download the `.nwa` file(s) of the game(s) you want
* Go to [my.numworks.com/apps](https://my.numworks.com/apps) (on a Chromium browser like Chrome or Edge)
* Connect your NumWorks calculator
* Follow the instructions on the website, and upload the file(s) you downloaded
* Click **DOWNLOAD** and you're good to go !

---

### Some technical details of the Numworks calculator

> Or what make these games more difficult to make than you think

The official software comes with a lot of limitations, but that's what makes it interesting too !

As I'm not an expert at all, I won't talk about the *truly* technical stuff. You can start on the [official website](https://www.numworks.com/engineering/software/#read-our-coding-guidelines) if you want more details.

* **RAM constraints:** Total RAM is only 256 KB, and the stack is just 32 KB. Applications get roughly ~125 KB + 32 KB stack. It is small, but by being careful and using static arrays to avoid stack overflows, it doesn't cause major problems.
* **No dynamic allocator (`no_std`):** There is no heap and no standard library, so no `malloc`. The `heapless` crate solves most problems, but I still hand-wrote basic utilities to avoid pulling extra dependencies and wasting binary space.
* **Display buffer:** The screen is 320×240 pixels using RGB565 (16 bits / 2 bytes per pixel). Maintaining a full-screen framebuffer in RAM is mathematically impossible with the memory budget.
* **Slow data bus:** The refresh rate is around 40 Hz, but the bus transferring pixels to the screen controller is slow. You cannot redraw the entire display in a single frame. Drawing right-to-left is good practice to prevent tearing against the scanline.
* Look at how the Engine and graphical utilities are structured to see how these bottlenecks were handled.

---

## Python Games (Old)

> *Note: These are older projects and I don't work on them anymore. Check out the Apps above for better versions!*

**Everything needs the menu.py script downloaded as well.**

* Snake : An incredible classic, a lot of fun if you are motivated. The default settings are the hardest one, try to beat 20 or so points! **See the Apps for a better one!**
* Connect4 : Simple but efficient and fun to play with friends. You can even play with strange rules or at 3 players! **See the Apps for a better one!**
* 2048 : Works but pretty bad looking compared to the original.
* Solitaire : A fully functional Classic Solitaire! **See the Apps for a better one!**

Thanks to [ZetaMap](https://github.com/ZetaMap/ZetaMap), you can run these python scripts on your pc with the Kandinksy and the Ion-Numworks modules installed to test them.
You can find a lot more programs on numworks.com, where anyone can submit their own.

---

### TODOS

* [x] Update Menu to add lot-of-options support with multi page
* [ ] Update Flappy Bird to be better !!
* [ ] NEW GAMES IDEAS : crossy-road like, platformer