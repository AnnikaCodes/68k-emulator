//! CPU emulation
//!
//! In the future, CPU features (e.g. 68000 vs 68030) may be configurable via Cargo features.
//!
//! However, we don't support non-68000s yet, so it's not terribly relevant.

use colored::*;
use std::fmt::Display;

use crate::{
    parsers::{binary::MachineCodeParser, Parser},
    ram::Memory,
    EmulationError,
};
pub mod addressing;
pub mod isa_68000;
pub mod registers;
use registers::*;

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
    pub fn run_one_cycle(&mut self) -> Result<(), EmulationError> {
        // Fetch
        let pc = self.registers.get(Register::ProgramCounter);

        let binary = self.memory.read_bytes(pc, 8)?;

        // Decode
        let (instruction, size, bytes_taken) = self.parser.parse(binary)?;

        // Execute
        println!("{}: {:?}", "Execute".green().bold(), instruction);
        instruction.execute(self, size)?;

        // Increment PC only if the instruction didn't alter it itself
        if pc == self.registers.get(Register::ProgramCounter) {
            self.registers
                .set(Register::ProgramCounter, pc + bytes_taken as u32);
        }
        Ok(())
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
