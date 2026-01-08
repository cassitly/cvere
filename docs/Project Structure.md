# Project Structure for new developers
To get aclimated to the project more easily. This file is to be updated for each structure change / update, this is a requirement.

```
cvere-reality/
├── core/                          # Core VM engine
│   ├── rust/                      # High-performance VM runtime
│   │   ├── src/
│   │   │   ├── vm.rs             # Virtual machine executor
│   │   │   ├── memory.rs         # Memory management
│   │   │   ├── registers.rs      # Register file
│   │   │   └── decoder.rs        # Instruction decoder
│   │   └── Cargo.toml
│   │
│   ├── python/                    # Instruction set tools
│   │   ├── isa_designer.py       # ISA specification
│   │   ├── assembler.py          # Hex assembler
│   │   └── disassembler.py       # Disassembler
│   │
│   └── cpp/                       # Performance-critical components
│       ├── pipeline.cpp          # Instruction pipeline
│       └── cache_sim.cpp         # Cache simulator
│
├── backend/                       # API and services
│   ├── go/                        # High-concurrency server
│   │   ├── main.go
│   │   ├── api/                  # REST/GraphQL API
│   │   └── simulator/            # Simulation orchestration
│   │
│   └── elixir/                    # Real-time event streaming
│       └── lib/
│           └── realtime_viz.ex   # WebSocket for live updates
│
├── frontend/                      # Visualization layer
│   ├── typescript/
│   │   ├── src/
│   │   │   ├── components/       # React/Vue components
│   │   │   ├── graph/            # D3.js/Three.js visualizations
│   │   │   └── wasm-bindings/    # WebAssembly bindings
│   │   └── package.json
│   │
│   └── shaders/                   # GPU-accelerated rendering
│       └── flow.glsl
│
├── tools/                         # Development tools
│   ├── compiler/                  # Custom language compiler
│   │   └── ocaml/                # Compiler in OCaml/Haskell
│   │
│   └── debugger/                  # Interactive debugger
│       └── c/                     # Low-level debugging tools
│
├── examples/                      # Example programs
│   ├── hex_programs/
│   └── benchmarks/
│
└── docs/                          # Documentation
    ├── ISA_SPEC.md
    └── ARCHITECTURE.md
```