"""
CVERE Disassembler - Converts hexadecimal machine code to assembly language
"""

from typing import List, Tuple, Optional
from dataclasses import dataclass


@dataclass
class DisassembledInstruction:
    """Represents a disassembled instruction"""
    address: int
    machine_code: int
    mnemonic: str
    operands: str
    comment: str = ""


class CVEREDisassembler:
    """Disassembler for CVERE ISA"""
    
    # Reverse opcode mappings
    OPCODES = {
        0x0: 'NOP',
        0x1: 'ADD',
        0x2: 'ADDI',
        0x3: 'SUB',
        0x4: 'AND',
        0x5: 'OR',
        0x6: 'XOR',
        0x7: 'NOT',
        0x8: 'SHL',
        0x9: 'SHR',
        0xA: 'LOAD',
        0xB: 'STORE',
        0xC: 'LOADI',
        0xD: 'JMP',
        0xE: 'BEQ',
        0xF: 'BNE',  # Also extended ops
    }
    
    EXTENDED_OPCODES = {
        0xFA: 'CALL',
        0xFB: 'RET',
        0xFC: 'PUSH',
        0xFD: 'POP',
    }
    
    R_TYPE = [0x1, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9]
    I_TYPE = [0x2, 0xC]
    M_TYPE = [0xA, 0xB]
    J_TYPE = [0xD]
    B_TYPE = [0xE, 0xF]
    
    def __init__(self):
        self.address_labels: dict[int, str] = {}
    
    def format_register(self, reg_num: int) -> str:
        """Format register number as string"""
        if reg_num < 16:
            return f"R{reg_num:X}"
        
        special_regs = {16: 'PC', 17: 'SP', 18: 'LR', 19: 'SR'}
        return special_regs.get(reg_num, f"R?{reg_num}")
    
    def format_immediate(self, imm: int, signed: bool = False, bits: int = 8) -> str:
        """Format immediate value"""
        if signed:
            # Sign extend
            mask = (1 << bits) - 1
            imm &= mask
            if imm & (1 << (bits - 1)):
                imm -= (1 << bits)
            return f"0x{imm & 0xFFFF:02X}" if imm >= 0 else f"-0x{(-imm) & 0xFFFF:02X}"
        else:
            return f"0x{imm:02X}"
    
    def decode_r_type(self, instruction: int, opcode: int) -> Tuple[str, str, str]:
        """Decode R-Type instruction"""
        rd = (instruction >> 8) & 0xF
        rs = (instruction >> 4) & 0xF
        rt = instruction & 0xF
        
        mnemonic = self.OPCODES[opcode]
        
        if mnemonic == 'NOT':
            operands = f"{self.format_register(rd)}, {self.format_register(rs)}"
            comment = f"Rd={rd:X}, Rs={rs:X}"
        else:
            operands = f"{self.format_register(rd)}, {self.format_register(rs)}, {self.format_register(rt)}"
            comment = f"Rd={rd:X}, Rs={rs:X}, Rt={rt:X}"
        
        return mnemonic, operands, comment
    
    def decode_i_type(self, instruction: int, opcode: int) -> Tuple[str, str, str]:
        """Decode I-Type instruction"""
        rd = (instruction >> 8) & 0xF
        imm = instruction & 0xFF
        
        mnemonic = self.OPCODES[opcode]
        operands = f"{self.format_register(rd)}, {self.format_immediate(imm)}"
        comment = f"Rd={rd:X}, Imm={imm:02X}"
        
        return mnemonic, operands, comment
    
    def decode_m_type(self, instruction: int, opcode: int) -> Tuple[str, str, str]:
        """Decode M-Type instruction"""
        rd = (instruction >> 8) & 0xF
        rs = (instruction >> 4) & 0xF
        offset = instruction & 0xF
        
        mnemonic = self.OPCODES[opcode]
        operands = f"{self.format_register(rd)}, {self.format_register(rs)}, 0x{offset:X}"
        comment = f"Rd={rd:X}, Rs={rs:X}, Off={offset:X}"
        
        return mnemonic, operands, comment
    
    def decode_j_type(self, instruction: int, opcode: int, address: int) -> Tuple[str, str, str]:
        """Decode J-Type instruction"""
        target_addr = instruction & 0xFFF
        
        mnemonic = self.OPCODES[opcode]
        
        # Check if we have a label for this address
        if target_addr in self.address_labels:
            operands = self.address_labels[target_addr]
        else:
            operands = f"0x{target_addr:03X}"
        
        comment = f"Target=0x{target_addr:03X}"
        
        return mnemonic, operands, comment
    
    def decode_b_type(self, instruction: int, opcode: int, address: int) -> Tuple[str, str, str]:
        """Decode B-Type instruction"""
        rc = (instruction >> 8) & 0xF
        offset = instruction & 0xFF
        
        # Calculate target address (sign-extended offset)
        if offset & 0x80:
            offset -= 0x100
        target_addr = address + 2 + (offset * 2)
        
        mnemonic = self.OPCODES.get(opcode, 'BNE')
        
        # Check if we have a label for this address
        if target_addr in self.address_labels:
            offset_str = self.address_labels[target_addr]
        else:
            offset_str = f"0x{target_addr:04X}"
        
        operands = f"{self.format_register(rc)}, {offset_str}"
        comment = f"Rc={rc:X}, Target=0x{target_addr:04X}"
        
        return mnemonic, operands, comment
    
    def disassemble_instruction(self, instruction: int, address: int) -> DisassembledInstruction:
        """Disassemble a single instruction"""
        # Special cases
        if instruction == 0x0000:
            return DisassembledInstruction(
                address=address,
                machine_code=instruction,
                mnemonic='NOP',
                operands='',
                comment='No operation'
            )
        
        if instruction == 0xFFFF:
            return DisassembledInstruction(
                address=address,
                machine_code=instruction,
                mnemonic='HALT',
                operands='',
                comment='Stop execution'
            )
        
        # Extract opcode
        opcode = (instruction >> 12) & 0xF
        
        # Check for extended instructions
        if opcode == 0xF:
            extended_op = (instruction >> 8) & 0xFF
            if extended_op in self.EXTENDED_OPCODES:
                return DisassembledInstruction(
                    address=address,
                    machine_code=instruction,
                    mnemonic=self.EXTENDED_OPCODES[extended_op],
                    operands='',
                    comment='Extended instruction'
                )
        
        # Decode based on instruction type
        if opcode in self.R_TYPE:
            mnemonic, operands, comment = self.decode_r_type(instruction, opcode)
        elif opcode in self.I_TYPE:
            mnemonic, operands, comment = self.decode_i_type(instruction, opcode)
        elif opcode in self.M_TYPE:
            mnemonic, operands, comment = self.decode_m_type(instruction, opcode)
        elif opcode in self.J_TYPE:
            mnemonic, operands, comment = self.decode_j_type(instruction, opcode, address)
        elif opcode in self.B_TYPE:
            mnemonic, operands, comment = self.decode_b_type(instruction, opcode, address)
        else:
            mnemonic, operands, comment = f"UNKNOWN_{opcode:X}", '', f"Unknown opcode: 0x{opcode:X}"
        
        return DisassembledInstruction(
            address=address,
            machine_code=instruction,
            mnemonic=mnemonic,
            operands=operands,
            comment=comment
        )
    
    def disassemble(self, machine_code: List[int], start_address: int = 0) -> List[DisassembledInstruction]:
        """Disassemble a list of machine code instructions"""
        result = []
        address = start_address
        
        for instruction in machine_code:
            disasm = self.disassemble_instruction(instruction, address)
            result.append(disasm)
            address += 2
        
        return result
    
    def disassemble_to_string(self, machine_code: List[int], start_address: int = 0, 
                             show_hex: bool = True, show_comments: bool = True) -> str:
        """Disassemble and format as string"""
        instructions = self.disassemble(machine_code, start_address)
        lines = []
        
        for instr in instructions:
            parts = []
            
            # Address
            parts.append(f"{instr.address:04X}:")
            
            # Machine code
            if show_hex:
                parts.append(f"{instr.machine_code:04X}")
            
            # Mnemonic and operands
            asm = f"{instr.mnemonic:6s}"
            if instr.operands:
                asm += f" {instr.operands}"
            parts.append(asm)
            
            # Comment
            if show_comments and instr.comment:
                parts.append(f"; {instr.comment}")
            
            lines.append("  ".join(parts))
        
        return '\n'.join(lines)
    
    def disassemble_from_file(self, filename: str, start_address: int = 0) -> List[DisassembledInstruction]:
        """Disassemble from binary file"""
        with open(filename, 'rb') as f:
            data = f.read()
        
        # Convert bytes to 16-bit instructions (little-endian)
        machine_code = []
        for i in range(0, len(data), 2):
            if i + 1 < len(data):
                instruction = int.from_bytes(data[i:i+2], byteorder='little')
                machine_code.append(instruction)
        
        return self.disassemble(machine_code, start_address)
    
    def add_label(self, address: int, label: str) -> None:
        """Add a label for an address"""
        self.address_labels[address] = label


def main():
    """Example usage"""
    # Sample machine code (from assembler example)
    machine_code = [
        0xC105,  # LOADI R1, 0x05
        0xC203,  # LOADI R2, 0x03
        0x1312,  # ADD R3, R1, R2
        0xB300,  # STORE R3, R0, 0x0
        0xFFFF,  # HALT
        0xC100,  # LOADI R1, 0x00
        0xC20A,  # LOADI R2, 0x0A
        0x2101,  # ADDI R1, 0x01
        0x3321,  # SUB R3, R2, R1
        0xF3FD,  # BNE R3, -3 (loop back)
        0xFFFF,  # HALT
    ]
    
    disassembler = CVEREDisassembler()
    
    # Add some labels
    disassembler.add_label(0x0000, "start")
    disassembler.add_label(0x000A, "loop_start")
    disassembler.add_label(0x000E, "loop")
    
    print("=== CVERE Disassembler ===\n")
    print("Machine Code Input:")
    for i, code in enumerate(machine_code):
        print(f"  {i*2:04X}: 0x{code:04X}")
    
    print("\n=== Disassembled Code ===")
    output = disassembler.disassemble_to_string(machine_code, show_hex=True, show_comments=True)
    print(output)
    
    print("\n=== Assembly Only (no hex, no comments) ===")
    output_clean = disassembler.disassemble_to_string(machine_code, show_hex=False, show_comments=False)
    print(output_clean)


if __name__ == "__main__":
    main()