(setq rustic-run-arguments "roms/cpu_instrs/individual/03-op_sp_hl.gb")

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
