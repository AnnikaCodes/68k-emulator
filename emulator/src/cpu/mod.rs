//! CPU emulation
//!
//!  Each CPU has its own file, so you can do things like
//! ```
//! use emulator::cpu::isa_68000;
//! ```
//! to only use the 68000 instructions.
//!
//! However, we don't support non-68000s yet, so it's not terribly relevant.

use colored::*;
use std::fmt::Display;

use crate::{
    cpu::isa_68000::ISA68000, parsers::binary::MachineCodeParser, ram::Memory, M68kInteger,
    OperandSize,
};
pub mod addressing;
pub mod isa_68000;
pub mod registers;
use registers::*;

use self::addressing::AddressMode;

/// Trait for all ISA enums to implement
pub trait InstructionSet {
    fn execute(&self, cpu: &mut CPU<impl Memory>, size: OperandSize) -> Result<(), CPUError>;
}

#[derive(Debug)]
pub enum CPUError {
    MemoryOutOfBoundsAccess(u32),
    WriteToReadOnly(String),
    WrongSizeInteger(M68kInteger),
    InvalidOperandSize(i32),
}

pub struct CPU<M: Memory> {
    pub registers: Registers,
    pub memory: M,
    pub parser: MachineCodeParser,
}

impl<M> Default for CPU<M>
where
    M: Memory,
{
    fn default() -> Self {
        Self::new(1024)
    }
}

impl<M> CPU<M>
where
    M: Memory,
{
    pub fn new(ram_size_in_bytes: usize) -> Self {
        Self {
            registers: Registers::new(),
            memory: M::new(ram_size_in_bytes),
            parser: MachineCodeParser::default(),
        }
    }

    /// Runs one cycle of the CPU:
    ///
    /// - Fetch the instruction
    ///
    /// - Decode the instruction
    ///
    /// - Execute the instruction
    pub fn run_one_cycle(&mut self) -> Result<(), CPUError> {
        // Fetch
        let pc = self.registers.get(Register::ProgramCounter);
        let binary = self.memory.read_bytes(pc, 8)?;
        // Decode
        let decoded_instruction =
        // TODO: should this be be?
        // it works for directly copying from a `gcc -Wl,--oformat=binary`...
        m68kdecode::decode_instruction(binary.as_slice()).unwrap();
        self.registers.set(
            Register::ProgramCounter,
            pc + decoded_instruction.bytes_used,
        );
        // Execute
        let size = decoded_instruction.instruction.size;
        let parsed_instruction: ISA68000 = decoded_instruction.instruction.into();
        println!("{}: {:?}", "Execute".green().bold(), parsed_instruction);
        parsed_instruction.execute(self, OperandSize::from_size_in_bytes(size)?)
    }
}

impl<M> Display for CPU<M>
where
    M: Memory,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.registers)
    }
}
