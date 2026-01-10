// ============================================================================
// desktop/rust/src/lib.rs
// Main library file - re-exports modules
// ============================================================================

pub mod memory;
pub mod registers;
pub mod decoder;
pub mod vm;

pub use vm::CVEREVM;
pub use memory::Memory;
pub use registers::{RegisterFile, StatusFlags};
pub use decoder::{InstructionDecoder, DecodedInstruction, InstructionFormat};
