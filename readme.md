# controls

Movement: wasd
Start: i
Select: u
B: j
A: k

# TODO

1. Main cycle timing
2. Pause game, stepping
3. Cpu information window
4. Memory viewer
5. Halt Bug
6. Simple background only ppu
7. Pass more blargg tests like instr_timing and rest of cpu
8. MBC3 and MBC5
9. Pass acid test
10. Tetris

# Blargg-cpu tests

- [x] 01-special
- [x] 02-interrupts.gb, it passes, BUUUT the timer appears to be slower to fire an interrupt, when compared to the verified logs
- [x] 03-op_sp_hl.gb
- [x] 04-op_r_imm.gb
- [x] 05-op_rp.gb
- [x] 06-ld_r,r.gb
- [x] 07-jr,jp,call,ret,rst.gb
- [ ] 08-misc_instrs.gb
- [ ] 09-op-r-r.gb
- [ ] 10-bit-ops.gb
- [ ] 11-op-a,(hl).gb
