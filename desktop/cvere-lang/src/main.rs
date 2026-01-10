// ============================================================================
// desktop/rust/src/main.rs
// Updated main entry point using the new module structure
// ============================================================================

mod memory;
mod registers;
mod decoder;
mod vm;

use vm::CVEREVM;
use std::env;
use std::fs;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "run" => {
            if args.len() < 3 {
                eprintln!("Error: Missing program file");
                return;
            }
            run_program(&args[2]);
        }
        "trace" => {
            if args.len() < 3 {
                eprintln!("Error: Missing program file");
                return;
            }
            trace_program(&args[2]);
        }
        "test" => {
            run_tests();
        }
        "repl" => {
            repl();
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("CVERE Virtual Machine");
    println!();
    println!("Usage:");
    println!("  cvere run <file>    - Run a program from file");
    println!("  cvere trace <file>  - Run with execution tracing");
    println!("  cvere test          - Run built-in tests");
    println!("  cvere repl          - Start interactive REPL");
}

fn run_program(filename: &str) {
    match load_program_from_file(filename) {
        Ok(program) => {
            let mut vm = CVEREVM::new();
            
            if let Err(e) = vm.load_program(&program, 0) {
                eprintln!("Error loading program: {}", e);
                return;
            }

            println!("Running program from: {}", filename);
            match vm.run(100000) {
                Ok(cycles) => {
                    println!("\nProgram completed in {} cycles", cycles);
                    println!("{}", vm);
                }
                Err(e) => {
                    eprintln!("Runtime error: {}", e);
                    println!("{}", vm);
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn trace_program(filename: &str) {
    match load_program_from_file(filename) {
        Ok(program) => {
            let mut vm = CVEREVM::new();
            vm.set_trace(true);
            
            if let Err(e) = vm.load_program(&program, 0) {
                eprintln!("Error loading program: {}", e);
                return;
            }

            println!("Tracing program from: {}", filename);
            println!("==========================================");
            
            match vm.run(100000) {
                Ok(cycles) => {
                    println!("==========================================");
                    println!("\nProgram completed in {} cycles", cycles);
                    println!("{}", vm);
                }
                Err(e) => {
                    eprintln!("\nRuntime error: {}", e);
                    println!("{}", vm);
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn load_program_from_file(filename: &str) -> Result<Vec<u16>, String> {
    let contents = fs::read_to_string(filename)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut program = Vec::new();
    
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        // Parse hex value
        let hex = line.trim_start_matches("0x").trim_start_matches("0X");
        match u16::from_str_radix(hex, 16) {
            Ok(value) => program.push(value),
            Err(_) => return Err(format!("Invalid hex value: {}", line)),
        }
    }

    Ok(program)
}

fn repl() {
    println!("CVERE Interactive REPL");
    println!("Commands: run, step, reset, reg <name>, mem <addr> <len>, dump, quit");
    println!();

    let mut vm = CVEREVM::new();
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        input.clear();

        if stdin.read_line(&mut input).is_err() {
            break;
        }

        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "quit" | "exit" => break,
            
            "reset" => {
                vm.reset();
                println!("VM reset");
            }
            
            "step" => {
                match vm.step() {
                    Ok(_) => println!("Step executed\n{}", vm),
                    Err(e) => println!("Error: {}", e),
                }
            }
            
            "run" => {
                let cycles = parts.get(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1000);
                
                match vm.run(cycles) {
                    Ok(executed) => println!("Executed {} cycles\n{}", executed, vm),
                    Err(e) => println!("Error: {}", e),
                }
            }
            
            "load" => {
                if parts.len() < 2 {
                    println!("Usage: load <instruction_hex>");
                    continue;
                }
                
                let mut program = Vec::new();
                for hex in &parts[1..] {
                    let hex = hex.trim_start_matches("0x");
                    match u16::from_str_radix(hex, 16) {
                        Ok(val) => program.push(val),
                        Err(_) => {
                            println!("Invalid hex: {}", hex);
                            continue;
                        }
                    }
                }
                
                vm.reset();
                if let Err(e) = vm.load_program(&program, 0) {
                    println!("Error loading: {}", e);
                } else {
                    println!("Loaded {} instructions", program.len());
                }
            }
            
            "dump" => {
                println!("{}", vm.state_dump());
            }
            
            "mem" => {
                let start = parts.get(1)
                    .and_then(|s| u16::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                    .unwrap_or(0);
                let len = parts.get(2)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(64);
                
                println!("{}", vm.memory_dump(start, len));
            }
            
            "trace" => {
                let enabled = parts.get(1)
                    .map(|s| *s == "on" || *s == "1" || *s == "true")
                    .unwrap_or(true);
                vm.set_trace(enabled);
                println!("Trace: {}", if enabled { "enabled" } else { "disabled" });
            }
            
            "help" => {
                println!("Commands:");
                println!("  load <hex> [hex...]  - Load instructions");
                println!("  step                 - Execute one instruction");
                println!("  run [cycles]         - Run for N cycles (default 1000)");
                println!("  reset                - Reset VM");
                println!("  dump                 - Show VM state");
                println!("  mem <addr> [len]     - Dump memory");
                println!("  trace [on|off]       - Toggle execution trace");
                println!("  quit                 - Exit REPL");
            }
            
            _ => {
                println!("Unknown command. Type 'help' for commands.");
            }
        }
    }

    println!("Goodbye!");
}

fn run_tests() {
    println!("Running CVERE VM Tests...\n");

    test_simple_arithmetic();
    test_loop();
    test_memory_operations();
    test_branches();
    test_logical_operations();
    test_shifts();
    test_r0_hardwired();

    println!("\nAll tests completed!");
}

fn test_simple_arithmetic() {
    println!("Test: Simple Arithmetic");
    let mut vm = CVEREVM::new();
    let program = vec![
        0xC105, // LOADI R1, 0x05
        0xC203, // LOADI R2, 0x03
        0x1312, // ADD R3, R1, R2
        0x3421, // SUB R4, R2, R1
        0xFFFF, // HALT
    ];
    
    vm.load_program(&program, 0).unwrap();
    vm.run(100).unwrap();
    
    assert_eq!(vm.registers.read_gp(1), 5, "R1 should be 5");
    assert_eq!(vm.registers.read_gp(2), 3, "R2 should be 3");
    assert_eq!(vm.registers.read_gp(3), 8, "R3 should be 8 (5+3)");
    assert_eq!(vm.registers.read_gp(4), 0xFFFE, "R4 should be -2 (3-5)");
    println!("  ✓ Passed");
}

fn test_loop() {
    println!("Test: Loop Counter");
    let mut vm = CVEREVM::new();
    let program = vec![
        0xC100, // LOADI R1, 0x00      ; Counter = 0
        0xC20A, // LOADI R2, 0x0A      ; Limit = 10
        // loop:
        0x2101, // ADDI R1, 0x01       ; Counter++ (now R1 = 1, 2, 3...)
        0x3321, // SUB R3, R2, R1      ; R3 = Limit - Counter
        0xF3FD, // BNE R3, -3          ; If R3 != 0, jump back to ADDI
        0xFFFF, // HALT
    ];
    
    vm.load_program(&program, 0).unwrap();
    vm.run(1000).unwrap();
    
    assert_eq!(vm.registers.read_gp(1), 10, "R1 should be 10");
    assert!(vm.halted, "VM should be halted");
    println!("  ✓ Passed");
}

fn test_memory_operations() {
    println!("Test: Memory Load/Store");
    let mut vm = CVEREVM::new();
    let program = vec![
        0xC142, // LOADI R1, 0x42
        0xC210, // LOADI R2, 0x10
        0xB120, // STORE R1, R2, 0x0
        0xA320, // LOAD R3, R2, 0x0
        0xFFFF, // HALT
    ];
    
    vm.load_program(&program, 0).unwrap();
    vm.run(100).unwrap();
    
    assert_eq!(vm.registers.read_gp(3), 0x42, "R3 should contain loaded value");
    println!("  ✓ Passed");
}

fn test_branches() {
    println!("Test: Conditional Branches");
    let mut vm = CVEREVM::new();
    let program = vec![
        0xC100, // LOADI R1, 0x00
        0xE101, // BEQ R1, +1 (should branch)
        0xC2FF, // LOADI R2, 0xFF (should skip)
        0xC301, // LOADI R3, 0x01
        0xF301, // BNE R3, +1 (should branch)
        0xC4FF, // LOADI R4, 0xFF (should skip)
        0xFFFF, // HALT
    ];
    
    vm.load_program(&program, 0).unwrap();
    vm.run(100).unwrap();
    
    assert_eq!(vm.registers.read_gp(2), 0, "R2 should be 0 (instruction skipped)");
    assert_eq!(vm.registers.read_gp(4), 0, "R4 should be 0 (instruction skipped)");
    println!("  ✓ Passed");
}

fn test_logical_operations() {
    println!("Test: Logical Operations");
    let mut vm = CVEREVM::new();
    let program = vec![
        0xC10F, // LOADI R1, 0x0F
        0xC233, // LOADI R2, 0x33
        0x4312, // AND R3, R1, R2
        0x5412, // OR R4, R1, R2
        0x6512, // XOR R5, R1, R2
        0x7611, // NOT R6, R1
        0xFFFF, // HALT
    ];
    
    vm.load_program(&program, 0).unwrap();
    vm.run(100).unwrap();
    
    assert_eq!(vm.registers.read_gp(3), 0x03, "AND result");
    assert_eq!(vm.registers.read_gp(4), 0x3F, "OR result");
    assert_eq!(vm.registers.read_gp(5), 0x3C, "XOR result");
    assert_eq!(vm.registers.read_gp(6), 0xFFF0, "NOT result");
    println!("  ✓ Passed");
}

fn test_shifts() {
    println!("Test: Shift Operations");
    let mut vm = CVEREVM::new();
    let program = vec![
        0xC10F, // LOADI R1, 0x0F
        0xC202, // LOADI R2, 0x02
        0x8312, // SHL R3, R1, R2
        0x9412, // SHR R4, R1, R2
        0xFFFF, // HALT
    ];
    
    vm.load_program(&program, 0).unwrap();
    vm.run(100).unwrap();
    
    assert_eq!(vm.registers.read_gp(3), 0x3C, "SHL result (0x0F << 2)");
    assert_eq!(vm.registers.read_gp(4), 0x03, "SHR result (0x0F >> 2)");
    println!("  ✓ Passed");
}

fn test_r0_hardwired() {
    println!("Test: R0 Hardwired to Zero");
    let mut vm = CVEREVM::new();
    
    // Try to write to R0
    vm.registers.write_gp(0, 0xFFFF);
    assert_eq!(vm.registers.read_gp(0), 0, "R0 should always read as 0");
    
    // Try to write to R0 via instruction
    let program = vec![
        0xC0FF, // LOADI R0, 0xFF (should be ignored)
        0xFFFF, // HALT
    ];
    
    vm.load_program(&program, 0).unwrap();
    vm.run(100).unwrap();
    
    assert_eq!(vm.registers.read_gp(0), 0, "R0 should still be 0");
    println!("  ✓ Passed");
}
