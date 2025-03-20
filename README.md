# Worlanwv

![worlawv_cover2](https://github.com/user-attachments/assets/8ba5ad0a-757d-4a71-9842-93b48070c3fd)

Whenever Worlanwv returns it spells disaster for the world. Can you finish your art project in time so that at least those who come after you will be able to prepare?

This very short game is a walking simulator with light puzzle elements. My personal best time was 156,000 years and I invite you to beat my score!

It was written in Rust with the game engine [Bevy](https://bevyengine.org/) and was submitted to the fifth official [Bevy game jam](https://itch.io/jam/bevy-jam-5). It is available in binary form for Linux, Mac, and Windows, and can be played in the browser on [itch.io](https://tracefree.itch.io/worlanwv). The browser version has a few technical limitations and does not look as good as the desktop version, so I recommend downloading the game.

## Controls
- W/A/S/D: Move
- Mouse: Look
- Space: Jump
- E: Interact
- Hold Q: Fast forward time after picking up the hourglass
- Escape: Pause

## If you get stuck
Unfortunately the game has some technical as well as design problems (mostly specific to the web build). Here are some workarounds if you run into a problem:
- Click the full-screen icon when playing the web build!
- If you can't see or click on the "Play" button at the beginning, press Enter.
- If you have troubles in Firefox and really don't want to use the desktop version (it's much better!), try Chrome.
- If you suddenly get stuck in geometry that spawned out of nowhere, I'm really sorry. It should go away after waiting 60 seconds, or if you have the hourglass hold Q to fast forward until you're free. If that - doesn't help I'm afraid you'll have to restart (it's a very short game though).
- There are some graphical glitches and sometimes it's hard to make out things. Please try the desktop version, it's better there!
- The Credits button in the menu doesn't actually do anything. Sorry about that, you'll have to finish the game to see it :P (Or just continue reading this page)

## Build
If you prefer to build the game from source, make sure you have git and cargo installed and then run the following commands:
```
git clone https://github.com/tracefree/Worlanwv.git
cd Worlanwv
cargo run
```

## Credits
### Code
- The [Bevy quickstart template](https://github.com/TheBevyFlock/bevy_new_2d) was used as a starting point for the project structure.
- The rest of the code including gameplay and shaders was written by me, tracefree a.k.a. Rie. It it available under you choice of either [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).

### Original composition
- Music by Ira Provectus and Michael Feigl, all rights reserved

### 3D Assets
- tracefree a.k.a. Rie, CC0

### Free sound effects
- "Impact sfx 031" by [AudioPaplin](https://freesound.org/people/AudioPapkin/sounds/648454/), CC Attribution NonCommercial 4.0
- "Roaring Ocean" by [kangaroovindaloo](https://freesound.org/people/kangaroovindaloo/sounds/246515/), CC Attribution 4.0
- "Mouse Hover" by [Andreas Mustola](https://freesound.org/people/Andreas.Mustola/sounds/255764/), CC0
- "Click03" by [moogy73](https://freesound.org/people/moogy73/sounds/425726/), CC0
- "Sound Effects Pack" by [OwlishMedia](https://opengameart.org/content/sound-effects-pack), CC0
- "Plant_Harvest_03" by [Valenspire](https://freesound.org/people/Valenspire/sounds/699492/), CC0
- "Rowing2.wav" by [juskiddink](https://freesound.org/people/juskiddink/sounds/101921/), CC Attribution 4.0
- "Rock_Hammer_Chisel_01.wav" by [dheming](https://freesound.org/people/dheming/sounds/240981/), CC Attribution 4.0

![worl_s2](https://github.com/user-attachments/assets/74df542d-7235-4fff-9c6f-533973b31ae4)
  
