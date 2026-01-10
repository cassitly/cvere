// ============================================================================
// desktop/rust/src/registers.rs
// Register file module for CVERE VM
// ============================================================================

/// Privilege levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrivilegeLevel {
    Kernel = 0,  // Ring 0 - Full access
    Supervisor = 1,  // Ring 1 - Intermediate access
    User = 2,    // Ring 2 - Restricted access 
}

/// Register file with 16 general-purpose registers and special registers
pub struct RegisterFile {
    // General purpose registers R0-RF
    gp_regs: [u16; 16],
    
    // Special registers
    pub pc: u16,    // Program Counter
    pub sp: u16,    // Stack Pointer
    pub lr: u16,    // Link Register
    pub sr: u16,    // Status Register

    // Privilege mode registers
    pub kernel_sp: u16,  // Kernel mode stack pointer
    pub supervisor_sp: u16,  // Supervisor mode stack pointer
    pub user_sp: u16,    // User mode stack pointer
    pub privilege: PrivilegeLevel,

    // Exception handling
    pub exception_handler: u16,  // Exception handler address
    pub saved_pc: u16,           // Saved PC on exception
    pub saved_sr: u16,           // Saved SR on exception
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

            kernel_sp: 0xFFFE,
            supervisor_sp: 0xEFFE, // Supervisor stack starts lower
            user_sp: 0xDFFE,  // User stack starts lower
            privilege: PrivilegeLevel::Kernel,  // Boot in kernel mode
            exception_handler: 0x0010,  // Default exception handler
            saved_pc: 0,
            saved_sr: 0,
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

    /// Switch to kernel mode
    pub fn enter_kernel_mode(&mut self) {
        if self.privilege == PrivilegeLevel::User {
            // Save user stack pointer
            self.user_sp = self.sp;
            // Switch to kernel stack
            self.sp = self.kernel_sp;
            self.privilege = PrivilegeLevel::Kernel;
        }
    }

    /// DEMOTION: Used by Kernel to drop privilege to Supervisor or User
    pub fn drop_privilege(&mut self, target: PrivilegeLevel) {
        // Save current SP to the correct bank
        match self.privilege {
            PrivilegeLevel::Kernel => self.kernel_sp = self.sp,
            PrivilegeLevel::Supervisor => self.supervisor_sp = self.sp,
            PrivilegeLevel::User => self.user_sp = self.sp,
        }
        
        // Load target SP and set mode
        self.sp = match target {
            PrivilegeLevel::Kernel => self.kernel_sp,
            PrivilegeLevel::Supervisor => self.supervisor_sp,
            PrivilegeLevel::User => self.user_sp,
        };
        self.privilege = target;
    }

    /// PROMOTION: Used only by hardware exceptions/interrupts
    pub fn raise_privilege_on_exception(&mut self) {
        // 1. Save state
        self.saved_pc = self.pc;
        self.saved_sr = self.sr;
        
        // 2. Save current SP
        match self.privilege {
            PrivilegeLevel::Supervisor => self.supervisor_sp = self.sp,
            PrivilegeLevel::User => self.user_sp = self.sp,
            _ => {} // Already in Kernel
        }

        // 3. Jump to Kernel mode (Ring 0) to handle the event
        self.privilege = PrivilegeLevel::Kernel;
        self.sp = self.kernel_sp;
        self.pc = self.exception_handler;
    }

    /// Check if in kernel mode
    pub fn is_kernel_mode(&self) -> bool {
        self.privilege == PrivilegeLevel::Kernel
    }

    /// Check if in supervisor mode
    pub fn is_supervisor_mode(&self) -> bool {
        self.privilege == PrivilegeLevel::Supervisor
    }

    /// Check if in user mode
    pub fn is_user_mode(&self) -> bool {
        self.privilege == PrivilegeLevel::User
    }

    /// Reset all registers
    pub fn reset(&mut self) {
        self.gp_regs = [0; 16];
        self.pc = 0;
        self.sp = 0xFFFE;
        self.kernel_sp = 0xFFFE;
        self.supervisor_sp = 0xEFFE;
        self.user_sp = 0xDFFE;
        self.lr = 0;
        self.sr = 0;
        self.privilege = PrivilegeLevel::Kernel;
        self.saved_pc = 0;
        self.saved_sr = 0;
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
        result.push_str(&format!("  SP: 0x{:04X} ({})\n", self.sp, 
            if self.is_kernel_mode() { "kernel" } else { "user" }));
        result.push_str(&format!("  LR: 0x{:04X}\n", self.lr));
        result.push_str(&format!("  SR: 0x{:04X} ", self.sr));
        
        let flags = self.get_flags();
        result.push_str(&format!("[Z={} N={} C={} V={}]\n", 
            flags.zero as u8, flags.negative as u8,
            flags.carry as u8, flags.overflow as u8));

        result.push_str(&format!("\nPrivilege Mode: {:?}\n", self.privilege));
        
        result
    }

    /// Get console output
    pub fn get_console_output(&self) -> String {
        self.console.get_output()
    }

    /// Queue console input
    pub fn queue_console_input(&mut self, input: &str) {
        self.console.queue_input(input);
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
