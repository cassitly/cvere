// ============================================================================
// desktop/rust/src/syscall.rs
// System call handler for CVERE VM
// ============================================================================

/// System call numbers
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
pub enum Syscall {
    Exit = 0x00,           // Exit program
    PrintChar = 0x01,      // Print character to console
    PrintHex = 0x02,       // Print hex value
    ReadChar = 0x03,       // Read character from input
    GetTime = 0x04,        // Get cycle count
    Sleep = 0x05,          // Sleep for N cycles
    AllocMem = 0x06,       // Allocate memory
    FreeMem = 0x07,        // Free memory
    OpenFile = 0x08,       // Open file (simulated)
    CloseFile = 0x09,      // Close file
    ReadFile = 0x0A,       // Read from file
    WriteFile = 0x0B,      // Write to file
    Unknown = 0xFFFF,
}

impl Syscall {
    pub fn from_u16(value: u16) -> Self {
        match value {
            0x00 => Syscall::Exit,
            0x01 => Syscall::PrintChar,
            0x02 => Syscall::PrintHex,
            0x03 => Syscall::ReadChar,
            0x04 => Syscall::GetTime,
            0x05 => Syscall::Sleep,
            0x06 => Syscall::AllocMem,
            0x07 => Syscall::FreeMem,
            0x08 => Syscall::OpenFile,
            0x09 => Syscall::CloseFile,
            0x0A => Syscall::ReadFile,
            0x0B => Syscall::WriteFile,
            _ => Syscall::Unknown,
        }
    }
}

/// Console output buffer
pub struct Console {
    output: Vec<char>,
    input: Vec<char>,
}

impl Console {
    pub fn new() -> Self {
        Console {
            output: Vec::new(),
            input: Vec::new(),
        }
    }

    pub fn print_char(&mut self, c: char) {
        self.output.push(c);
        print!("{}", c);
    }

    pub fn print_hex(&mut self, value: u16) {
        let hex = format!("0x{:04X}", value);
        for c in hex.chars() {
            self.output.push(c);
        }
        print!("{}", hex);
    }

    pub fn read_char(&mut self) -> Option<char> {
        if self.input.is_empty() {
            None
        } else {
            Some(self.input.remove(0))
        }
    }

    pub fn queue_input(&mut self, input: &str) {
        for c in input.chars() {
            self.input.push(c);
        }
    }

    pub fn get_output(&self) -> String {
        self.output.iter().collect()
    }

    pub fn clear_output(&mut self) {
        self.output.clear();
    }
}