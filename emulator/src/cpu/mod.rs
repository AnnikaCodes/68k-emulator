//! CPU emulation
//!
//!  Each CPU has its own file, so you can do things like
//! ```
//! use emulator::cpu::isa_68000;
//! ```
//! to only use the 68000 instructions.
//!
//! However, we don't support non-68000s yet, so it's not terribly relevant.

use std::fmt::Display;

use crate::{ram::Memory, M68kInteger};
pub mod addressing;
pub mod isa_68000;
pub mod registers;
use registers::*;

use self::addressing::AddressMode;

/// Trait for all ISA enums to implement
pub trait InstructionSet {
    fn execute(&self, cpu: &mut CPU<impl Memory>) -> Result<(), CPUError>;
}

#[derive(Debug)]
pub enum CPUError {
    MemoryOutOfBoundsAccess(u32),
    WriteToReadOnly(String),
    WrongSizeInteger(M68kInteger),
}

pub struct CPU<M: Memory> {
    pub registers: Registers,
    pub memory: M,
}

impl<M> Default for CPU<M> where M: Memory {
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
        }
    }

    /// Syntactic sugar
    pub fn run_instruction(&mut self, instruction: impl InstructionSet) -> Result<(), CPUError> {
        instruction.execute(self)
    }

    pub fn get_address_value(&mut self, addr: AddressMode) -> Result<M68kInteger, CPUError> {
        addr.get_value(self)
    }
}

impl<M> Display for CPU<M>
where
    M: Memory,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.registers, self.memory)
    }
}
