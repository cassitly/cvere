// ============================================================================
// desktop/rust/src/memory.rs
// Memory management module for CVERE VM
// ============================================================================

/// Memory subsystem for CVERE VM
pub struct Memory {
    data: Vec<u8>,
    size: usize,
}

impl Memory {
    /// Create new memory with specified size in bytes
    pub fn new(size: usize) -> Self {
        Memory {
            data: vec![0; size],
            size,
        }
    }

    /// Read a byte from memory
    pub fn read_byte(&self, address: usize) -> Result<u8, String> {
        if address >= self.size {
            return Err(format!("Memory read out of bounds: 0x{:04X}", address));
        }
        Ok(self.data[address])
    }

    /// Write a byte to memory
    pub fn write_byte(&mut self, address: usize, value: u8) -> Result<(), String> {
        if address >= self.size {
            return Err(format!("Memory write out of bounds: 0x{:04X}", address));
        }
        self.data[address] = value;
        Ok(())
    }

    /// Read a 16-bit word (little-endian)
    pub fn read_word(&self, address: usize) -> Result<u16, String> {
        if address + 1 >= self.size {
            return Err(format!("Memory word read out of bounds: 0x{:04X}", address));
        }
        let low = self.data[address] as u16;
        let high = self.data[address + 1] as u16;
        Ok((high << 8) | low)
    }

    /// Write a 16-bit word (little-endian)
    pub fn write_word(&mut self, address: usize, value: u16) -> Result<(), String> {
        if address + 1 >= self.size {
            return Err(format!("Memory word write out of bounds: 0x{:04X}", address));
        }
        self.data[address] = (value & 0xFF) as u8;
        self.data[address + 1] = (value >> 8) as u8;
        Ok(())
    }

    /// Load program into memory starting at address
    pub fn load_program(&mut self, program: &[u16], start_address: usize) -> Result<(), String> {
        let mut addr = start_address;
        for &instruction in program {
            self.write_word(addr, instruction)?;
            addr += 2;
        }
        Ok(())
    }

    /// Get memory dump as hex string
    pub fn dump(&self, start: usize, length: usize) -> String {
        let mut result = String::new();
        let end = std::cmp::min(start + length, self.size);
        
        for addr in (start..end).step_by(16) {
            result.push_str(&format!("{:04X}: ", addr));
            
            // Hex bytes
            for i in 0..16 {
                if addr + i < end {
                    result.push_str(&format!("{:02X} ", self.data[addr + i]));
                } else {
                    result.push_str("   ");
                }
            }
            
            // ASCII representation
            result.push_str(" |");
            for i in 0..16 {
                if addr + i < end {
                    let byte = self.data[addr + i];
                    if byte >= 0x20 && byte <= 0x7E {
                        result.push(byte as char);
                    } else {
                        result.push('.');
                    }
                }
            }
            result.push_str("|\n");
        }
        
        result
    }

    /// Clear all memory
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    /// Get memory size
    pub fn size(&self) -> usize {
        self.size
    }
}
