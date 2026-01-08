mod vm;

use vm::CVEREVM;

fn main() {
    let program = vec![
        0xC105, 0xC203, 0x1312, 0xFFFF,
        0xC100, 0xC20A, 0x2101, 0x3321, 0xF3FD, 0xFFFF,
    ]; // Machine code to run

    let mut vm = CVEREVM::new();
    vm.load_program(&program, 0);
    if let Err(e) = vm.run(1000) {
        eprintln!("Error: {}", e);
    }
}