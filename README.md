# Numworks Games
Some simple games for numworks calculator, to run in python or as applications ! 
These games are made for fun, when I have a little time and want something simple to program. Don't expect anything incredible, but I'm a bit of a perfectionist so there shoudn't be any (major) bugs.

## Python Games

  **Everything needs the menu.py script downloaded as well.** 

- Snake : An incredible classic, works and a lot of fun if you are motivated. The default settings are the hardest one, try to beat 20 or so points! *See the Apps for a better one!*
- Connect4 : Simple but efficient and fun to play with friends. You can even play with strange rules or at 3 players! *See the Apps for a better one!*
- Chess : Not finished (and will probably not be), but still playable. Many rules are not implemented.
- 2048 : Works and as fun as the original.
- Solitaire : My proudest achievement. A Classic Solitaire fully functional! *See the Apps for a better one!*

Thanks to [ZetaMap](https://github.com/ZetaMap/ZetaMap), you can run these python scripts on your pc with the Kandinksy and the Ion-Numworks modules installed to test them.
You can find a lot more programs on numworks.com, where you can even submit your own.

## Applications

Feel free to look at the code and download it if you want, but it's quite messy. 
Remember that it is a calculator and not a game console, so anything graphical is quite challenging with the official software! I tried a lot of things before finding a good (enough for now) way to print images, and I can't speed up the refresh rate. It's quite frustrating, but it is also what makes programming on Numworks so interesting!

I'm also using these projects to write some Rust, as I am already used to code in C.

  **Instructions and controls are included in the games!**

- Snake : My take on the classic game!
- Puissance4 = Connect4 (I'm not bothering changing every file name): works as the python version, but better.
- Solitaire : After managing the Python version, it was just a matter of making a good way to display the cards.
- Tetris : Yeah. Tetris! I *think* I followed every rule of the original game.
- Flappy Bird : Work in progress. Difficult because first game with real moving parts.

- Numworks_utils contains every utility code I use : the numworks functions, the entire menu code, Tilemap tools and other functions.
- Model is a basic repo you can use to start your game using my template.
- Ignore ppm_decoder and engine for now, as it doesn't work at all. I'm trying to find a better way to manage sprites and draw images despite the bad refresh rate. 


There are still a lot of impossible things with the official software (like saving anything), so consider installing an other one like Omega if you want to do some really complex things. 
If you want to create some apps that run on Epsilon, see the [Rust-based template](https://github.com/numworks/epsilon-sample-app-rust), [C++](https://github.com/numworks/epsilon-sample-app-cpp) and [C](https://github.com/numworks/epsilon-sample-app-c). 

### Installation instructions : 
  - go to the "apps" folder
  - download the .nwa file(s) of the game(s) you want
  - go to my.numworks.com/apps (on chromium)
  - follow the instructions on the website, and put the file(s) you downloaded.
  - click DOWNLOAD and you're good to go !

