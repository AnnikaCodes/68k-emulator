use std::io::Write;

use emulator::{cpu::{CPU, CPUError, InstructionSet, isa_68000::ISA68000}, parsers::{Interpreter, ParseError, assembly::AssemblyInterpreter}, ram::VecBackedMemory};


fn get_input(interpreter: &mut impl Interpreter<String>) -> Result<impl InstructionSet, ParseError> {
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
    println!("{}", cpu);
    loop {
        match get_input(&mut interpreter) {
            Ok(instruction) => match cpu.run_instruction(instruction) {
                Ok(_) => println!("{}", cpu),
                Err(e) => eprintln!("CPU Error: {:?}", e)
            },
            Err(e) => eprintln!("Parsing Error: {:?}", e)
        };
    }
}
