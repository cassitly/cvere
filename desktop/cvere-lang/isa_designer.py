"""
CVERE ISA Designer - Interactive tool for designing and testing instruction encodings
"""

import json
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass, asdict
from enum import Enum


class InstructionFormat(Enum):
    """Instruction format types"""
    R_TYPE = "R-Type"      # Register operations
    I_TYPE = "I-Type"      # Immediate operations
    M_TYPE = "M-Type"      # Memory operations
    J_TYPE = "J-Type"      # Jump operations
    B_TYPE = "B-Type"      # Branch operations
    EXTENDED = "Extended"  # Two-word instructions
    SPECIAL = "Special"    # Special cases


@dataclass
class InstructionSpec:
    """Specification for a single instruction"""
    mnemonic: str
    opcode: int
    format: InstructionFormat
    description: str
    operands: List[str]
    example: str
    cycles: int = 1
    
    def to_dict(self) -> dict:
        d = asdict(self)
        d['format'] = self.format.value
        return d
    
    @classmethod
    def from_dict(cls, data: dict) -> 'InstructionSpec':
        data['format'] = InstructionFormat(data['format'])
        return cls(**data)


class ISADesigner:
    """Tool for designing and managing ISA specifications"""
    
    def __init__(self):
        self.instructions: Dict[str, InstructionSpec] = {}
        self.opcode_map: Dict[int, str] = {}
        self.load_default_isa()
    
    def load_default_isa(self) -> None:
        """Load the default CVERE ISA"""
        default_instructions = [
            InstructionSpec("NOP", 0x0, InstructionFormat.SPECIAL, 
                          "No operation", [], "NOP", 1),
            InstructionSpec("ADD", 0x1, InstructionFormat.R_TYPE,
                          "Add two registers", ["Rd", "Rs", "Rt"], 
                          "ADD R3, R1, R2", 1),
            InstructionSpec("ADDI", 0x2, InstructionFormat.I_TYPE,
                          "Add immediate to register", ["Rd", "Imm8"],
                          "ADDI R3, 0x42", 1),
            InstructionSpec("SUB", 0x3, InstructionFormat.R_TYPE,
                          "Subtract registers", ["Rd", "Rs", "Rt"],
                          "SUB R3, R1, R2", 1),
            InstructionSpec("AND", 0x4, InstructionFormat.R_TYPE,
                          "Bitwise AND", ["Rd", "Rs", "Rt"],
                          "AND R3, R1, R2", 1),
            InstructionSpec("OR", 0x5, InstructionFormat.R_TYPE,
                          "Bitwise OR", ["Rd", "Rs", "Rt"],
                          "OR R3, R1, R2", 1),
            InstructionSpec("XOR", 0x6, InstructionFormat.R_TYPE,
                          "Bitwise XOR", ["Rd", "Rs", "Rt"],
                          "XOR R3, R1, R2", 1),
            InstructionSpec("NOT", 0x7, InstructionFormat.R_TYPE,
                          "Bitwise NOT", ["Rd", "Rs"],
                          "NOT R3, R1", 1),
            InstructionSpec("SHL", 0x8, InstructionFormat.R_TYPE,
                          "Shift left", ["Rd", "Rs", "Rt"],
                          "SHL R3, R1, R2", 1),
            InstructionSpec("SHR", 0x9, InstructionFormat.R_TYPE,
                          "Shift right", ["Rd", "Rs", "Rt"],
                          "SHR R3, R1, R2", 1),
            InstructionSpec("LOAD", 0xA, InstructionFormat.M_TYPE,
                          "Load from memory", ["Rd", "Rs", "Offset"],
                          "LOAD R3, R1, 0x4", 2),
            InstructionSpec("STORE", 0xB, InstructionFormat.M_TYPE,
                          "Store to memory", ["Rd", "Rs", "Offset"],
                          "STORE R3, R1, 0x4", 2),
            InstructionSpec("LOADI", 0xC, InstructionFormat.I_TYPE,
                          "Load immediate", ["Rd", "Imm8"],
                          "LOADI R3, 0x42", 1),
            InstructionSpec("JMP", 0xD, InstructionFormat.J_TYPE,
                          "Unconditional jump", ["Addr12"],
                          "JMP 0x100", 2),
            InstructionSpec("BEQ", 0xE, InstructionFormat.B_TYPE,
                          "Branch if equal to zero", ["Rc", "Offset"],
                          "BEQ R1, loop", 1),
            InstructionSpec("BNE", 0xF, InstructionFormat.B_TYPE,
                          "Branch if not equal to zero", ["Rc", "Offset"],
                          "BNE R1, loop", 1),
            InstructionSpec("HALT", 0xFF, InstructionFormat.SPECIAL,
                          "Halt execution", [],
                          "HALT", 1),
        ]
        
        for instr in default_instructions:
            self.add_instruction(instr)
    
    def add_instruction(self, spec: InstructionSpec) -> None:
        """Add or update an instruction specification"""
        if spec.opcode in self.opcode_map and self.opcode_map[spec.opcode] != spec.mnemonic:
            raise ValueError(f"Opcode 0x{spec.opcode:X} already used by {self.opcode_map[spec.opcode]}")
        
        self.instructions[spec.mnemonic] = spec
        self.opcode_map[spec.opcode] = spec.mnemonic
    
    def remove_instruction(self, mnemonic: str) -> None:
        """Remove an instruction"""
        if mnemonic in self.instructions:
            spec = self.instructions[mnemonic]
            del self.opcode_map[spec.opcode]
            del self.instructions[mnemonic]
    
    def get_instruction(self, mnemonic: str) -> Optional[InstructionSpec]:
        """Get instruction specification"""
        return self.instructions.get(mnemonic)
    
    def encode_instruction(self, mnemonic: str, operands: List[int]) -> int:
        """Encode an instruction with given operands"""
        spec = self.get_instruction(mnemonic)
        if not spec:
            raise ValueError(f"Unknown instruction: {mnemonic}")
        
        if spec.format == InstructionFormat.R_TYPE:
            return self._encode_r_type(spec.opcode, operands)
        elif spec.format == InstructionFormat.I_TYPE:
            return self._encode_i_type(spec.opcode, operands)
        elif spec.format == InstructionFormat.M_TYPE:
            return self._encode_m_type(spec.opcode, operands)
        elif spec.format == InstructionFormat.J_TYPE:
            return self._encode_j_type(spec.opcode, operands)
        elif spec.format == InstructionFormat.B_TYPE:
            return self._encode_b_type(spec.opcode, operands)
        elif spec.format == InstructionFormat.SPECIAL:
            if mnemonic == "NOP":
                return 0x0000
            elif mnemonic == "HALT":
                return 0xFFFF
        
        return 0
    
    def _encode_r_type(self, opcode: int, operands: List[int]) -> int:
        """Encode R-Type: [Op:4][Rd:4][Rs:4][Rt:4]"""
        rd = operands[0] & 0xF
        rs = operands[1] & 0xF
        rt = operands[2] if len(operands) > 2 else 0
        rt &= 0xF
        return (opcode << 12) | (rd << 8) | (rs << 4) | rt
    
    def _encode_i_type(self, opcode: int, operands: List[int]) -> int:
        """Encode I-Type: [Op:4][Rd:4][Imm:8]"""
        rd = operands[0] & 0xF
        imm = operands[1] & 0xFF
        return (opcode << 12) | (rd << 8) | imm
    
    def _encode_m_type(self, opcode: int, operands: List[int]) -> int:
        """Encode M-Type: [Op:4][Rd:4][Rs:4][Off:4]"""
        rd = operands[0] & 0xF
        rs = operands[1] & 0xF
        offset = operands[2] & 0xF
        return (opcode << 12) | (rd << 8) | (rs << 4) | offset
    
    def _encode_j_type(self, opcode: int, operands: List[int]) -> int:
        """Encode J-Type: [Op:4][Addr:12]"""
        addr = operands[0] & 0xFFF
        return (opcode << 12) | addr
    
    def _encode_b_type(self, opcode: int, operands: List[int]) -> int:
        """Encode B-Type: [Op:4][Rc:4][Offset:8]"""
        rc = operands[0] & 0xF
        offset = operands[1] & 0xFF
        return (opcode << 12) | (rc << 8) | offset
    
    def decode_instruction(self, machine_code: int) -> Tuple[str, List[int]]:
        """Decode machine code to instruction and operands"""
        if machine_code == 0x0000:
            return "NOP", []
        if machine_code == 0xFFFF:
            return "HALT", []
        
        opcode = (machine_code >> 12) & 0xF
        
        if opcode not in self.opcode_map:
            return f"UNKNOWN_0x{opcode:X}", []
        
        mnemonic = self.opcode_map[opcode]
        spec = self.instructions[mnemonic]
        
        if spec.format == InstructionFormat.R_TYPE:
            rd = (machine_code >> 8) & 0xF
            rs = (machine_code >> 4) & 0xF
            rt = machine_code & 0xF
            return mnemonic, [rd, rs, rt]
        elif spec.format == InstructionFormat.I_TYPE:
            rd = (machine_code >> 8) & 0xF
            imm = machine_code & 0xFF
            return mnemonic, [rd, imm]
        elif spec.format == InstructionFormat.M_TYPE:
            rd = (machine_code >> 8) & 0xF
            rs = (machine_code >> 4) & 0xF
            offset = machine_code & 0xF
            return mnemonic, [rd, rs, offset]
        elif spec.format == InstructionFormat.J_TYPE:
            addr = machine_code & 0xFFF
            return mnemonic, [addr]
        elif spec.format == InstructionFormat.B_TYPE:
            rc = (machine_code >> 8) & 0xF
            offset = machine_code & 0xFF
            return mnemonic, [rc, offset]
        
        return mnemonic, []
    
    def generate_documentation(self) -> str:
        """Generate markdown documentation for the ISA"""
        doc = ["# CVERE ISA Reference\n"]
        
        # Group by format
        by_format: Dict[InstructionFormat, List[InstructionSpec]] = {}
        for spec in self.instructions.values():
            if spec.format not in by_format:
                by_format[spec.format] = []
            by_format[spec.format].append(spec)
        
        # Generate sections
        for fmt, instrs in sorted(by_format.items(), key=lambda x: x[0].value):
            doc.append(f"\n## {fmt.value} Instructions\n")
            doc.append("| Opcode | Mnemonic | Operands | Description | Example | Cycles |")
            doc.append("|--------|----------|----------|-------------|---------|--------|")
            
            for spec in sorted(instrs, key=lambda x: x.opcode):
                operands = ", ".join(spec.operands)
                doc.append(
                    f"| 0x{spec.opcode:X} | {spec.mnemonic} | {operands} | "
                    f"{spec.description} | `{spec.example}` | {spec.cycles} |"
                )
        
        return "\n".join(doc)
    
    def export_to_json(self, filename: str) -> None:
        """Export ISA to JSON file"""
        data = {
            "version": "1.0",
            "instructions": [spec.to_dict() for spec in self.instructions.values()]
        }
        with open(filename, 'w') as f:
            json.dump(data, f, indent=2)
    
    def import_from_json(self, filename: str) -> None:
        """Import ISA from JSON file"""
        with open(filename, 'r') as f:
            data = json.load(f)
        
        self.instructions.clear()
        self.opcode_map.clear()
        
        for instr_data in data['instructions']:
            spec = InstructionSpec.from_dict(instr_data)
            self.add_instruction(spec)
    
    def analyze_encoding_space(self) -> Dict[str, any]:
        """Analyze the encoding space usage"""
        total_opcodes = 16  # 4-bit opcode space
        used_opcodes = len(set(self.opcode_map.keys()))
        
        format_counts = {}
        for spec in self.instructions.values():
            fmt = spec.format.value
            format_counts[fmt] = format_counts.get(fmt, 0) + 1
        
        return {
            "total_instructions": len(self.instructions),
            "used_opcodes": used_opcodes,
            "available_opcodes": total_opcodes - used_opcodes,
            "utilization_percent": (used_opcodes / total_opcodes) * 100,
            "format_distribution": format_counts,
        }
    
    def visualize_instruction_encoding(self, mnemonic: str) -> str:
        """Generate a visual representation of instruction encoding"""
        spec = self.get_instruction(mnemonic)
        if not spec:
            return f"Unknown instruction: {mnemonic}"
        
        visual = [f"\n{mnemonic} - {spec.description}"]
        visual.append(f"Opcode: 0x{spec.opcode:X} | Format: {spec.format.value}")
        visual.append(f"Example: {spec.example}\n")
        
        # Show bit layout
        if spec.format == InstructionFormat.R_TYPE:
            visual.append("┌────────┬────────┬────────┬────────┐")
            visual.append("│   Op   │   Rd   │   Rs   │   Rt   │")
            visual.append("│ (4bit) │ (4bit) │ (4bit) │ (4bit) │")
            visual.append("└────────┴────────┴────────┴────────┘")
            visual.append(" 15-12    11-8     7-4      3-0")
        elif spec.format == InstructionFormat.I_TYPE:
            visual.append("┌────────┬────────┬─────────────────┐")
            visual.append("│   Op   │   Rd   │      Imm8       │")
            visual.append("│ (4bit) │ (4bit) │     (8bit)      │")
            visual.append("└────────┴────────┴─────────────────┘")
            visual.append(" 15-12    11-8          7-0")
        elif spec.format == InstructionFormat.M_TYPE:
            visual.append("┌────────┬────────┬────────┬────────┐")
            visual.append("│   Op   │   Rd   │   Rs   │  Off   │")
            visual.append("│ (4bit) │ (4bit) │ (4bit) │ (4bit) │")
            visual.append("└────────┴────────┴────────┴────────┘")
            visual.append(" 15-12    11-8     7-4      3-0")
        elif spec.format == InstructionFormat.J_TYPE:
            visual.append("┌────────┬─────────────────────────────┐")
            visual.append("│   Op   │          Addr12             │")
            visual.append("│ (4bit) │          (12bit)            │")
            visual.append("└────────┴─────────────────────────────┘")
            visual.append(" 15-12              11-0")
        elif spec.format == InstructionFormat.B_TYPE:
            visual.append("┌────────┬────────┬─────────────────┐")
            visual.append("│   Op   │   Rc   │    Offset8      │")
            visual.append("│ (4bit) │ (4bit) │     (8bit)      │")
            visual.append("└────────┴────────┴─────────────────┘")
            visual.append(" 15-12    11-8          7-0")
        
        return "\n".join(visual)


def main():
    """Example usage"""
    designer = ISADesigner()
    
    print("=== CVERE ISA Designer ===\n")
    
    # Show all instructions
    print("Loaded Instructions:")
    for mnemonic, spec in sorted(designer.instructions.items()):
        print(f"  {spec.mnemonic:6s} (0x{spec.opcode:02X}) - {spec.description}")
    
    # Analyze encoding space
    print("\n=== Encoding Space Analysis ===")
    analysis = designer.analyze_encoding_space()
    for key, value in analysis.items():
        print(f"  {key}: {value}")
    
    # Visualize some instructions
    print("\n=== Instruction Encoding Visualizations ===")
    for instr in ["ADD", "LOADI", "LOAD", "JMP", "BEQ"]:
        print(designer.visualize_instruction_encoding(instr))
    
    # Test encoding/decoding
    print("\n=== Encoding/Decoding Test ===")
    test_cases = [
        ("ADD", [3, 1, 2]),    # ADD R3, R1, R2
        ("LOADI", [5, 0x42]),  # LOADI R5, 0x42
        ("LOAD", [3, 2, 4]),   # LOAD R3, R2, 0x4
    ]
    
    for mnemonic, operands in test_cases:
        encoded = designer.encode_instruction(mnemonic, operands)
        decoded_mn, decoded_ops = designer.decode_instruction(encoded)
        print(f"{mnemonic} {operands} -> 0x{encoded:04X} -> {decoded_mn} {decoded_ops}")
    
    # Generate documentation
    print("\n=== Generated Documentation ===")
    print(designer.generate_documentation())


if __name__ == "__main__":
    main()