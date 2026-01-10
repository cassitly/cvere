// ============================================================================
// desktop/rust/src/vm.rs (Updated version using modules)
// CVERE Virtual Machine - Core execution engine
// ============================================================================

use crate::memory::Memory;
use crate::registers::{RegisterFile, StatusFlags};
use crate::decoder::{InstructionDecoder, InstructionFormat};
use std::fmt;

/// CVERE Virtual Machine
pub struct CVEREVM {
    /// Register file
    pub registers: RegisterFile,
    
    /// Memory subsystem (64KB)
    pub memory: Memory,
    
    /// Execution state
    pub halted: bool,
    pub cycle_count: u64,
    
    /// Debugging
    pub trace_enabled: bool,
}

impl CVEREVM {
    /// Create a new VM instance
    pub fn new() -> Self {
        CVEREVM {
            registers: RegisterFile::new(),
            memory: Memory::new(65536), // 64KB
            halted: false,
            cycle_count: 0,
            trace_enabled: false,
        }
    }

    /// Load program into memory
    pub fn load_program(&mut self, program: &[u16], start_address: u16) -> Result<(), String> {
        self.memory.load_program(program, start_address as usize)
    }

    /// Fetch instruction from memory at PC
    fn fetch(&mut self) -> Result<u16, String> {
        let instruction = self.memory.read_word(self.registers.pc as usize)?;
        self.registers.pc = self.registers.pc.wrapping_add(2);
        Ok(instruction)
    }

    /// Update status flags based on result
    fn update_flags(&mut self, result: u16) {
        let mut flags = self.registers.get_flags();
        flags.zero = result == 0;
        flags.negative = (result & 0x8000) != 0;
        self.registers.set_flags(flags);
    }

    /// Update flags with carry
    fn update_flags_with_carry(&mut self, result: u32) {
        let mut flags = self.registers.get_flags();
        let result_16 = result as u16;
        flags.zero = result_16 == 0;
        flags.negative = (result_16 & 0x8000) != 0;
        flags.carry = result > 0xFFFF;
        self.registers.set_flags(flags);
    }

    /// Execute a single instruction
    pub fn step(&mut self) -> Result<(), String> {
        if self.halted {
            return Err("VM is halted".to_string());
        }

        let pc_before = self.registers.pc;
        let instruction = self.fetch()?;
        let decoded = InstructionDecoder::decode(instruction);
        
        if self.trace_enabled {
            println!("{}", InstructionDecoder::disassemble(pc_before, instruction));
        }

        self.cycle_count += 1;

        // Execute based on format
        match decoded.format {
            InstructionFormat::RType => self.execute_r_type(&decoded)?,
            InstructionFormat::IType => self.execute_i_type(&decoded)?,
            InstructionFormat::MType => self.execute_m_type(&decoded)?,
            InstructionFormat::JType => self.execute_j_type(&decoded)?,
            InstructionFormat::BType => self.execute_b_type(&decoded)?,
            InstructionFormat::Extended => self.execute_extended(&decoded)?,
            InstructionFormat::Special => self.execute_special(&decoded)?,
        }

        Ok(())
    }

    /// Execute R-Type instruction
    fn execute_r_type(&mut self, decoded: &crate::decoder::DecodedInstruction) -> Result<(), String> {
        let rs_val = self.registers.read_gp(decoded.rs);
        let rt_val = self.registers.read_gp(decoded.rt);
        
        let result = match decoded.mnemonic {
            "ADD" => {
                let res = rs_val.wrapping_add(rt_val) as u32;
                self.update_flags_with_carry(res);
                res as u16
            }
            "SUB" => {
                let res = rs_val.wrapping_sub(rt_val);
                self.update_flags(res);
                res
            }
            "AND" => {
                let res = rs_val & rt_val;
                self.update_flags(res);
                res
            }
            "OR" => {
                let res = rs_val | rt_val;
                self.update_flags(res);
                res
            }
            "XOR" => {
                let res = rs_val ^ rt_val;
                self.update_flags(res);
                res
            }
            "NOT" => {
                let res = !rs_val;
                self.update_flags(res);
                res
            }
            "SHL" => {
                let shift = rt_val & 0xF;
                let res = rs_val << shift;
                self.update_flags(res);
                res
            }
            "SHR" => {
                let shift = rt_val & 0xF;
                let res = rs_val >> shift;
                self.update_flags(res);
                res
            }
            _ => return Err(format!("Unknown R-Type instruction: {}", decoded.mnemonic)),
        };

        self.registers.write_gp(decoded.rd, result);
        Ok(())
    }

    /// Execute I-Type instruction
    fn execute_i_type(&mut self, decoded: &crate::decoder::DecodedInstruction) -> Result<(), String> {
        let result = match decoded.mnemonic {
            "ADDI" => {
                let rd_val = self.registers.read_gp(decoded.rd);
                let res = rd_val.wrapping_add(decoded.imm8 as u16) as u32;
                self.update_flags_with_carry(res);
                res as u16
            }
            "LOADI" => {
                // Sign-extend 8-bit immediate to 16-bit
                let value = if decoded.imm8 & 0x80 != 0 {
                    (decoded.imm8 as u16) | 0xFF00
                } else {
                    decoded.imm8 as u16
                };
                value
            }
            _ => return Err(format!("Unknown I-Type instruction: {}", decoded.mnemonic)),
        };

        self.registers.write_gp(decoded.rd, result);
        Ok(())
    }

    /// Execute M-Type instruction
    fn execute_m_type(&mut self, decoded: &crate::decoder::DecodedInstruction) -> Result<(), String> {
        let rs_val = self.registers.read_gp(decoded.rs);
        let address = rs_val.wrapping_add((decoded.offset as u16) * 2);

        match decoded.mnemonic {
            "LOAD" => {
                let value = self.memory.read_word(address as usize)?;
                self.registers.write_gp(decoded.rd, value);
            }
            "STORE" => {
                let rd_val = self.registers.read_gp(decoded.rd);
                self.memory.write_word(address as usize, rd_val)?;
            }
            _ => return Err(format!("Unknown M-Type instruction: {}", decoded.mnemonic)),
        }

        Ok(())
    }

    /// Execute J-Type instruction
    fn execute_j_type(&mut self, decoded: &crate::decoder::DecodedInstruction) -> Result<(), String> {
        match decoded.mnemonic {
            "JMP" => {
                self.registers.pc = decoded.addr12;
            }
            _ => return Err(format!("Unknown J-Type instruction: {}", decoded.mnemonic)),
        }
        Ok(())
    }

    /// Execute B-Type instruction
    fn execute_b_type(&mut self, decoded: &crate::decoder::DecodedInstruction) -> Result<(), String> {
        let rc_val = self.registers.read_gp(decoded.rd); // Note: Rd field used as Rc for branches
        
        // Sign-extend offset
        let offset = if decoded.imm8 & 0x80 != 0 {
            ((decoded.imm8 as i8) as i16) * 2
        } else {
            (decoded.imm8 as i16) * 2
        };

        let should_branch = match decoded.mnemonic {
            "BEQ" => rc_val == 0,
            "BNE" => rc_val != 0,
            _ => return Err(format!("Unknown B-Type instruction: {}", decoded.mnemonic)),
        };

        if should_branch {
            self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
        }

        Ok(())
    }

    /// Execute Extended instruction
    fn execute_extended(&mut self, decoded: &crate::decoder::DecodedInstruction) -> Result<(), String> {
        match decoded.mnemonic {
            "CALL" => {
                // Save return address in LR
                self.registers.lr = self.registers.pc;
                // Read second word for address
                let target = self.fetch()?;
                self.registers.pc = target;
            }
            "RET" => {
                self.registers.pc = self.registers.lr;
            }
            "PUSH" => {
                // Fetch second word for register/value
                let value_word = self.fetch()?;
                let reg = ((value_word >> 8) & 0xF) as u8;
                let value = self.registers.read_gp(reg);
                
                self.registers.sp = self.registers.sp.wrapping_sub(2);
                self.memory.write_word(self.registers.sp as usize, value)?;
            }
            "POP" => {
                // Fetch second word for register
                let value_word = self.fetch()?;
                let reg = ((value_word >> 8) & 0xF) as u8;
                
                let value = self.memory.read_word(self.registers.sp as usize)?;
                self.registers.write_gp(reg, value);
                self.registers.sp = self.registers.sp.wrapping_add(2);
            }
            _ => return Err(format!("Unknown Extended instruction: {}", decoded.mnemonic)),
        }
        Ok(())
    }

    /// Execute Special instruction
    fn execute_special(&mut self, decoded: &crate::decoder::DecodedInstruction) -> Result<(), String> {
        match decoded.mnemonic {
            "NOP" => {
                // Do nothing
            }
            "HALT" => {
                self.halted = true;
            }
            _ => return Err(format!("Unknown Special instruction: {}", decoded.mnemonic)),
        }
        Ok(())
    }

    /// Run until HALT or error
    pub fn run(&mut self, max_cycles: u64) -> Result<u64, String> {
        let start_cycle = self.cycle_count;
        
        while !self.halted && (self.cycle_count - start_cycle) < max_cycles {
            self.step()?;
        }
        
        Ok(self.cycle_count - start_cycle)
    }

    /// Reset VM to initial state
    pub fn reset(&mut self) {
        self.registers.reset();
        self.memory.clear();
        self.halted = false;
        self.cycle_count = 0;
    }

    /// Enable/disable execution tracing
    pub fn set_trace(&mut self, enabled: bool) {
        self.trace_enabled = enabled;
    }

    /// Get current state as string
    pub fn state_dump(&self) -> String {
        let mut result = String::new();
        result.push_str("=== CVERE VM State ===\n");
        result.push_str(&self.registers.dump());
        result.push_str(&format!("\nCycles: {}\n", self.cycle_count));
        result.push_str(&format!("Halted: {}\n", self.halted));
        result
    }

    /// Dump memory region
    pub fn memory_dump(&self, start: u16, length: u16) -> String {
        self.memory.dump(start as usize, length as usize)
    }
}

impl fmt::Display for CVEREVM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.state_dump())
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_add() {
        let mut vm = CVEREVM::new();
        let program = vec![
            0xC105, // LOADI R1, 0x05
            0xC203, // LOADI R2, 0x03
            0x1312, // ADD R3, R1, R2
            0xFFFF, // HALT
        ];
        
        vm.load_program(&program, 0).unwrap();
        vm.run(100).unwrap();
        
        assert_eq!(vm.registers.read_gp(1), 5);
        assert_eq!(vm.registers.read_gp(2), 3);
        assert_eq!(vm.registers.read_gp(3), 8);
        assert!(vm.halted);
    }

    #[test]
    fn test_loop() {
        let mut vm = CVEREVM::new();
        let program = vec![
            0xC100, // LOADI R1, 0x00
            0xC20A, // LOADI R2, 0x0A
            0x2101, // ADDI R1, 0x01
            0x3321, // SUB R3, R2, R1
            0xF3FD, // BNE R3, -3
            0xFFFF, // HALT
        ];
        
        vm.load_program(&program, 0).unwrap();
        vm.run(1000).unwrap();
        
        assert_eq!(vm.registers.read_gp(1), 10);
        assert!(vm.halted);
    }

    #[test]
    fn test_memory_operations() {
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
        
        assert_eq!(vm.registers.read_gp(3), 0x42);
        assert!(vm.halted);
    }

    #[test]
    fn test_r0_hardwired() {
        let mut vm = CVEREVM::new();
        
        // Try to write to R0
        vm.registers.write_gp(0, 0xFFFF);
        assert_eq!(vm.registers.read_gp(0), 0);
    }

    #[test]
    fn test_flags() {
        let mut vm = CVEREVM::new();
        let program = vec![
            0xC100, // LOADI R1, 0x00
            0xFFFF, // HALT
        ];
        
        vm.load_program(&program, 0).unwrap();
        vm.step().unwrap();
        
        let flags = vm.registers.get_flags();
        assert!(flags.zero); // Result is 0
    }
}
