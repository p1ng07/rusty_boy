# controls

Movement: wasd
Start: i
Select: u
B: j
A: k

# Info

- Does not emulate bootrom, and will not run alternative bootroms.
- Therefore, Gameboy Color palettes cannot be chosen during run time, maybe I will add them via a color chooser in the application

# TODO

1. Better timer
2. Turn into a gameboy color
3. 2x and 4x speed
4. Cpu information window
5. Memory viewer
6. Save states
7. Quick reloading of save states

# Gameboy color

1. HDMA
2. VRAM Banking
3. Background Map attributes
4. BG-OBJ priority fix (don't forget bit 0 of LCDC)
5. Palette arrays and adding default bg and obj's palettes to these arrays
6. Pass cgb-acid2
