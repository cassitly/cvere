// ============================================================================
// desktop/rust/src/registers.rs
// Register file module for CVERE VM
// ============================================================================

/// Register file with 16 general-purpose registers and special registers
pub struct RegisterFile {
    // General purpose registers R0-RF
    gp_regs: [u16; 16],
    
    // Special registers
    pub pc: u16,    // Program Counter
    pub sp: u16,    // Stack Pointer
    pub lr: u16,    // Link Register
    pub sr: u16,    // Status Register
}

impl RegisterFile {
    /// Create new register file
    pub fn new() -> Self {
        RegisterFile {
            gp_regs: [0; 16],
            pc: 0,
            sp: 0xFFFE,  // Stack grows downward
            lr: 0,
            sr: 0,
        }
    }

    /// Read from general purpose register
    pub fn read_gp(&self, reg: u8) -> u16 {
        if reg >= 16 {
            return 0;
        }
        // R0 always reads as 0
        if reg == 0 {
            0
        } else {
            self.gp_regs[reg as usize]
        }
    }

    /// Write to general purpose register
    pub fn write_gp(&mut self, reg: u8, value: u16) {
        if reg >= 16 {
            return;
        }
        // R0 is hardwired to 0, writes are ignored
        if reg != 0 {
            self.gp_regs[reg as usize] = value;
        }
    }

    /// Reset all registers
    pub fn reset(&mut self) {
        self.gp_regs = [0; 16];
        self.pc = 0;
        self.sp = 0xFFFE;
        self.lr = 0;
        self.sr = 0;
    }

    /// Get status flags
    pub fn get_flags(&self) -> StatusFlags {
        StatusFlags::from_u16(self.sr)
    }

    /// Set status flags
    pub fn set_flags(&mut self, flags: StatusFlags) {
        self.sr = flags.to_u16();
    }

    /// Dump register state for debugging
    pub fn dump(&self) -> String {
        let mut result = String::new();
        result.push_str("General Purpose Registers:\n");
        for i in 0..16 {
            result.push_str(&format!("  R{:X}: 0x{:04X}", i, self.read_gp(i)));
            if (i + 1) % 4 == 0 {
                result.push('\n');
            }
        }
        result.push_str(&format!("\nSpecial Registers:\n"));
        result.push_str(&format!("  PC: 0x{:04X}\n", self.pc));
        result.push_str(&format!("  SP: 0x{:04X}\n", self.sp));
        result.push_str(&format!("  LR: 0x{:04X}\n", self.lr));
        result.push_str(&format!("  SR: 0x{:04X} ", self.sr));
        
        let flags = self.get_flags();
        result.push_str(&format!("[Z={} N={} C={} V={}]\n", 
            flags.zero as u8, flags.negative as u8,
            flags.carry as u8, flags.overflow as u8));
        
        result
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StatusFlags {
    pub zero: bool,
    pub negative: bool,
    pub carry: bool,
    pub overflow: bool,
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
