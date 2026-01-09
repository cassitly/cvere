// ============================================================================
// desktop/rust/src/decoder.rs
// Instruction decoder module for CVERE VM
// ============================================================================

/// Instruction format types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstructionFormat {
    RType,      // Register operations
    IType,      // Immediate operations
    MType,      // Memory operations
    JType,      // Jump operations
    BType,      // Branch operations
    Extended,   // Two-word instructions
    Special,    // Special cases (NOP, HALT)
}

/// Decoded instruction
#[derive(Debug, Clone)]
pub struct DecodedInstruction {
    pub format: InstructionFormat,
    pub opcode: u8,
    pub rd: u8,
    pub rs: u8,
    pub rt: u8,
    pub imm8: u8,
    pub offset: u8,
    pub addr12: u16,
    pub mnemonic: &'static str,
}

pub struct InstructionDecoder;

impl InstructionDecoder {
    /// Decode a 16-bit instruction
    pub fn decode(instruction: u16) -> DecodedInstruction {
        let opcode = ((instruction >> 12) & 0xF) as u8;
        let rd = ((instruction >> 8) & 0xF) as u8;
        let rs = ((instruction >> 4) & 0xF) as u8;
        let rt = (instruction & 0xF) as u8;
        let imm8 = (instruction & 0xFF) as u8;
        let offset = (instruction & 0xF) as u8;
        let addr12 = instruction & 0xFFF;

        // Determine format and mnemonic
        let (format, mnemonic) = Self::classify_instruction(instruction, opcode);

        DecodedInstruction {
            format,
            opcode,
            rd,
            rs,
            rt,
            imm8,
            offset,
            addr12,
            mnemonic,
        }
    }

    fn classify_instruction(instruction: u16, opcode: u8) -> (InstructionFormat, &'static str) {
        // Special cases first
        if instruction == 0x0000 {
            return (InstructionFormat::Special, "NOP");
        }
        if instruction == 0xFFFF {
            return (InstructionFormat::Special, "HALT");
        }

        // Check for extended instructions
        if opcode == 0xF {
            let extended_op = ((instruction >> 8) & 0xFF) as u8;
            match extended_op {
                0xF0 => return (InstructionFormat::Extended, "CALL"),
                0xF1 => return (InstructionFormat::Extended, "RET"),
                0xF2 => return (InstructionFormat::Extended, "PUSH"),
                0xF3 => return (InstructionFormat::Extended, "POP"),
                _ => return (InstructionFormat::BType, "BNE"),
            }
        }

        // Standard instruction decoding
        match opcode {
            0x0 => (InstructionFormat::Special, "NOP"),
            0x1 => (InstructionFormat::RType, "ADD"),
            0x2 => (InstructionFormat::IType, "ADDI"),
            0x3 => (InstructionFormat::RType, "SUB"),
            0x4 => (InstructionFormat::RType, "AND"),
            0x5 => (InstructionFormat::RType, "OR"),
            0x6 => (InstructionFormat::RType, "XOR"),
            0x7 => (InstructionFormat::RType, "NOT"),
            0x8 => (InstructionFormat::RType, "SHL"),
            0x9 => (InstructionFormat::RType, "SHR"),
            0xA => (InstructionFormat::MType, "LOAD"),
            0xB => (InstructionFormat::MType, "STORE"),
            0xC => (InstructionFormat::IType, "LOADI"),
            0xD => (InstructionFormat::JType, "JMP"),
            0xE => (InstructionFormat::BType, "BEQ"),
            _ => (InstructionFormat::Special, "UNKNOWN"),
        }
    }

    /// Get instruction format as string
    pub fn format_instruction(decoded: &DecodedInstruction) -> String {
        match decoded.format {
            InstructionFormat::RType => {
                if decoded.mnemonic == "NOT" {
                    format!("{} R{:X}, R{:X}", 
                        decoded.mnemonic, decoded.rd, decoded.rs)
                } else {
                    format!("{} R{:X}, R{:X}, R{:X}", 
                        decoded.mnemonic, decoded.rd, decoded.rs, decoded.rt)
                }
            }
            InstructionFormat::IType => {
                format!("{} R{:X}, 0x{:02X}", 
                    decoded.mnemonic, decoded.rd, decoded.imm8)
            }
            InstructionFormat::MType => {
                format!("{} R{:X}, R{:X}, 0x{:X}", 
                    decoded.mnemonic, decoded.rd, decoded.rs, decoded.offset)
            }
            InstructionFormat::JType => {
                format!("{} 0x{:03X}", decoded.mnemonic, decoded.addr12)
            }
            InstructionFormat::BType => {
                let offset = if decoded.imm8 & 0x80 != 0 {
                    (decoded.imm8 as i8) as i16
                } else {
                    decoded.imm8 as i16
                };
                format!("{} R{:X}, {}", decoded.mnemonic, decoded.rd, offset)
            }
            InstructionFormat::Extended => {
                format!("{}", decoded.mnemonic)
            }
            InstructionFormat::Special => {
                format!("{}", decoded.mnemonic)
            }
        }
    }

    /// Disassemble instruction with address
    pub fn disassemble(address: u16, instruction: u16) -> String {
        let decoded = Self::decode(instruction);
        format!("{:04X}: {:04X}  {}", 
            address, instruction, Self::format_instruction(&decoded))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_add() {
        let instr = 0x1312; // ADD R3, R1, R2
        let decoded = InstructionDecoder::decode(instr);
        assert_eq!(decoded.format, InstructionFormat::RType);
        assert_eq!(decoded.mnemonic, "ADD");
        assert_eq!(decoded.rd, 3);
        assert_eq!(decoded.rs, 1);
        assert_eq!(decoded.rt, 2);
    }

    #[test]
    fn test_decode_loadi() {
        let instr = 0xC105; // LOADI R1, 0x05
        let decoded = InstructionDecoder::decode(instr);
        assert_eq!(decoded.format, InstructionFormat::IType);
        assert_eq!(decoded.mnemonic, "LOADI");
        assert_eq!(decoded.rd, 1);
        assert_eq!(decoded.imm8, 0x05);
    }

    #[test]
    fn test_decode_halt() {
        let instr = 0xFFFF; // HALT
        let decoded = InstructionDecoder::decode(instr);
        assert_eq!(decoded.format, InstructionFormat::Special);
        assert_eq!(decoded.mnemonic, "HALT");
    }
}
