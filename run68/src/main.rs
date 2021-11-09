use std::io::Write;
use std::path::PathBuf;

use emulator::cpu::registers::Register;
use emulator::ram::Memory;
use emulator::{
    cpu::{isa_68000::ISA68000, CPUError, InstructionSet, CPU},
    parsers::{assembly::AssemblyInterpreter, Interpreter, ParseError},
    ram::VecBackedMemory,
};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    author = "Annika L.",
    about = "Executes code on an experimental 68000 emulator"
)]
struct Options {
    #[structopt(
        help = "File to run. Currently, only raw binary files (compiled with the `-Wl,--oformat=binary` gcc flags) are supported."
    )]
    file: PathBuf,
    #[structopt(
        short = "v",
        long = "verbose",
        help = "Prints the CPU state after each instruction, instead of only after the file is run"
    )]
    verbose: bool,
}

fn main() {
    let options = Options::from_args();
    let code = std::fs::read(&options.file).expect("Could not read file");
    let mut cpu = CPU::<VecBackedMemory>::new(8_192 * 1_024); // 8MB
    cpu.memory
        .write_bytes(cpu.registers.get(Register::ProgramCounter), code)
        .unwrap();
    println!("{}", cpu);
    let mut cycles = 1;
    loop {
        if options.verbose {
            println!("{}> Cycle #{}", "=".repeat(cycles), cycles);
        }
        match cpu.run_one_cycle() {
            Ok(_) => {
                cycles += 1;
                if options.verbose {
                    println!("{}", cpu);
                }
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                break;
            }
        }
    }
    println!("{}", cpu);
}
