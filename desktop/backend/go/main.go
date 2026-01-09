// ============================================================================
// backend/go/main.go
// Main API server entry point
// ============================================================================

package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"

	"cvere/simulator"
)

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	// API routes
	http.HandleFunc("/api/health", healthHandler)
	http.HandleFunc("/api/assemble", assembleHandler)
	http.HandleFunc("/api/disassemble", disassembleHandler)
	http.HandleFunc("/api/simulate", simulateHandler)
	http.HandleFunc("/api/step", stepHandler)
	http.HandleFunc("/api/reset", resetHandler)

	// CORS middleware
	handler := corsMiddleware(http.DefaultServeMux)

	log.Printf("CVERE Backend starting on port %s", port)
	log.Fatal(http.ListenAndServe(":"+port, handler))
}

func corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization")

		if r.Method == "OPTIONS" {
			w.WriteHeader(http.StatusOK)
			return
		}

		next.ServeHTTP(w, r)
	})
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
	json.NewEncoder(w).Encode(map[string]string{
		"status":  "ok",
		"service": "CVERE Backend",
		"version": "1.0.0",
	})
}

func assembleHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req struct {
		Source string `json:"source"`
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	assembler := simulator.NewAssembler()
	machineCode, err := assembler.Assemble(req.Source)
	if err != nil {
		json.NewEncoder(w).Encode(map[string]interface{}{
			"error": err.Error(),
		})
		return
	}

	json.NewEncoder(w).Encode(map[string]interface{}{
		"machineCode": machineCode,
		"labels":      assembler.Labels,
	})
}

func disassembleHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req struct {
		MachineCode []uint16 `json:"machineCode"`
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	disassembler := simulator.NewDisassembler()
	instructions := disassembler.Disassemble(req.MachineCode, 0)

	json.NewEncoder(w).Encode(map[string]interface{}{
		"instructions": instructions,
	})
}

func simulateHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req struct {
		MachineCode []uint16 `json:"machineCode"`
		MaxCycles   int      `json:"maxCycles"`
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	if req.MaxCycles == 0 {
		req.MaxCycles = 1000
	}

	vm := simulator.NewVM()
	vm.LoadProgram(req.MachineCode, 0)

	history := []simulator.VMState{}
	for i := 0; i < req.MaxCycles && !vm.Halted; i++ {
		if err := vm.Step(); err != nil {
			break
		}
		history = append(history, vm.GetState())
	}

	json.NewEncoder(w).Encode(map[string]interface{}{
		"history":     history,
		"finalState":  vm.GetState(),
		"cycleCount":  vm.CycleCount,
		"halted":      vm.Halted,
	})
}

func stepHandler(w http.ResponseWriter, r *http.Request) {
	// This would require session management - simplified for now
	json.NewEncoder(w).Encode(map[string]string{
		"message": "Step requires session management - use simulate endpoint",
	})
}

func resetHandler(w http.ResponseWriter, r *http.Request) {
	json.NewEncoder(w).Encode(map[string]string{
		"status": "ok",
	})
}

// ============================================================================
// backend/go/simulator/vm.go
// Virtual Machine implementation in Go
// ============================================================================

package simulator

import "fmt"

type VM struct {
	Registers  [16]uint16
	PC         uint16
	SP         uint16
	LR         uint16
	SR         uint16
	Memory     [32768]uint16 // 64KB as 16-bit words
	Halted     bool
	CycleCount int
}

type VMState struct {
	Registers  [16]uint16 `json:"registers"`
	PC         uint16     `json:"pc"`
	SP         uint16     `json:"sp"`
	LR         uint16     `json:"lr"`
	SR         uint16     `json:"sr"`
	CycleCount int        `json:"cycleCount"`
	Halted     bool       `json:"halted"`
}

func NewVM() *VM {
	return &VM{
		PC: 0,
		SP: 0xFFFE,
	}
}

func (vm *VM) LoadProgram(program []uint16, startAddr uint16) {
	for i, instruction := range program {
		vm.Memory[startAddr/2+uint16(i)] = instruction
	}
}

func (vm *VM) Fetch() uint16 {
	if vm.PC/2 >= uint16(len(vm.Memory)) {
		return 0xFFFF
	}
	instruction := vm.Memory[vm.PC/2]
	vm.PC += 2
	return instruction
}

func (vm *VM) UpdateFlags(result uint16) {
	vm.SR &= 0xFFF0 // Clear flags
	if result == 0 {
		vm.SR |= 1 << 0 // Zero flag
	}
	if result&0x8000 != 0 {
		vm.SR |= 1 << 1 // Negative flag
	}
}

func (vm *VM) UpdateFlagsWithCarry(result uint32) {
	vm.UpdateFlags(uint16(result))
	if result > 0xFFFF {
		vm.SR |= 1 << 2 // Carry flag
	}
}

func (vm *VM) Step() error {
	if vm.Halted {
		return fmt.Errorf("VM is halted")
	}

	instruction := vm.Fetch()
	vm.CycleCount++

	opcode := (instruction >> 12) & 0xF
	rd := uint8((instruction >> 8) & 0xF)
	rs := uint8((instruction >> 4) & 0xF)
	rt := uint8(instruction & 0xF)
	imm8 := uint8(instruction & 0xFF)
	offset := uint8(instruction & 0xF)
	addr12 := instruction & 0xFFF

	switch opcode {
	case 0x0: // NOP
		// Do nothing

	case 0x1: // ADD
		result := uint32(vm.Registers[rs]) + uint32(vm.Registers[rt])
		vm.Registers[rd] = uint16(result)
		vm.Registers[0] = 0
		vm.UpdateFlagsWithCarry(result)

	case 0x2: // ADDI
		result := uint32(vm.Registers[rd]) + uint32(imm8)
		vm.Registers[rd] = uint16(result)
		vm.Registers[0] = 0
		vm.UpdateFlagsWithCarry(result)

	case 0x3: // SUB
		result := vm.Registers[rs] - vm.Registers[rt]
		vm.Registers[rd] = result
		vm.Registers[0] = 0
		vm.UpdateFlags(result)

	case 0x4: // AND
		result := vm.Registers[rs] & vm.Registers[rt]
		vm.Registers[rd] = result
		vm.Registers[0] = 0
		vm.UpdateFlags(result)

	case 0x5: // OR
		result := vm.Registers[rs] | vm.Registers[rt]
		vm.Registers[rd] = result
		vm.Registers[0] = 0
		vm.UpdateFlags(result)

	case 0x6: // XOR
		result := vm.Registers[rs] ^ vm.Registers[rt]
		vm.Registers[rd] = result
		vm.Registers[0] = 0
		vm.UpdateFlags(result)

	case 0x7: // NOT
		result := ^vm.Registers[rs]
		vm.Registers[rd] = result
		vm.Registers[0] = 0
		vm.UpdateFlags(result)

	case 0x8: // SHL
		shift := vm.Registers[rt] & 0xF
		result := vm.Registers[rs] << shift
		vm.Registers[rd] = result
		vm.Registers[0] = 0
		vm.UpdateFlags(result)

	case 0x9: // SHR
		shift := vm.Registers[rt] & 0xF
		result := vm.Registers[rs] >> shift
		vm.Registers[rd] = result
		vm.Registers[0] = 0
		vm.UpdateFlags(result)

	case 0xA: // LOAD
		address := (vm.Registers[rs] + uint16(offset)*2) / 2
		if address < uint16(len(vm.Memory)) {
			vm.Registers[rd] = vm.Memory[address]
			vm.Registers[0] = 0
		}

	case 0xB: // STORE
		address := (vm.Registers[rs] + uint16(offset)*2) / 2
		if address < uint16(len(vm.Memory)) {
			vm.Memory[address] = vm.Registers[rd]
		}

	case 0xC: // LOADI
		var value uint16
		if imm8&0x80 != 0 {
			value = uint16(imm8) | 0xFF00 // Sign extend
		} else {
			value = uint16(imm8)
		}
		vm.Registers[rd] = value
		vm.Registers[0] = 0

	case 0xD: // JMP
		vm.PC = addr12

	case 0xE: // BEQ
		var signedOffset int16
		if imm8&0x80 != 0 {
			signedOffset = int16(int8(imm8))
		} else {
			signedOffset = int16(imm8)
		}
		if vm.Registers[rd] == 0 {
			vm.PC = uint16(int32(vm.PC) + int32(signedOffset)*2)
		}

	case 0xF: // BNE or HALT
		if instruction == 0xFFFF {
			vm.Halted = true
		} else {
			var signedOffset int16
			if imm8&0x80 != 0 {
				signedOffset = int16(int8(imm8))
			} else {
				signedOffset = int16(imm8)
			}
			if vm.Registers[rd] != 0 {
				vm.PC = uint16(int32(vm.PC) + int32(signedOffset)*2)
			}
		}

	default:
		return fmt.Errorf("unknown opcode: 0x%X", opcode)
	}

	return nil
}

func (vm *VM) GetState() VMState {
	return VMState{
		Registers:  vm.Registers,
		PC:         vm.PC,
		SP:         vm.SP,
		LR:         vm.LR,
		SR:         vm.SR,
		CycleCount: vm.CycleCount,
		Halted:     vm.Halted,
	}
}

func (vm *VM) Reset() {
	vm.Registers = [16]uint16{}
	vm.PC = 0
	vm.SP = 0xFFFE
	vm.LR = 0
	vm.SR = 0
	vm.Halted = false
	vm.CycleCount = 0
}

// ============================================================================
// backend/go/simulator/assembler.go
// Assembler implementation in Go
// ============================================================================

package simulator

import (
	"fmt"
	"strconv"
	"strings"
)

type Assembler struct {
	Labels       map[string]uint16
	Instructions []Instruction
	Address      uint16
}

type Instruction struct {
	Label    string
	Opcode   string
	Operands []string
	Address  uint16
}

var Opcodes = map[string]uint8{
	"NOP":   0x0,
	"ADD":   0x1,
	"ADDI":  0x2,
	"SUB":   0x3,
	"AND":   0x4,
	"OR":    0x5,
	"XOR":   0x6,
	"NOT":   0x7,
	"SHL":   0x8,
	"SHR":   0x9,
	"LOAD":  0xA,
	"STORE": 0xB,
	"LOADI": 0xC,
	"JMP":   0xD,
	"BEQ":   0xE,
	"BNE":   0xF,
	"HALT":  0xFF,
}

func NewAssembler() *Assembler {
	return &Assembler{
		Labels:       make(map[string]uint16),
		Instructions: []Instruction{},
		Address:      0,
	}
}

func (a *Assembler) Assemble(source string) ([]uint16, error) {
	// First pass: collect labels
	if err := a.firstPass(source); err != nil {
		return nil, err
	}

	// Second pass: encode instructions
	return a.secondPass()
}

func (a *Assembler) firstPass(source string) error {
	a.Labels = make(map[string]uint16)
	a.Instructions = []Instruction{}
	a.Address = 0

	lines := strings.Split(source, "\n")
	for _, line := range lines {
		// Remove comments
		if idx := strings.Index(line, ";"); idx != -1 {
			line = line[:idx]
		}
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}

		var label, opcode string
		var operands []string

		// Check for label
		if strings.Contains(line, ":") {
			parts := strings.SplitN(line, ":", 2)
			label = strings.TrimSpace(parts[0])
			a.Labels[label] = a.Address
			line = strings.TrimSpace(parts[1])
		}

		if line != "" {
			parts := strings.Fields(line)
			opcode = strings.ToUpper(parts[0])

			if len(parts) > 1 {
				opStr := strings.Join(parts[1:], "")
				operands = strings.Split(opStr, ",")
				for i := range operands {
					operands[i] = strings.TrimSpace(operands[i])
				}
			}

			a.Instructions = append(a.Instructions, Instruction{
				Label:    label,
				Opcode:   opcode,
				Operands: operands,
				Address:  a.Address,
			})

			a.Address += 2
		}
	}

	return nil
}

func (a *Assembler) secondPass() ([]uint16, error) {
	machineCode := []uint16{}

	for _, instr := range a.Instructions {
		code, err := a.encodeInstruction(instr)
		if err != nil {
			return nil, err
		}
		machineCode = append(machineCode, code)
	}

	return machineCode, nil
}

func (a *Assembler) encodeInstruction(instr Instruction) (uint16, error) {
	opcode, ok := Opcodes[instr.Opcode]
	if !ok {
		return 0, fmt.Errorf("unknown opcode: %s", instr.Opcode)
	}

	if instr.Opcode == "NOP" {
		return 0x0000, nil
	}
	if instr.Opcode == "HALT" {
		return 0xFFFF, nil
	}

	switch instr.Opcode {
	case "ADD", "SUB", "AND", "OR", "XOR", "SHL", "SHR":
		rd := a.parseRegister(instr.Operands[0])
		rs := a.parseRegister(instr.Operands[1])
		rt := a.parseRegister(instr.Operands[2])
		return (uint16(opcode) << 12) | (uint16(rd) << 8) | (uint16(rs) << 4) | uint16(rt), nil

	case "NOT":
		rd := a.parseRegister(instr.Operands[0])
		rs := a.parseRegister(instr.Operands[1])
		return (uint16(opcode) << 12) | (uint16(rd) << 8) | (uint16(rs) << 4), nil

	case "ADDI", "LOADI":
		rd := a.parseRegister(instr.Operands[0])
		imm := a.parseImmediate(instr.Operands[1]) & 0xFF
		return (uint16(opcode) << 12) | (uint16(rd) << 8) | uint16(imm), nil

	case "LOAD", "STORE":
		rd := a.parseRegister(instr.Operands[0])
		rs := a.parseRegister(instr.Operands[1])
		offset := a.parseImmediate(instr.Operands[2]) & 0xF
		return (uint16(opcode) << 12) | (uint16(rd) << 8) | (uint16(rs) << 4) | uint16(offset), nil

	case "JMP":
		var addr uint16
		if labelAddr, ok := a.Labels[instr.Operands[0]]; ok {
			addr = labelAddr
		} else {
			addr = uint16(a.parseImmediate(instr.Operands[0]))
		}
		return (uint16(opcode) << 12) | (addr & 0xFFF), nil

	case "BEQ", "BNE":
		rc := a.parseRegister(instr.Operands[0])
		var offset uint16
		if labelAddr, ok := a.Labels[instr.Operands[1]]; ok {
			offset = uint16(int16(labelAddr-instr.Address-2) / 2)
		} else {
			offset = uint16(a.parseImmediate(instr.Operands[1]))
		}
		return (uint16(opcode) << 12) | (uint16(rc) << 8) | (offset & 0xFF), nil
	}

	return 0, fmt.Errorf("cannot encode instruction: %s", instr.Opcode)
}

func (a *Assembler) parseRegister(reg string) uint8 {
	reg = strings.ToUpper(reg)
	if strings.HasPrefix(reg, "R") {
		val, _ := strconv.ParseInt(reg[1:], 16, 32)
		return uint8(val)
	}
	return 0
}

func (a *Assembler) parseImmediate(imm string) int {
	imm = strings.TrimSpace(imm)
	if strings.HasPrefix(imm, "0X") || strings.HasPrefix(imm, "0x") {
		val, _ := strconv.ParseInt(imm[2:], 16, 32)
		return int(val)
	}
	if strings.HasPrefix(imm, "0B") || strings.HasPrefix(imm, "0b") {
		val, _ := strconv.ParseInt(imm[2:], 2, 32)
		return int(val)
	}
	val, _ := strconv.Atoi(imm)
	return val
}

// ============================================================================
// backend/go/simulator/disassembler.go
// Disassembler implementation in Go
// ============================================================================

package simulator

import "fmt"

type Disassembler struct {
	Labels map[uint16]string
}

type DisassembledInstruction struct {
	Address     uint16 `json:"address"`
	MachineCode uint16 `json:"machineCode"`
	Mnemonic    string `json:"mnemonic"`
	Operands    string `json:"operands"`
	Comment     string `json:"comment"`
}

func NewDisassembler() *Disassembler {
	return &Disassembler{
		Labels: make(map[uint16]string),
	}
}

func (d *Disassembler) Disassemble(machineCode []uint16, startAddress uint16) []DisassembledInstruction {
	result := []DisassembledInstruction{}
	address := startAddress

	for _, instruction := range machineCode {
		disasm := d.disassembleInstruction(instruction, address)
		result = append(result, disasm)
		address += 2
	}

	return result
}

func (d *Disassembler) disassembleInstruction(instruction uint16, address uint16) DisassembledInstruction {
	if instruction == 0x0000 {
		return DisassembledInstruction{
			Address:     address,
			MachineCode: instruction,
			Mnemonic:    "NOP",
			Operands:    "",
			Comment:     "No operation",
		}
	}

	if instruction == 0xFFFF {
		return DisassembledInstruction{
			Address:     address,
			MachineCode: instruction,
			Mnemonic:    "HALT",
			Operands:    "",
			Comment:     "Stop execution",
		}
	}

	opcode := (instruction >> 12) & 0xF
	rd := (instruction >> 8) & 0xF
	rs := (instruction >> 4) & 0xF
	rt := instruction & 0xF
	imm8 := instruction & 0xFF
	offset := instruction & 0xF
	addr12 := instruction & 0xFFF

	var mnemonic, operands, comment string

	switch opcode {
	case 0x1:
		mnemonic = "ADD"
		operands = fmt.Sprintf("R%X, R%X, R%X", rd, rs, rt)
		comment = fmt.Sprintf("R%X = R%X + R%X", rd, rs, rt)

	case 0x2:
		mnemonic = "ADDI"
		operands = fmt.Sprintf("R%X, 0x%02X", rd, imm8)
		comment = fmt.Sprintf("R%X = R%X + 0x%02X", rd, rd, imm8)

	case 0x3:
		mnemonic = "SUB"
		operands = fmt.Sprintf("R%X, R%X, R%X", rd, rs, rt)
		comment = fmt.Sprintf("R%X = R%X - R%X", rd, rs, rt)

	case 0xC:
		mnemonic = "LOADI"
		operands = fmt.Sprintf("R%X, 0x%02X", rd, imm8)
		comment = fmt.Sprintf("R%X = 0x%02X", rd, imm8)

	case 0xD:
		mnemonic = "JMP"
		operands = fmt.Sprintf("0x%03X", addr12)
		comment = fmt.Sprintf("PC = 0x%03X", addr12)

	case 0xE:
		mnemonic = "BEQ"
		operands = fmt.Sprintf("R%X, %d", rd, int8(imm8))
		comment = "Branch if zero"

	case 0xF:
		mnemonic = "BNE"
		operands = fmt.Sprintf("R%X, %d", rd, int8(imm8))
		comment = "Branch if not zero"

	default:
		mnemonic = fmt.Sprintf("UNKNOWN_%X", opcode)
		operands = ""
		comment = fmt.Sprintf("Unknown opcode: 0x%X", opcode)
	}

	return DisassembledInstruction{
		Address:     address,
		MachineCode: instruction,
		Mnemonic:    mnemonic,
		Operands:    operands,
		Comment:     comment,
	}
}
