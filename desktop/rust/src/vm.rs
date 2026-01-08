// CVERE Virtual Machine - Core execution engine
// vm.rs - Main VM executor

use std::fmt;

/// Status register flags
#[derive(Debug, Clone, Copy)]
pub struct StatusFlags {
    pub zero: bool,      // Z flag
    pub negative: bool,  // N flag
    pub carry: bool,     // C flag
    pub overflow: bool,  // V flag
}

impl StatusFlags {
    pub fn new() -> Self {
        StatusFlags {
            zero: false,
            negative: false,
            carry: false,
            overflow: false,
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut sr = 0u16;
        if self.zero { sr |= 1 << 0; }
        if self.negative { sr |= 1 << 1; }
        if self.carry { sr |= 1 << 2; }
        if self.overflow { sr |= 1 << 3; }
        sr
    }

    pub fn from_u16(sr: u16) -> Self {
        StatusFlags {
            zero: (sr & (1 << 0)) != 0,
            negative: (sr & (1 << 1)) != 0,
            carry: (sr & (1 << 2)) != 0,
            overflow: (sr & (1 << 3)) != 0,
        }
    }
}

/// CVERE Virtual Machine
pub struct CVEREVM {
    // General purpose registers (R0-RF)
    pub registers: [u16; 16],
    
    // Special registers
    pub pc: u16,           // Program Counter
    pub sp: u16,           // Stack Pointer
    pub lr: u16,           // Link Register
    pub sr: StatusFlags,   // Status Register
    
    // Memory (64KB)
    pub memory: Vec<u8>,
    
    // Execution state
    pub halted: bool,
    pub cycle_count: u64,
}

impl CVEREVM {
    /// Create a new VM instance
    pub fn new() -> Self {
        CVEREVM {
            registers: [0; 16],
            pc: 0,
            sp: 0xFFFE,  // Stack grows downward from top of memory
            lr: 0,
            sr: StatusFlags::new(),
            memory: vec![0; 65536],
            halted: false,
            cycle_count: 0,
        }
    }

    /// Load program into memory
    pub fn load_program(&mut self, program: &[u16], start_address: u16) {
        let mut addr = start_address as usize;
        for &instruction in program {
            if addr + 1 < self.memory.len() {
                // Little-endian storage
                self.memory[addr] = (instruction & 0xFF) as u8;
                self.memory[addr + 1] = (instruction >> 8) as u8;
                addr += 2;
            }
        }
    }

    /// Fetch instruction from memory
    fn fetch(&mut self) -> u16 {
        let addr = self.pc as usize;
        if addr + 1 >= self.memory.len() {
            return 0xFFFF; // HALT on out of bounds
        }
        
        // Little-endian fetch
        let low = self.memory[addr] as u16;
        let high = self.memory[addr + 1] as u16;
        self.pc = self.pc.wrapping_add(2);
        
        (high << 8) | low
    }

    /// Read from memory (word-aligned)
    fn read_memory(&self, address: u16) -> u16 {
        let addr = address as usize;
        if addr + 1 >= self.memory.len() {
            return 0;
        }
        
        let low = self.memory[addr] as u16;
        let high = self.memory[addr + 1] as u16;
        (high << 8) | low
    }

    /// Write to memory (word-aligned)
    fn write_memory(&mut self, address: u16, value: u16) {
        let addr = address as usize;
        if addr + 1 < self.memory.len() {
            self.memory[addr] = (value & 0xFF) as u8;
            self.memory[addr + 1] = (value >> 8) as u8;
        }
    }

    /// Update status flags based on result
    fn update_flags(&mut self, result: u16) {
        self.sr.zero = result == 0;
        self.sr.negative = (result & 0x8000) != 0;
    }

    /// Update flags with carry
    fn update_flags_with_carry(&mut self, result: u32) {
        let result_16 = result as u16;
        self.sr.zero = result_16 == 0;
        self.sr.negative = (result_16 & 0x8000) != 0;
        self.sr.carry = result > 0xFFFF;
    }

    /// Execute a single instruction
    pub fn step(&mut self) -> Result<(), String> {
        if self.halted {
            return Err("VM is halted".to_string());
        }

        let instruction = self.fetch();
        self.cycle_count += 1;

        // Decode opcode
        let opcode = (instruction >> 12) & 0xF;

        match opcode {
            0x0 => self.exec_nop(),
            0x1 => self.exec_add(instruction),
            0x2 => self.exec_addi(instruction),
            0x3 => self.exec_sub(instruction),
            0x4 => self.exec_and(instruction),
            0x5 => self.exec_or(instruction),
            0x6 => self.exec_xor(instruction),
            0x7 => self.exec_not(instruction),
            0x8 => self.exec_shl(instruction),
            0x9 => self.exec_shr(instruction),
            0xA => self.exec_load(instruction),
            0xB => self.exec_store(instruction),
            0xC => self.exec_loadi(instruction),
            0xD => self.exec_jmp(instruction),
            0xE => self.exec_beq(instruction),
            0xF => self.exec_bne_or_extended(instruction),
            _ => Err(format!("Invalid opcode: 0x{:X}", opcode)),
        }
    }

    /// Run until HALT or error
    pub fn run(&mut self, max_cycles: u64) -> Result<u64, String> {
        let start_cycle = self.cycle_count;
        
        while !self.halted && (self.cycle_count - start_cycle) < max_cycles {
            self.step()?;
        }
        
        Ok(self.cycle_count - start_cycle)
    }

    // Instruction implementations

    fn exec_nop(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn exec_add(&mut self, instr: u16) -> Result<(), String> {
        let rd = ((instr >> 8) & 0xF) as usize;
        let rs = ((instr >> 4) & 0xF) as usize;
        let rt = (instr & 0xF) as usize;
        
        let result = self.registers[rs].wrapping_add(self.registers[rt]) as u32;
        println!("{}", result as u16);
        self.registers[rd] = result as u16;
        
        // R0 always reads as 0
        self.registers[0] = 0;
        
        self.update_flags_with_carry(result);
        Ok(())
    }

    fn exec_addi(&mut self, instr: u16) -> Result<(), String> {
        let rd = ((instr >> 8) & 0xF) as usize;
        let imm = (instr & 0xFF) as u16;
        
        let result = self.registers[rd].wrapping_add(imm) as u32;
        self.registers[rd] = result as u16;
        self.registers[0] = 0;
        
        self.update_flags_with_carry(result);
        Ok(())
    }

    fn exec_sub(&mut self, instr: u16) -> Result<(), String> {
        let rd = ((instr >> 8) & 0xF) as usize;
        let rs = ((instr >> 4) & 0xF) as usize;
        let rt = (instr & 0xF) as usize;
        
        let result = self.registers[rs].wrapping_sub(self.registers[rt]);
        self.registers[rd] = result;
        self.registers[0] = 0;
        
        self.update_flags(result);
        Ok(())
    }

    fn exec_and(&mut self, instr: u16) -> Result<(), String> {
        let rd = ((instr >> 8) & 0xF) as usize;
        let rs = ((instr >> 4) & 0xF) as usize;
        let rt = (instr & 0xF) as usize;
        
        let result = self.registers[rs] & self.registers[rt];
        self.registers[rd] = result;
        self.registers[0] = 0;
        
        self.update_flags(result);
        Ok(())
    }

    fn exec_or(&mut self, instr: u16) -> Result<(), String> {
        let rd = ((instr >> 8) & 0xF) as usize;
        let rs = ((instr >> 4) & 0xF) as usize;
        let rt = (instr & 0xF) as usize;
        
        let result = self.registers[rs] | self.registers[rt];
        self.registers[rd] = result;
        self.registers[0] = 0;
        
        self.update_flags(result);
        Ok(())
    }

    fn exec_xor(&mut self, instr: u16) -> Result<(), String> {
        let rd = ((instr >> 8) & 0xF) as usize;
        let rs = ((instr >> 4) & 0xF) as usize;
        let rt = (instr & 0xF) as usize;
        
        let result = self.registers[rs] ^ self.registers[rt];
        self.registers[rd] = result;
        self.registers[0] = 0;
        
        self.update_flags(result);
        Ok(())
    }

    fn exec_not(&mut self, instr: u16) -> Result<(), String> {
        let rd = ((instr >> 8) & 0xF) as usize;
        let rs = ((instr >> 4) & 0xF) as usize;
        
        let result = !self.registers[rs];
        self.registers[rd] = result;
        self.registers[0] = 0;
        
        self.update_flags(result);
        Ok(())
    }

    fn exec_shl(&mut self, instr: u16) -> Result<(), String> {
        let rd = ((instr >> 8) & 0xF) as usize;
        let rs = ((instr >> 4) & 0xF) as usize;
        let rt = (instr & 0xF) as usize;
        
        let shift = self.registers[rt] & 0xF; // Limit shift to 0-15
        let result = self.registers[rs] << shift;
        self.registers[rd] = result;
        self.registers[0] = 0;
        
        self.update_flags(result);
        Ok(())
    }

    fn exec_shr(&mut self, instr: u16) -> Result<(), String> {
        let rd = ((instr >> 8) & 0xF) as usize;
        let rs = ((instr >> 4) & 0xF) as usize;
        let rt = (instr & 0xF) as usize;
        
        let shift = self.registers[rt] & 0xF;
        let result = self.registers[rs] >> shift;
        self.registers[rd] = result;
        self.registers[0] = 0;
        
        self.update_flags(result);
        Ok(())
    }

    fn exec_load(&mut self, instr: u16) -> Result<(), String> {
        let rd = ((instr >> 8) & 0xF) as usize;
        let rs = ((instr >> 4) & 0xF) as usize;
        let offset = (instr & 0xF) as u16;
        
        let address = self.registers[rs].wrapping_add(offset * 2); // Word-aligned
        let value = self.read_memory(address);
        
        self.registers[rd] = value;
        self.registers[0] = 0;
        
        Ok(())
    }

    fn exec_store(&mut self, instr: u16) -> Result<(), String> {
        let rd = ((instr >> 8) & 0xF) as usize;
        let rs = ((instr >> 4) & 0xF) as usize;
        let offset = (instr & 0xF) as u16;
        
        let address = self.registers[rs].wrapping_add(offset * 2);
        self.write_memory(address, self.registers[rd]);
        
        Ok(())
    }

    fn exec_loadi(&mut self, instr: u16) -> Result<(), String> {
        let rd = ((instr >> 8) & 0xF) as usize;
        let imm = (instr & 0xFF) as u16;
        
        // Sign-extend 8-bit immediate to 16-bit
        let value = if imm & 0x80 != 0 {
            imm | 0xFF00
        } else {
            imm
        };
        
        self.registers[rd] = value;
        self.registers[0] = 0;
        
        Ok(())
    }

    fn exec_jmp(&mut self, instr: u16) -> Result<(), String> {
        let target = instr & 0xFFF;
        self.pc = target;
        Ok(())
    }

    fn exec_beq(&mut self, instr: u16) -> Result<(), String> {
        let rc = ((instr >> 8) & 0xF) as usize;
        let offset = (instr & 0xFF) as i8; // Signed offset
        
        if self.registers[rc] == 0 {
            self.pc = self.pc.wrapping_add((offset as i16 * 2) as u16);
        }
        
        Ok(())
    }

    fn exec_bne_or_extended(&mut self, instr: u16) -> Result<(), String> {
        // Check for HALT
        if instr == 0xFFFF {
            self.halted = true;
            return Ok(());
        }
        
        // Check for extended instructions
        let extended_op = (instr >> 8) & 0xFF;
        if extended_op >= 0xF0 && extended_op <= 0xF3 {
            return Ok(()); // Extended instructions not fully implemented
        }
        
        // BNE instruction
        let rc = ((instr >> 8) & 0xF) as usize;
        let offset = (instr & 0xFF) as i8;
        
        if self.registers[rc] != 0 {
            self.pc = self.pc.wrapping_add((offset as i16 * 2) as u16);
        }
        
        Ok(())
    }

    /// Reset VM to initial state
    pub fn reset(&mut self) {
        self.registers = [0; 16];
        self.pc = 0;
        self.sp = 0xFFFE;
        self.lr = 0;
        self.sr = StatusFlags::new();
        self.halted = false;
        self.cycle_count = 0;
    }

    /// Get register value by name
    pub fn get_register(&self, name: &str) -> Option<u16> {
        match name.to_uppercase().as_str() {
            "PC" => Some(self.pc),
            "SP" => Some(self.sp),
            "LR" => Some(self.lr),
            "SR" => Some(self.sr.to_u16()),
            r if r.starts_with('R') => {
                let num = u8::from_str_radix(&r[1..], 16).ok()?;
                if (num as usize) < 16 {
                    Some(self.registers[num as usize])
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl fmt::Display for CVEREVM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "=== CVERE VM State ===")?;
        writeln!(f, "PC: 0x{:04X}  SP: 0x{:04X}  LR: 0x{:04X}", self.pc, self.sp, self.lr)?;
        writeln!(f, "SR: Z={} N={} C={} V={}", 
            self.sr.zero as u8, self.sr.negative as u8, 
            self.sr.carry as u8, self.sr.overflow as u8)?;
        writeln!(f, "Cycles: {}  Halted: {}", self.cycle_count, self.halted)?;
        writeln!(f, "\nRegisters:")?;
        for i in 0..16 {
            write!(f, "R{:X}: 0x{:04X}  ", i, self.registers[i])?;
            if (i + 1) % 4 == 0 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

// Example usage and tests
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
        
        vm.load_program(&program, 0);
        vm.run(100).unwrap();
        
        assert_eq!(vm.registers[1], 5);
        assert_eq!(vm.registers[2], 3);
        assert_eq!(vm.registers[3], 8);
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
        
        vm.load_program(&program, 0);
        vm.run(100).unwrap();
        
        assert_eq!(vm.registers[1], 10);
    }
}