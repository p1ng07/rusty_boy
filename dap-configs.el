thread 'main' panicked at 'Instruction [d6] not implemented', src/cpu.rs:485:18

(setq rustic-run-arguments "roms/cpu_instrs/individual/01-special.gb")
(dap-register-debug-template "Rusty boy debug"
			     (list
			      :name "Rusty boy debug"
			      :type "cppdbg"
			      :request "launch"
			      :MIMode "gdb"
			      :miDebuggerPath "/usr/bin/gdb"
			      :program "./target/debug/rusty_boy"
			      :args '("./roms/cpu_instrs/cpu_instrs.gb")
			      :cwd "${workspaceFolder}"))
