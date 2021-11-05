use std::io::Write;

use emulator::ram::Memory;
use emulator::{
    cpu::{isa_68000::ISA68000, CPUError, InstructionSet, CPU},
    parsers::{assembly::AssemblyInterpreter, Interpreter, ParseError},
    ram::VecBackedMemory,
};

fn get_input(
    interpreter: &mut impl Interpreter<String>,
) -> Result<impl InstructionSet, ParseError> {
    let mut input = String::new();
    print!("> ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();
    interpreter.parse_instruction(input)
}

fn main() {
    println!("Welcome to the Motorola 68000 Assembly REPL!");
    let mut cpu = CPU::<VecBackedMemory>::new(32_768); // 32K
    let mut interpreter = AssemblyInterpreter::new();
    cpu.memory.write_long(0x00, 0x06400064);
    println!("{}", cpu);
    loop {
        println!("Running 1 cycle.");
        cpu.run_one_cycle().unwrap();
        println!("{}", cpu);
    }
}
