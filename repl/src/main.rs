
use emulator::cpu::CPU;

fn main() {
    println!("Welcome to the Motorola 68000 Assembly REPL!");
    let mut cpu = emulator::cpu::CPU::new();
    // print_machine_state(); // registers, RAM, etc
    // loop {
    //     let instruction_and_args = get_input();
    //     cpu.run_instruction(instruction_and_args);
    //     print_machine_state();
    // }
}
