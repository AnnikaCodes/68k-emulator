use emulator::{cpu::CPU, ram::VecBackedMemory};

fn main() {
    println!("Welcome to the Motorola 68000 Assembly REPL!");
    let mut cpu = CPU::<VecBackedMemory>::new(32_768); // 32K
                                                       // print_machine_state(); // registers, RAM, etc
                                                       // loop {
                                                       //     let instruction_and_args = get_input();
                                                       //     cpu.run_instruction(instruction_and_args);
                                                       //     print_machine_state();
                                                       // }
}
