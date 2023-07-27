# controls

- Movement: wasd
- Start: i
- Select: u
- B: j
- A: k
- Shift: Enter 2x speed mode
- Space: Pause

# Info

A Semi-accurate Gameboy Color emulator in Rust!

_Features:_

- Save states.
- Double speed (press shift)
- Pause and stepping(press space)

_*No, no sound is planned*_, but if you find any bugs or crashes, feel free to open an issue.

The graphics are done with a `scanline renderer`, instead of a cycle accurate state machine,
so expect some visual bugs on tests and the few GB/GBC games that require super accurate emulation.

# Build

There are available builds in the releases page.

But you can also just clone the repo and run `cargo run --release`
