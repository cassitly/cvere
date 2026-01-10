"""
CVERE Assembler - Converts assembly language to hexadecimal machine code
"""

import re
from typing import List, Tuple, Dict, Optional
from dataclasses import dataclass


@dataclass
class Instruction:
    """Represents a parsed instruction"""
    label: Optional[str]
    opcode: str
    operands: List[str]
    line_num: int
    address: int = 0


class CVEREAssembler:
    """Assembler for CVERE ISA"""
    
    # Opcode mappings
    OPCODES = {
        'NOP': 0x0,
        'ADD': 0x1,
        'ADDI': 0x2,
        'SUB': 0x3,
        'AND': 0x4,
        'OR': 0x5,
        'XOR': 0x6,
        'NOT': 0x7,
        'SHL': 0x8,
        'SHR': 0x9,
        'LOAD': 0xA,
        'STORE': 0xB,
        'LOADI': 0xC,
        'JMP': 0xD,
        'BEQ': 0xE,
        'BNE': 0xF,
        'CALL': 0xFA,
        'RET': 0xFB,
        'PUSH': 0xFC,
        'POP': 0xFD,
        'HALT': 0xFF,
    }
    
    # Instruction format types
    R_TYPE = ['ADD', 'SUB', 'AND', 'OR', 'XOR', 'NOT', 'SHL', 'SHR']
    I_TYPE = ['ADDI', 'LOADI']
    M_TYPE = ['LOAD', 'STORE']
    J_TYPE = ['JMP']
    B_TYPE = ['BEQ', 'BNE']
    EXTENDED = ['CALL', 'RET', 'PUSH', 'POP']
    SPECIAL = ['NOP', 'HALT']
    
    def __init__(self):
        self.labels: Dict[str, int] = {}
        self.instructions: List[Instruction] = []
        self.current_address = 0
        
    def parse_register(self, reg_str: str) -> int:
        """Parse register string to register number"""
        reg_str = reg_str.strip().upper()
        
        # Handle special registers
        special_regs = {'PC': 16, 'SP': 17, 'LR': 18, 'SR': 19}
        if reg_str in special_regs:
            return special_regs[reg_str]
        
        # Handle Rx format
        if reg_str.startswith('R'):
            reg_num = reg_str[1:]
            if reg_num.startswith('0X'):
                return int(reg_num[2:], 16)
            else:
                return int(reg_num, 16) if len(reg_num) <= 1 else int(reg_num)
        
        raise ValueError(f"Invalid register: {reg_str}")
    
    def parse_immediate(self, imm_str: str) -> int:
        """Parse immediate value"""
        imm_str = imm_str.strip()
        
        # Hexadecimal
        if imm_str.startswith('0x') or imm_str.startswith('0X'):
            return int(imm_str[2:], 16)
        # Binary
        elif imm_str.startswith('0b') or imm_str.startswith('0B'):
            return int(imm_str[2:], 2)
        # Decimal
        else:
            return int(imm_str)
    
    def tokenize_line(self, line: str) -> Tuple[Optional[str], Optional[str], List[str]]:
        """Tokenize a single line into label, opcode, and operands"""
        # Remove comments
        line = re.sub(r';.*$', '', line).strip()
        
        if not line:
            return None, None, []
        
        # Check for label
        label = None
        if ':' in line:
            label, line = line.split(':', 1)
            label = label.strip()
            line = line.strip()
        
        if not line:
            return label, None, []
        
        # Split opcode and operands
        parts = line.split(None, 1)
        opcode = parts[0].upper()
        
        operands = []
        if len(parts) > 1:
            # Split operands by comma
            operands = [op.strip() for op in parts[1].split(',')]
        
        return label, opcode, operands
    
    def first_pass(self, source: str) -> None:
        """First pass: collect labels and instructions"""
        self.labels.clear()
        self.instructions.clear()
        self.current_address = 0
        
        for line_num, line in enumerate(source.split('\n'), 1):
            label, opcode, operands = self.tokenize_line(line)
            
            # Store label
            if label:
                self.labels[label] = self.current_address
            
            # Store instruction
            if opcode:
                instr = Instruction(
                    label=label,
                    opcode=opcode,
                    operands=operands,
                    line_num=line_num,
                    address=self.current_address
                )
                self.instructions.append(instr)
                
                # Calculate address increment
                if opcode in self.EXTENDED:
                    self.current_address += 4  # Two-word instruction
                elif opcode == 'HALT':
                    self.current_address += 2
                else:
                    self.current_address += 2  # Standard 16-bit instruction
    
    def encode_r_type(self, instr: Instruction) -> int:
        """Encode R-Type instruction"""
        opcode = self.OPCODES[instr.opcode]
        
        if instr.opcode == 'NOT':
            # NOT Rd, Rs (Rt unused, set to 0)
            rd = self.parse_register(instr.operands[0])
            rs = self.parse_register(instr.operands[1])
            rt = 0
        else:
            # Standard: OP Rd, Rs, Rt
            rd = self.parse_register(instr.operands[0])
            rs = self.parse_register(instr.operands[1])
            rt = self.parse_register(instr.operands[2])
        
        return (opcode << 12) | (rd << 8) | (rs << 4) | rt
    
    def encode_i_type(self, instr: Instruction) -> int:
        """Encode I-Type instruction"""
        opcode = self.OPCODES[instr.opcode]
        rd = self.parse_register(instr.operands[0])
        imm = self.parse_immediate(instr.operands[1]) & 0xFF  # 8-bit immediate
        
        return (opcode << 12) | (rd << 8) | imm
    
    def encode_m_type(self, instr: Instruction) -> int:
        """Encode M-Type instruction"""
        opcode = self.OPCODES[instr.opcode]
        rd = self.parse_register(instr.operands[0])
        rs = self.parse_register(instr.operands[1])
        offset = self.parse_immediate(instr.operands[2]) & 0xF  # 4-bit offset
        
        return (opcode << 12) | (rd << 8) | (rs << 4) | offset
    
    def encode_j_type(self, instr: Instruction) -> int:
        """Encode J-Type instruction"""
        opcode = self.OPCODES[instr.opcode]
        
        # Handle label or immediate address
        target = instr.operands[0]
        if target in self.labels:
            addr = self.labels[target]
        else:
            addr = self.parse_immediate(target)
        
        addr &= 0xFFF  # 12-bit address
        return (opcode << 12) | addr
    
    def encode_b_type(self, instr: Instruction) -> int:
        """Encode B-Type instruction"""
        opcode = self.OPCODES[instr.opcode]
        rc = self.parse_register(instr.operands[0])
        
        # Calculate offset
        target = instr.operands[1]
        if target in self.labels:
            offset = (self.labels[target] - (instr.address + 2)) // 2
        else:
            offset = self.parse_immediate(target)
        
        offset &= 0xFF  # 8-bit signed offset
        return (opcode << 12) | (rc << 8) | offset
    
    def second_pass(self) -> List[int]:
        """Second pass: encode instructions to machine code"""
        machine_code = []
        
        for instr in self.instructions:
            if instr.opcode in self.R_TYPE:
                code = self.encode_r_type(instr)
            elif instr.opcode in self.I_TYPE:
                code = self.encode_i_type(instr)
            elif instr.opcode in self.M_TYPE:
                code = self.encode_m_type(instr)
            elif instr.opcode in self.J_TYPE:
                code = self.encode_j_type(instr)
            elif instr.opcode in self.B_TYPE:
                code = self.encode_b_type(instr)
            elif instr.opcode == 'NOP':
                code = 0x0000
            elif instr.opcode == 'HALT':
                code = 0xFFFF
            elif instr.opcode in self.EXTENDED:
                # Extended instructions (placeholder for now)
                code = self.OPCODES[instr.opcode] << 8
                machine_code.append(code)
                machine_code.append(0x0000)  # Second word
                continue
            else:
                raise ValueError(f"Unknown opcode: {instr.opcode}")
            
            machine_code.append(code)
        
        return machine_code
    
    def assemble(self, source: str) -> List[int]:
        """Assemble source code to machine code"""
        self.first_pass(source)
        return self.second_pass()
    
    def assemble_to_hex(self, source: str) -> str:
        """Assemble source code and return as hex string"""
        machine_code = self.assemble(source)
        return '\n'.join(f'0x{code:04X}' for code in machine_code)
    
    def assemble_to_binary_file(self, source: str, filename: str) -> None:
        """Assemble and write to binary file"""
        machine_code = self.assemble(source)
        with open(filename, 'wb') as f:
            for code in machine_code:
                f.write(code.to_bytes(2, byteorder='little'))


def main():
    """Example usage"""
    sample_program = """
    ; Simple addition program
    start:
        LOADI R1, 0x05      ; R1 = 5
        LOADI R2, 0x03      ; R2 = 3
        ADD   R3, R1, R2    ; R3 = R1 + R2
        STORE R3, R0, 0x0   ; Store result
        HALT
    
    ; Loop counter
    loop_start:
        LOADI R1, 0x00      ; Counter = 0
        LOADI R2, 0x0A      ; Limit = 10
    loop:
        ADDI  R1, 0x01      ; Counter++
        SUB   R3, R2, R1    ; R3 = Limit - Counter
        BNE   R3, loop      ; If R3 != 0, continue
        HALT
    """
    
    assembler = CVEREAssembler()
    
    print("=== CVERE Assembler ===\n")
    print("Source Code:")
    print(sample_program)
    
    print("\n=== Machine Code ===")
    hex_output = assembler.assemble_to_hex(sample_program)
    print(hex_output)
    
    print("\n=== Symbol Table ===")
    for label, addr in assembler.labels.items():
        print(f"{label}: 0x{addr:04X}")


if __name__ == "__main__":
    main()