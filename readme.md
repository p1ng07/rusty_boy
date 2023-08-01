# WORK IN PROGRESS

# Info

A Semi-accurate Gameboy Color emulator in Rust!

_Features:_

- Save states.
- Double speed (press shift)
- Pause and stepping(press space)

_*No, no sound is planned*_, but if you find any bugs or crashes, feel free to open an issue.

The graphics are done with a `scanline renderer`, instead of a cycle accurate state machine,
so expect some visual bugs on tests and the few GB/GBC games that require super accurate emulation.

# controls

- Movement: wasd
- Start: i
- Select: u
- B: j
- A: k
- Shift: Enter 2x speed mode
- Space: Pause

# Images

![GIF](https://github.com/p1ng07/rusty_boy/blob/master/res/recording.gif)
![Zelda DX](https://github.com/p1ng07/rusty_boy/blob/master/res/zelda_screenshot.png)
![Pokemon crystal](https://github.com/p1ng07/rusty_boy/blob/master/res/pokemon.png)

# Build

Just clone the repo and run `cargo run --release`

Once it is done, there will be ready-made binaries in the releases page.

# TODO:

1. Fifo renderer
